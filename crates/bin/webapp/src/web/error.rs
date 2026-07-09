use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bad request")]
    BadRequest,

    #[error("payload too large")]
    PayloadTooLarge,

    #[error("internal server error")]
    InternalServerError,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            Error::BadRequest => StatusCode::BAD_REQUEST,
            Error::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Error::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
        };

        tracing::error!("request failed: status={} error={}", status, self);

        (status, self.to_string()).into_response()
    }
}
