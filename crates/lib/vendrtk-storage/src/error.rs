use std::path::PathBuf;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid extension: {path}")]
    InvalidExtension { path: PathBuf },

    #[error("invalid magic bytes")]
    InvalidMagicBytes,
}
