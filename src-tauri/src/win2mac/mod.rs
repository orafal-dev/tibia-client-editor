use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::repack::AssetsInfo;

#[derive(Debug, Error)]
pub enum Win2MacError {
    #[error("failed to read assets info: {0}")]
    ReadAssetsInfo(#[source] std::io::Error),
    #[error("failed to save assets info: {0}")]
    SaveAssetsInfo(#[source] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Win2MacResult {
    pub logs: Vec<String>,
    pub files_updated: usize,
    pub output_path: String,
}

pub fn win2mac(src: &Path, dst: &Path) -> Result<Win2MacResult, Win2MacError> {
    let mut logs = Vec::new();
    logs.push(format!(
        "Converting assets paths from {} to {}",
        src.display(),
        dst.display()
    ));

    let mut assets_info = read_assets_info(src)?;
    let count = assets_info.files.len();

    for file in &mut assets_info.files {
        file.local_file = format!("Contents/Resources/{}", file.local_file);
    }

    save_assets_info(dst, &assets_info)?;
    logs.push(format!("Updated {count} file entries with macOS resource paths"));

    Ok(Win2MacResult {
        logs,
        files_updated: count,
        output_path: dst.display().to_string(),
    })
}

fn read_assets_info(file_path: &Path) -> Result<AssetsInfo, Win2MacError> {
    let file = fs::read(file_path).map_err(Win2MacError::ReadAssetsInfo)?;
    serde_json::from_slice(&file).map_err(Win2MacError::from)
}

fn save_assets_info(file_path: &Path, assets_info: &AssetsInfo) -> Result<(), Win2MacError> {
    let assets_json = serde_json::to_string_pretty(assets_info)?;
    fs::write(file_path, assets_json).map_err(Win2MacError::SaveAssetsInfo)?;
    Ok(())
}