use std::collections::HashSet;

pub fn find_all_offsets(data: &[u8], needle: &[u8]) -> Vec<usize> {
    let mut offsets = Vec::new();
    if needle.is_empty() || data.len() < needle.len() {
        return offsets;
    }
    let mut search_offset = 0;
    while search_offset <= data.len().saturating_sub(needle.len()) {
        let Some(index) = data[search_offset..].windows(needle.len()).position(|w| w == needle) else {
            break;
        };
        let offset = search_offset + index;
        offsets.push(offset);
        search_offset = offset + needle.len();
    }
    offsets
}

pub fn format_offsets_limited(offsets: &[usize], limit: usize) -> String {
    if offsets.is_empty() {
        return "none".to_string();
    }
    let (display, truncated) = if limit > 0 && offsets.len() > limit {
        (&offsets[..limit], offsets.len() - limit)
    } else {
        (offsets, 0)
    };
    let mut formatted: Vec<String> = display.iter().map(|o| format!("0x{o:X}")).collect();
    if truncated > 0 {
        formatted.push(format!("... +{truncated} more"));
    }
    formatted.join(", ")
}

pub fn format_nearest_offsets(anchor: usize, offsets: &[usize], limit: usize) -> String {
    if offsets.is_empty() {
        return "none".to_string();
    }
    let mut display: Vec<usize> = offsets.to_vec();
    display.sort_by(|left, right| {
        let ld = abs_distance(anchor, *left);
        let rd = abs_distance(anchor, *right);
        ld.cmp(&rd).then_with(|| left.cmp(right))
    });
    let truncated = if limit > 0 && display.len() > limit {
        let t = display.len() - limit;
        display.truncate(limit);
        t
    } else {
        0
    };
    let formatted: Vec<String> = display
        .iter()
        .map(|offset| {
            let delta = *offset as i64 - anchor as i64;
            if delta >= 0 {
                format!("0x{offset:X}(+0x{delta:X})")
            } else {
                let abs_delta = (-delta) as u64;
                format!("0x{offset:X}(-0x{abs_delta:X})")
            }
        })
        .collect();
    let mut result = formatted.join(", ");
    if truncated > 0 {
        result.push_str(&format!(", ... +{truncated} more"));
    }
    result
}

pub fn format_bytes(data: &[u8]) -> String {
    if data.is_empty() {
        return "none".to_string();
    }
    data.iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn bytes_around(data: &[u8], center: usize, radius: usize) -> (usize, Vec<u8>) {
    if data.is_empty() || center >= data.len() {
        return (0, Vec::new());
    }
    let start = center.saturating_sub(radius);
    let end = (center + radius).min(data.len());
    (start, data[start..end].to_vec())
}

pub fn bytes_around_range(data: &[u8], start: usize, length: usize, radius: usize) -> (usize, Vec<u8>) {
    if data.is_empty() || start >= data.len() || length == 0 {
        return (0, Vec::new());
    }
    let end = (start + length).min(data.len());
    let context_start = start.saturating_sub(radius);
    let context_end = (end + radius).min(data.len());
    (context_start, data[context_start..context_end].to_vec())
}

pub fn abs_distance(left: usize, right: usize) -> usize {
    if left >= right {
        left - right
    } else {
        right - left
    }
}

pub fn difference_strings(left: &[String], right: &[String]) -> Vec<String> {
    let right_set: HashSet<_> = right.iter().collect();
    let mut difference: Vec<String> = left
        .iter()
        .filter(|v| !right_set.contains(v))
        .cloned()
        .collect();
    difference.sort();
    difference
}

pub fn utf16_le_bytes(value: &str) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(value.len() * 2);
    for ch in value.chars() {
        let code = ch as u32;
        if code > 0xffff {
            continue;
        }
        encoded.push(code as u8);
        encoded.push((code >> 8) as u8);
    }
    encoded
}

pub fn is_windows_executable(data: &[u8]) -> bool {
    if data.len() < 0x40 || data[0] != b'M' || data[1] != b'Z' {
        return false;
    }
    let pe_offset = u32::from_le_bytes(data[0x3c..0x40].try_into().unwrap()) as usize;
    if pe_offset + 4 > data.len() {
        return false;
    }
    data[pe_offset] == b'P'
        && data[pe_offset + 1] == b'E'
        && data[pe_offset + 2] == 0
        && data[pe_offset + 3] == 0
}

pub fn resolve_source_executable(tibia_exe: &std::path::Path, source: Option<&std::path::Path>) -> std::path::PathBuf {
    if let Some(src) = source {
        return src.to_path_buf();
    }
    let default_source = tibia_exe
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("client - original.exe");
    if default_source != tibia_exe && default_source.exists() {
        return default_source;
    }
    tibia_exe.to_path_buf()
}

