use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::error::{EditError, EditResult};
use super::patterns::URL_PROPERTIES;
use super::types::LogSink;

pub fn validate_config_values(
    mut values: HashMap<String, String>,
) -> EditResult<HashMap<String, String>> {
    let mut missing = Vec::new();
    for prop in URL_PROPERTIES {
        if !values.contains_key(*prop) {
            missing.push(*prop);
        }
    }
    if !missing.is_empty() {
        return Err(EditError::MissingProperties(missing.join(", ")));
    }

    for prop in URL_PROPERTIES {
        values.entry(prop.to_string()).or_default();
    }
    Ok(values)
}

pub fn resolve_config_values(
    config_path: Option<&Path>,
    config_values: Option<HashMap<String, String>>,
) -> EditResult<HashMap<String, String>> {
    match (config_path, config_values) {
        (Some(path), _) => load_config_values(path),
        (None, Some(values)) => validate_config_values(values),
        (None, None) => Err(EditError::Config(
            "No configuration provided. Set URLs in the UI or choose a config.toml file.".into(),
        )),
    }
}

pub fn load_config_values(config_path: &Path) -> EditResult<HashMap<String, String>> {
    let content =
        std::fs::read_to_string(config_path).map_err(|e| EditError::Config(e.to_string()))?;
    let parsed: toml::Value =
        toml::from_str(&content).map_err(|e| EditError::Config(e.to_string()))?;

    let mut missing = Vec::new();
    for prop in URL_PROPERTIES {
        if parsed.get(*prop).is_none() {
            missing.push(*prop);
        }
    }
    if !missing.is_empty() {
        return Err(EditError::MissingProperties(missing.join(", ")));
    }

    let mut values = HashMap::new();
    for prop in URL_PROPERTIES {
        let value = parsed
            .get(*prop)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        values.insert(prop.to_string(), value);
    }
    Ok(values)
}

const CONFIG_INI_FILE_NAME: &str = "config.ini";
const CONFIG_INI_DIR_NAME: &str = "conf";
const CONFIG_INI_START_MARKER: &str = "[URLS]";

pub fn sync_config_ini(
    log: &mut LogSink,
    tibia_path: &Path,
    tibia_binary: &[u8],
    config_values: &HashMap<String, String>,
) -> EditResult<()> {
    let Some(embedded_data) = extract_embedded_config_ini_block(tibia_binary) else {
        log.warn(format!(
            "Embedded config.ini block starting at {CONFIG_INI_START_MARKER:?} was not found; {CONFIG_INI_FILE_NAME} sync skipped"
        ));
        return Ok(());
    };

    let Some(mut embedded_config) = parse_embedded_config_ini(&embedded_data) else {
        log.warn(format!(
            "Embedded config.ini block could not be parsed; {CONFIG_INI_FILE_NAME} sync skipped"
        ));
        return Ok(());
    };
    embedded_config = override_embedded_config_values(embedded_config, config_values);

    let (config_path, config_exists) = resolve_config_ini_path(tibia_path);
    let config_data = if config_exists {
        std::fs::read(&config_path).map_err(|e| {
            log.error(format!("Unable to read {}: {e}", config_path.display()));
            EditError::Io(e)
        })?
    } else {
        Vec::new()
    };

    let (updated_config, changed_count, added_count, removed_count, changed) =
        update_config_ini_content(&config_data, &embedded_config);

    if !changed {
        log.info(format!("{CONFIG_INI_FILE_NAME} already up to date"));
        return Ok(());
    }

    std::fs::write(&config_path, updated_config).map_err(|e| {
        log.error(format!("Unable to write {}: {e}", config_path.display()));
        EditError::Io(e)
    })?;

    if config_exists {
        log.patch(format!(
            "{CONFIG_INI_FILE_NAME} updated from embedded client config ({changed_count} outdated value(s), {added_count} new key(s), {removed_count} obsolete key(s) removed)"
        ));
    } else {
        log.patch(format!(
            "{CONFIG_INI_FILE_NAME} created from embedded client config ({added_count} key(s))"
        ));
    }
    Ok(())
}

