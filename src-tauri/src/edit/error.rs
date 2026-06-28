use thiserror::Error;

#[derive(Debug, Error)]
pub enum EditError {
    #[error("failed to read config: {0}")]
    Config(String),
    #[error("missing properties in config: {0}")]
    MissingProperties(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Message(String),
    #[error("unsupported client-check evidence remains ({0} code reference(s))")]
    UnsupportedClientCheck(usize),
    #[error("partial client-check support and strict mode is enabled")]
    PartialStrict,
    #[error("warning client-check support and strict mode is enabled")]
    WarningStrict,
    #[error("RSA key not found")]
    RsaNotFound,
    #[error("invalid patched file size, original: {0}, modified: {1}")]
    InvalidPatchSize(usize, usize),
    #[error("invalid BattlEye patch {0}: replacement length differs from signature length")]
    InvalidPatch(String),
}

impl EditError {
    pub fn msg(s: impl Into<String>) -> Self {
        Self::Message(s.into())
    }
}

pub type EditResult<T> = Result<T, EditError>;
