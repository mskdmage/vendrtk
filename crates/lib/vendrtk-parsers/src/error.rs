pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("empty OCR content")]
    EmptyOcrContent,

    #[error("schema validation failed: {details}")]
    SchemaValidation { details: String },

    #[error("LLM request failed: {0}")]
    LlmRequestFailed(String),

    #[error(transparent)]
    Ocr(#[from] vendrtk_ocr::error::Error),
}
