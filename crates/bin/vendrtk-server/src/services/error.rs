pub type Result<T> = core::result::Result<T, ServiceError>;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Storage(#[from] vendrtk_storage::error::Error),

    #[error(transparent)]
    Ocr(#[from] vendrtk_ocr::error::Error),

    #[error(transparent)]
    Parser(#[from] vendrtk_parsers::error::Error),

    #[error(transparent)]
    Azure(#[from] vendrtk_azure::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
