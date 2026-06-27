pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error(transparent)]
    Storage(#[from] vendrtk_storage::error::Error),

    #[error(transparent)]
    Ocr(#[from] vendrtk_ocr::error::Error),

    #[error(transparent)]
    Parser(#[from] vendrtk_parsers::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("could not classify document type")]
    UnknownDocumentType,

    #[error("document has conflicting parsed cache entries")]
    ConflictingParsedCache,
}
