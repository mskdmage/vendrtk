use axum::{
    Json, Router,
    extract::{State, Multipart},
    routing,
};
use serde::Serialize;
use tracing::info;

use std::sync::Arc;
use crate::state::AppState;
use crate::web::error::{Error, Result};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new().route("/upload", routing::post(upload_handler))
    .with_state(state)
}

async fn upload_handler(
    State(state): State<std::sync::Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadFileResponse>> {
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

        let doc = state
            .vendor_reconciliation_service
            .lock()
            .await
            .save_pdf(&filename, &bytes)
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        return Ok(Json(UploadFileResponse {
            message: "file uploaded successfully".into(),
            filename,
            size,
            key: doc.key,
        }));
    }

    Err(Error::BadRequest("missing file field".into()))
}

#[derive(Serialize)]
struct UploadFileResponse {
    message: String,
    filename: String,
    size: usize,
    key: String,
}
