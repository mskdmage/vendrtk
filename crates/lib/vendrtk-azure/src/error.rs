pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing configuration: {0}")]
    Config(String),

    #[error("authentication failed: {0}")]
    Auth(String),

    #[error(transparent)]
    Request(#[from] reqwest::Error),

    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },

    #[error("analyze operation failed: {0}")]
    AnalyzeFailed(String),

    #[error("operation timed out after {attempts} attempts")]
    PollTimeout { attempts: u32 },

    #[error("missing response header: {0}")]
    MissingHeader(String),

    #[error("client setup failed: {0}")]
    Client(String),
}
