pub type Result<T> = core::result::Result<T, ServiceError>;

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error(transparent)]
    Pipeline(#[from] vendrtk_pipelines::error::Error),

    #[error(transparent)]
    Azure(#[from] vendrtk_azure::error::Error),

    #[error(transparent)]
    Storage(#[from] vendrtk_storage::error::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
