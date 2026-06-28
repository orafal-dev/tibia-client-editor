use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    #[serde(rename = "localfile")]
    pub local_file: String,
    #[serde(rename = "packedhash")]
    pub packed_hash: String,
    #[serde(rename = "packedsize")]
    pub packed_size: i64,
    pub url: String,
    #[serde(rename = "unpackedhash")]
    pub unpacked_hash: String,
    #[serde(rename = "unpackedsize")]
    pub unpacked_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetsInfo {
    pub files: Vec<FileEntry>,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub version: String,
    pub files: Vec<FileEntry>,
    pub executable: String,
    pub generation: String,
    pub variant: String,
    pub revision: i32,
}
