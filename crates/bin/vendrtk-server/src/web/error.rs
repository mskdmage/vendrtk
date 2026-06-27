use axum::{http::StatusCode, response::IntoResponse};
use vendrtk_azure::error::Error as AzureError;
use vendrtk_ocr::error::Error as OcrError;
use vendrtk_parsers::error::Error as ParserError;
use vendrtk_pipelines::error::Error as PipelineError;
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
            ServiceError::Pipeline(pipeline_err) if is_bad_request(pipeline_err) => {
                Self::BadRequest(err.to_string())
            }
            ServiceError::Pipeline(PipelineError::Storage(StorageError::NotFound(key))) => {
                Self::NotFound(key.clone())
            }
            ServiceError::Azure(AzureError::Config(_)) => Self::BadRequest(err.to_string()),
            _ => {
                tracing::error!(error = ?err, "pipeline failed");
                Self::InternalServer
            }
        }
    }
}

fn is_bad_request(err: &PipelineError) -> bool {
    matches!(
        err,
        PipelineError::Storage(StorageError::InvalidExtension { .. })
            | PipelineError::Storage(StorageError::InvalidMagicBytes)
            | PipelineError::Parser(ParserError::EmptyOcrContent)
            | PipelineError::Ocr(OcrError::InvalidResult)
            | PipelineError::UnknownDocumentType
            | PipelineError::ConflictingParsedCache
    )
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
