pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("provider error: {0}")]
    Provider(#[from] providers::error::Error),

    #[error("OCR service error: {0}")]
    Service(String),

    #[error("invalid or missing OCR result")]
    InvalidResult,
}
