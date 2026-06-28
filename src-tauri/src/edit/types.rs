use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisReport {
    pub path: String,
    pub size: usize,
    pub sha256: String,
    pub is_windows_exe: bool,
    pub pe: PeInfo,
    pub patch_statuses: Vec<BattleyePatchStatus>,
    pub client_check_findings: Vec<ClientCheckFinding>,
    pub qt_indicators: Vec<String>,
    pub client_check_verdict: String,
    pub known_patch_coverage: usize,
    pub patchable_count: usize,
    pub original_patch_signature_count: usize,
    pub patched_patch_signature_count: usize,
    pub strong_unsupported_evidence_count: usize,
    pub suspicious_active_evidence_count: usize,
    pub high_risk_diagnostic_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeInfo {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_text: Option<String>,
    pub image_base: u64,
    pub sections: Vec<PeSectionInfo>,
    pub imports: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeSectionInfo {
    pub name: String,
    pub raw_start: usize,
    pub raw_end: usize,
    pub rva_start: usize,
    pub rva_end: usize,
    pub is_code: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BattleyePatchStatus {
    pub name: String,
    pub diagnostic_only: bool,
    pub high_risk_client_check: bool,
    pub false_positive_check: String,
    pub original_offset: Vec<usize>,
    pub patched_offset: Vec<usize>,
    pub expected_offset_hits: Vec<KnownPatchOffset>,
    pub expected_offset_misses: Vec<KnownPatchOffset>,
    pub aob_mask: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownPatchOffset {
    pub sha256: String,
    pub offset: usize,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCheckFinding {
    pub name: String,
    pub encoding: String,
    pub offsets: Vec<usize>,
    pub references: Vec<ClientCheckReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCheckReference {
    pub offset: usize,
    pub section: String,
    pub instruction: String,
    pub branch_offsets: Vec<usize>,
    pub call_offsets: Vec<usize>,
    pub pattern_matches: Vec<PatternMatch>,
    pub context_start: usize,
    #[serde(with = "base64_bytes")]
    pub context_bytes: Vec<u8>,
    pub known_patch_nearby: bool,
    pub strong_unsupported: bool,
    pub suspicious_active: bool,
    pub possible_instructions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub name: String,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOutput {
    pub logs: Vec<String>,
    pub diagnosis: DiagnosisReport,
    pub success: bool,
    pub output_path: String,
    pub backup_path: Option<String>,
    pub properties_patched: Vec<String>,
    pub properties_failed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnoseOutput {
    pub logs: Vec<String>,
    pub target: DiagnosisReport,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub baseline: Option<DiagnosisReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison_logs: Option<Vec<String>>,
}

mod base64_bytes {
    use base64::{engine::general_purpose::STANDARD, Engine};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&STANDARD.encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let s = String::deserialize(deserializer)?;
        STANDARD.decode(s).map_err(serde::de::Error::custom)
    }
}

pub struct LogSink {
    pub messages: Vec<String>,
}

impl LogSink {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        self.messages.push(format!("[INFO] {}", msg.into()));
    }

    pub fn warn(&mut self, msg: impl Into<String>) {
        self.messages.push(format!("[WARN] {}", msg.into()));
    }

    pub fn error(&mut self, msg: impl Into<String>) {
        self.messages.push(format!("[ERROR] {}", msg.into()));
    }

    pub fn patch(&mut self, msg: impl Into<String>) {
        self.messages.push(format!("[PATCH] {}", msg.into()));
    }
}

impl Default for LogSink {
    fn default() -> Self {
        Self::new()
    }
}