fn resolve_config_ini_path(tibia_path: &Path) -> (PathBuf, bool) {
    let tibia_dir = tibia_path.parent().unwrap_or_else(|| Path::new("."));
    let conf_config_path = tibia_dir
        .join("..")
        .join(CONFIG_INI_DIR_NAME)
        .join(CONFIG_INI_FILE_NAME);
    let bin_config_path = tibia_dir.join(CONFIG_INI_FILE_NAME);

    for candidate in [&conf_config_path, &bin_config_path] {
        if candidate.is_file() {
            return (candidate.clone(), true);
        }
    }

    if conf_config_path
        .parent()
        .map(|p| p.is_dir())
        .unwrap_or(false)
    {
        return (conf_config_path, false);
    }

    (bin_config_path, false)
}

fn extract_embedded_config_ini_block(data: &[u8]) -> Option<Vec<u8>> {
    let start = data
        .windows(CONFIG_INI_START_MARKER.len())
        .position(|w| w == CONFIG_INI_START_MARKER.as_bytes())?;
    let mut end = start;
    while end < data.len() {
        let value = data[end];
        if value == 0 {
            break;
        }
        if value == b'\r' || value == b'\n' || value == b'\t' || (0x20..=0x7e).contains(&value) {
            end += 1;
        } else {
            break;
        }
    }
    if end <= start {
        return None;
    }
    Some(data[start..end].to_vec())
}

#[derive(Clone)]
struct ConfigIniKeyValue {
    key: String,
    value: String,
}

#[derive(Clone)]
struct ConfigIniSection {
    name: String,
    keys: Vec<ConfigIniKeyValue>,
    key_values: HashMap<String, String>,
}

#[derive(Clone)]
struct EmbeddedConfigIni {
    sections: Vec<ConfigIniSection>,
    section_by_name: HashMap<String, ConfigIniSection>,
}

fn parse_embedded_config_ini(config_data: &[u8]) -> Option<EmbeddedConfigIni> {
    let mut config = EmbeddedConfigIni {
        sections: Vec::new(),
        section_by_name: HashMap::new(),
    };
    let mut current_section_index: Option<usize> = None;

    for line in split_config_ini_lines(config_data) {
        if let Some(section_name) = parse_config_ini_section_line(&line) {
            config.sections.push(ConfigIniSection {
                name: section_name,
                keys: Vec::new(),
                key_values: HashMap::new(),
            });
            current_section_index = Some(config.sections.len() - 1);
            continue;
        }
        let Some(idx) = current_section_index else {
            continue;
        };
        let Some((key, value)) = split_config_ini_line(&line) else {
            continue;
        };
        let section = &mut config.sections[idx];
        if section.key_values.contains_key(&key) {
            continue;
        }
        section.keys.push(ConfigIniKeyValue {
            key: key.clone(),
            value: value.clone(),
        });
        section.key_values.insert(key, value);
    }

    let total_keys: usize = config.sections.iter().map(|s| s.keys.len()).sum();
    if total_keys == 0 {
        return None;
    }
    for section in &config.sections {
        config
            .section_by_name
            .insert(section.name.clone(), section.clone());
    }
    Some(config)
}

fn override_embedded_config_values(
    mut embedded: EmbeddedConfigIni,
    config_values: &HashMap<String, String>,
) -> EmbeddedConfigIni {
    for section in &mut embedded.sections {
        for kv in &mut section.keys {
            if let Some(value) = config_values.get(&kv.key) {
                kv.value = value.clone();
                section.key_values.insert(kv.key.clone(), value.clone());
            }
        }
    }
    embedded.section_by_name.clear();
    for section in &embedded.sections {
        embedded
            .section_by_name
            .insert(section.name.clone(), section.clone());
    }
    embedded
}

