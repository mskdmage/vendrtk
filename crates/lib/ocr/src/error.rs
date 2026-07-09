pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("missing configuration: {0}")]
    Config(String),

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error("{context}: {source}")]
    Request {
        context: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("analyze failed: {0}")]
    AnalyzeFailed(String),

    #[error("operation timed out after {attempts} attempts")]
    PollTimeout { attempts: u32 },

    #[error("missing response header: {0}")]
    MissingHeader(String),

    #[error("provider error: {0}")]
    Provider(#[from] providers::error::Error),

    #[error("OCR service error: {0}")]
    Service(String),

    #[error("invalid or missing OCR result")]
    InvalidResult,
}

impl Error {
    pub fn request(context: impl Into<String>, source: reqwest::Error) -> Self {
        Self::Request {
            context: context.into(),
            source,
        }
    }
}
