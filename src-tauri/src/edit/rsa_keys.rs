use std::path::PathBuf;

use super::error::{EditError, EditResult};

fn key_search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        dirs.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            dirs.push(dir.to_path_buf());
        }
    }
    dirs
}

fn read_local_key(name: &str) -> Option<Vec<u8>> {
    for base in key_search_dirs() {
        let path = base.join(name);
        if path.is_file() {
            return std::fs::read(path).ok();
        }
    }
    None
}

fn read_bundled_key(name: &str) -> Option<Vec<u8>> {
    match name {
        "tibia_rsa.key" => Some(include_bytes!("../../../tibia_rsa.key").to_vec()),
        "otserv_rsa.key" => Some(include_bytes!("../../../otserv_rsa.key").to_vec()),
        _ => None,
    }
}

/// Matches [opentibiabr/client-editor](https://github.com/opentibiabr/client-editor):
/// read `tibia_rsa.key` / `otserv_rsa.key` from the working directory first,
/// then beside the executable, then the keys shipped with this application.
pub fn read_key_file(name: &str) -> EditResult<Vec<u8>> {
    if let Some(bytes) = read_local_key(name) {
        return Ok(bytes);
    }

    read_bundled_key(name).ok_or(EditError::RsaNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_keys_match_upstream_files() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");
        for name in ["tibia_rsa.key", "otserv_rsa.key"] {
            let on_disk = std::fs::read(root.join(name)).expect("repo key file");
            let bundled = read_bundled_key(name).expect("bundled key");
            assert_eq!(on_disk, bundled, "{name} should match repo file");
            assert_eq!(on_disk.len(), 256, "{name} should be 256-byte hex payload");
        }
    }
}