fn update_config_ini_content(
    config_data: &[u8],
    embedded: &EmbeddedConfigIni,
) -> (Vec<u8>, usize, usize, usize, bool) {
    let line_ending = detect_line_ending(config_data);
    if config_data.is_empty() {
        return (
            render_embedded_config_ini(embedded, line_ending),
            0,
            embedded_config_key_count(embedded),
            0,
            true,
        );
    }

    let lines = split_config_ini_lines(config_data);
    let mut output: Vec<String> = Vec::new();
    let mut seen_sections: HashMap<String, ()> = HashMap::new();
    let mut seen_keys: HashMap<String, HashMap<String, ()>> = HashMap::new();
    let mut changed_count = 0;
    let mut added_count = 0;
    let mut removed_count = 0;
    let mut current_section = String::new();
    let mut current_managed = false;

    let append_missing_keys = |output: &mut Vec<String>,
                               section_name: &str,
                               seen_keys: &mut HashMap<String, HashMap<String, ()>>,
                               added_count: &mut usize| {
        let Some(section) = embedded.section_by_name.get(section_name) else {
            return;
        };
        let section_keys = seen_keys.entry(section_name.to_string()).or_default();
        for item in &section.keys {
            if section_keys.contains_key(&item.key) {
                continue;
            }
            output.push(format!("{}={}", item.key, item.value));
            section_keys.insert(item.key.clone(), ());
            *added_count += 1;
        }
    };

    for line in lines {
        if let Some(section_name) = parse_config_ini_section_line(&line) {
            if current_managed {
                append_missing_keys(
                    &mut output,
                    &current_section,
                    &mut seen_keys,
                    &mut added_count,
                );
            }
            current_section = section_name.clone();
            current_managed = embedded.section_by_name.contains_key(&current_section);
            if current_managed {
                seen_sections.insert(current_section.clone(), ());
                seen_keys.entry(current_section.clone()).or_default();
            }
            output.push(line);
            continue;
        }

        if current_managed {
            if let Some((key, _)) = split_config_ini_line(&line) {
                let section = embedded.section_by_name.get(&current_section).unwrap();
                if let Some(value) = section.key_values.get(&key) {
                    let section_keys = seen_keys.get_mut(&current_section).unwrap();
                    if section_keys.contains_key(&key) {
                        removed_count += 1;
                        continue;
                    }
                    section_keys.insert(key.clone(), ());
                    let next_line = format!("{key}={value}");
                    if line != next_line {
                        output.push(next_line);
                        changed_count += 1;
                        continue;
                    }
                } else {
                    removed_count += 1;
                    continue;
                }
            }
        }
        output.push(line);
    }

    if current_managed {
        append_missing_keys(
            &mut output,
            &current_section,
            &mut seen_keys,
            &mut added_count,
        );
    }

    for section in &embedded.sections {
        if seen_sections.contains_key(&section.name) {
            continue;
        }
        if !output.is_empty() && !output.last().map(|l| l.trim().is_empty()).unwrap_or(true) {
            output.push(String::new());
        }
        output.push(format!("[{}]", section.name));
        for item in &section.keys {
            output.push(format!("{}={}", item.key, item.value));
            added_count += 1;
        }
    }

    if changed_count == 0 && added_count == 0 && removed_count == 0 {
        return (config_data.to_vec(), 0, 0, 0, false);
    }

    let mut result = output.join(line_ending);
    result.push_str(line_ending);
    (
        result.into_bytes(),
        changed_count,
        added_count,
        removed_count,
        true,
    )
}

fn split_config_ini_lines(config_data: &[u8]) -> Vec<String> {
    let normalized = String::from_utf8_lossy(config_data)
        .replace("\r\n", "\n")
        .trim_end_matches('\0')
        .trim_end_matches('\n')
        .to_string();
    if normalized.is_empty() {
        return Vec::new();
    }
    normalized.split('\n').map(|s| s.to_string()).collect()
}

fn parse_config_ini_section_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.len() < 3 || !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return None;
    }
    let section_name = trimmed[1..trimmed.len() - 1].trim();
    if section_name.is_empty() {
        return None;
    }
    Some(section_name.to_string())
}

fn split_config_ini_line(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
        return None;
    }
    let separator = trimmed.find('=')?;
    if separator == 0 {
        return None;
    }
    let key = trimmed[..separator].trim().to_string();
    let value = trimmed[separator + 1..].trim().to_string();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

fn render_embedded_config_ini(embedded: &EmbeddedConfigIni, line_ending: &str) -> Vec<u8> {
    let mut lines = Vec::new();
    for (index, section) in embedded.sections.iter().enumerate() {
        if index > 0 {
            lines.push(String::new());
        }
        lines.push(format!("[{}]", section.name));
        for item in &section.keys {
            lines.push(format!("{}={}", item.key, item.value));
        }
    }
    let mut result = lines.join(line_ending);
    result.push_str(line_ending);
    result.into_bytes()
}

fn embedded_config_key_count(embedded: &EmbeddedConfigIni) -> usize {
    embedded.sections.iter().map(|s| s.keys.len()).sum()
}

fn detect_line_ending(data: &[u8]) -> &'static str {
    if data.windows(2).any(|w| w == b"\r\n") {
        "\r\n"
    } else {
        "\n"
    }
}
