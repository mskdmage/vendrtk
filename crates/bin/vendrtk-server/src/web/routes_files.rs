use axum::{Json, Router, extract::Multipart, routing};
use serde::Serialize;
use tracing::info;

use crate::web::error::{Error, Result};

pub fn routes() -> Router {
    Router::new().route("/upload", routing::post(upload_handler))
}

async fn upload_handler(mut multipart: Multipart) -> Result<Json<UploadFileResponse>> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field.file_name().unwrap_or("unknown").to_string();

        let bytes = field
            .bytes()
            .await
            .map_err(|_| Error::BadRequest("failed to read file".into()))?;

        let size = bytes.len();

        info!(%filename, size, "received upload");

        let _ = bytes;

        return Ok(Json(UploadFileResponse {
            message: "file uploaded successfully".into(),
            filename,
            size,
        }));
    }

    Err(Error::BadRequest("missing file field".into()))
}

#[derive(Serialize)]
struct UploadFileResponse {
    message: String,
    filename: String,
    size: usize,
}
