pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Parsing(String),
    Client(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Parsing(source) => write!(f, "parsing error: {source}"),
            Self::Client(source) => write!(f, "client error: {source}"),
        }
    }
}

impl std::error::Error for Error {}
