use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("internal server error")]
    InternalServerError,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Error::InternalServerError => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal Server Error").into_response()
            },
        }
    }
}