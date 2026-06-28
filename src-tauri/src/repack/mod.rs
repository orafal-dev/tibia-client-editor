mod types;

pub use types::*;

use std::fs;
use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepackError {
    #[error("failed to read client info: {0}")]
    ReadClientInfo(#[source] std::io::Error),
    #[error("failed to read assets info: {0}")]
    ReadAssetsInfo(#[source] std::io::Error),
    #[error("failed to create repacked directory: {0}")]
    CreateDir(#[source] std::io::Error),
    #[error("failed to repack file: {0}")]
    RepackFile(String),
    #[error("failed to save client info: {0}")]
    SaveClientInfo(#[source] std::io::Error),
    #[error("failed to save assets info: {0}")]
    SaveAssetsInfo(#[source] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RepackResult {
    pub logs: Vec<String>,
    pub client_files: usize,
    pub assets_files: usize,
    pub revision: i32,
    pub output_dir: String,
}

pub fn repack(src: &Path, dst: &Path, platform: &str) -> Result<RepackResult, RepackError> {
    let mut logs = Vec::new();
    logs.push(format!(
        "Repacking {} into {}",
        src.display(),
        dst.display()
    ));

    let client_file_path = src.join("client.json");
    let assets_file_path = src.join("assets.json");

    let mut client_info = read_client_info(&client_file_path)?;
    let mut assets_info = read_assets_info(&assets_file_path)?;

    logs.push(format!(
        "Repacking {} client files and {} assets files",
        client_info.files.len(),
        assets_info.files.len()
    ));

    fs::create_dir_all(dst).map_err(RepackError::CreateDir)?;

    for file in &mut client_info.files {
        repack_file(file, src, dst).map_err(|e| RepackError::RepackFile(e.to_string()))?;
    }

    for file in &mut assets_info.files {
        repack_file(file, src, dst).map_err(|e| RepackError::RepackFile(e.to_string()))?;
    }

    client_info.files.retain(|f| f.unpacked_size != 0);
    assets_info.files.retain(|f| f.unpacked_size != 0);

    let client_count = client_info.files.len();
    let assets_count = assets_info.files.len();
    logs.push(format!(
        "Repacked {} client files and {} assets files",
        client_count, assets_count
    ));

    client_info.revision += 1;

    save_client_info(&dst.join(format!("client.{platform}.json")), &client_info)?;
    save_assets_info(&dst.join("assets.mac.json"), &assets_info, "mac")?;
    save_assets_info(&dst.join("assets.windows.json"), &assets_info, "windows")?;

    Ok(RepackResult {
        logs,
        client_files: client_count,
        assets_files: assets_count,
        revision: client_info.revision,
        output_dir: dst.display().to_string(),
    })
}

fn read_client_info(file_path: &Path) -> Result<ClientInfo, RepackError> {
    let file = fs::read(file_path).map_err(RepackError::ReadClientInfo)?;
    serde_json::from_slice(&file).map_err(RepackError::from)
}

fn read_assets_info(file_path: &Path) -> Result<AssetsInfo, RepackError> {
    let file = fs::read(file_path).map_err(RepackError::ReadAssetsInfo)?;
    serde_json::from_slice(&file).map_err(RepackError::from)
}

fn repack_file(file: &mut FileEntry, src: &Path, dst: &Path) -> Result<(), std::io::Error> {
    let local_file_path = src.join(&file.local_file);
    let packed_file_path = dst.join(&file.url);

    if !local_file_path.exists() {
        return Ok(());
    }

    if let Some(parent) = packed_file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let local_ext = local_file_path.extension().and_then(|e| e.to_str());
    let packed_ext = packed_file_path.extension().and_then(|e| e.to_str());

    if local_ext == packed_ext {
        fs::copy(&local_file_path, &packed_file_path)?;
    } else {
        let local_data = fs::read(&local_file_path)?;
        let compressed = compress_lzma(&local_data)?;
        fs::write(&packed_file_path, compressed)?;
    }

    let (unpacked_hash, unpacked_size) = calculate_hash_and_size(&local_file_path)?;
    let (packed_hash, packed_size) = calculate_hash_and_size(&packed_file_path)?;

    file.packed_hash = packed_hash;
    file.packed_size = packed_size;
    file.unpacked_hash = unpacked_hash;
    file.unpacked_size = unpacked_size;

    Ok(())
}

fn compress_lzma(data: &[u8]) -> Result<Vec<u8>, std::io::Error> {
    let mut output = Vec::new();
    lzma_rs::lzma_compress(&mut std::io::Cursor::new(data), &mut output)?;
    Ok(output)
}

fn calculate_hash_and_size(file_path: &Path) -> Result<(String, i64), std::io::Error> {
    use sha2::{Digest, Sha256};
    let data = fs::read(file_path)?;
    let hash = hex::encode(Sha256::digest(&data));
    Ok((hash, data.len() as i64))
}

fn save_client_info(file_path: &Path, client_info: &ClientInfo) -> Result<(), RepackError> {
    let client_json = serde_json::to_string_pretty(client_info)?;
    fs::write(file_path, client_json).map_err(RepackError::SaveClientInfo)?;
    Ok(())
}

fn save_assets_info(
    file_path: &Path,
    assets_info: &AssetsInfo,
    platform: &str,
) -> Result<(), RepackError> {
    let mut copy = assets_info.clone();
    for file in &mut copy.files {
        file.local_file = file.local_file.replace("Contents/Resources/", "");
        if platform == "mac" {
            file.local_file = format!("Contents/Resources/{}", file.local_file);
        }
    }
    let assets_json = serde_json::to_string_pretty(&copy)?;
    fs::write(file_path, assets_json).map_err(RepackError::SaveAssetsInfo)?;
    Ok(())
}
