use axum::{http::StatusCode, response::IntoResponse};
use vendrtk_storage::error::Error as StorageError;

use crate::services::error::ServiceError;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    NotFound(String),

    #[error("internal server error")]
    InternalServer,
}

impl From<ServiceError> for Error {
    fn from(err: ServiceError) -> Self {
        match &err {
            ServiceError::Storage(StorageError::InvalidExtension { .. })
            | ServiceError::Storage(StorageError::InvalidMagicBytes)
            | ServiceError::Parser(vendrtk_parsers::error::Error::EmptyOcrContent)
            | ServiceError::Ocr(vendrtk_ocr::error::Error::InvalidResult) => {
                Self::BadRequest(err.to_string())
            }
            ServiceError::Storage(StorageError::NotFound(key)) => Self::NotFound(key.clone()),
            _ => {
                tracing::error!(error = %err, "pipeline failed");
                Self::InternalServer
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            Self::InternalServer => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_SERVER_ERROR".to_string(),
            ),
        };

        (status, message).into_response()
    }
}
