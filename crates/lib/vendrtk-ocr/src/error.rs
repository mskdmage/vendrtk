pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("OCR service error: {0}")]
    Service(String),

    #[error("invalid or missing OCR result")]
    InvalidResult,
}
