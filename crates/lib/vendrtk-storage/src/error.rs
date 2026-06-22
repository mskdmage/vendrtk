pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Json(serde_json::Error),
    NotFound(String),
    InvalidDocument(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Io(source) => write!(f, "io error: {source}"),
            Self::Json(source) => write!(f, "json error: {source}"),
            Self::NotFound(identifier) => write!(f, "not found: {identifier}"),
            Self::InvalidDocument(path) => write!(f, "invalid document: {path}"),
        }
    }
}

impl std::error::Error for Error {}
