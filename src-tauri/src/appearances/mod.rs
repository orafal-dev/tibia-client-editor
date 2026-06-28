pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/appearances.rs"));
}

pub mod config;
mod flags;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use prost::Message;
use thiserror::Error;

pub use config::AppearanceEdit;
pub use proto::Appearances;

#[derive(Debug, Error)]
pub enum AppearancesError {
    #[error("failed to read appearances file: {0}")]
    Read(#[source] std::io::Error),
    #[error("failed to write output file: {0}")]
    Write(#[source] std::io::Error),
    #[error("failed to parse protobuf: {0}")]
    Protobuf(String),
    #[error("failed to parse config: {0}")]
    Config(String),
    #[error("invalid appearance id: {0}")]
    InvalidId(String),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct AppearancesResult {
    pub logs: Vec<String>,
    pub edits_applied: usize,
    pub output_path: String,
}

pub fn resolve_edits(
    config_path: Option<&Path>,
    edits: Option<Vec<AppearanceEdit>>,
) -> Result<Vec<AppearanceEdit>, AppearancesError> {
    match (config_path, edits) {
        (Some(path), _) => config::load_edits(path),
        (None, Some(items)) => Ok(config::edits_from_inputs(items)),
        (None, None) => Ok(Vec::new()),
    }
}

pub fn edit_appearances(
    appearances_path: &Path,
    config_path: Option<&Path>,
    edits: Option<Vec<AppearanceEdit>>,
    output_path: Option<&Path>,
) -> Result<AppearancesResult, AppearancesError> {
    let mut logs = Vec::new();

    let data = fs::read(appearances_path).map_err(AppearancesError::Read)?;
    let mut appearances_data = Appearances::decode(data.as_slice())
        .map_err(|e| AppearancesError::Protobuf(e.to_string()))?;

    let edits = resolve_edits(config_path, edits)?;
    logs.push(format!("Loaded {} appearance edit(s)", edits.len()));

    let edit_map: HashMap<u32, AppearanceEdit> = edits
        .into_iter()
        .map(|e| {
            let id: u32 =
                e.id.parse()
                    .map_err(|_| AppearancesError::InvalidId(e.id.clone()))?;
            Ok((id, e))
        })
        .collect::<Result<_, AppearancesError>>()?;

    let mut applied = 0usize;
    for appearance in &mut appearances_data.object {
        let Some(id) = appearance.id else {
            continue;
        };
        if let Some(edit) = edit_map.get(&id) {
            appearance.flags = Some(flags::merge_flags(appearance.flags.take(), &edit.fields));
            applied += 1;
            logs.push(format!("Applied edit to appearance id {id}"));
        }
    }

    let out_path = output_path
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("appearances.out.dat"));

    let out = appearances_data.encode_to_vec();
    fs::write(&out_path, &out).map_err(AppearancesError::Write)?;

    logs.push(format!(
        "Wrote {} bytes to {}",
        out.len(),
        out_path.display()
    ));

    Ok(AppearancesResult {
        logs,
        edits_applied: applied,
        output_path: out_path.display().to_string(),
    })
}
