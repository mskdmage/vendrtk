use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, State},
    routing,
};
use serde::{Deserialize, Serialize};

use crate::config::config;
use crate::state::AppState;
use crate::web::error::{Error, Result};
use vendrtk::parsers::models::{doc_type::ParsedDocumentType, invoice::ParsedInvoices};
use vendrtk::storage::models::FileRef;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/upload", routing::post(upload_handler))
        .layer(DefaultBodyLimit::max(config().max_upload_bytes))
        .with_state(state)
}

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    filename: String,
    size: usize,
    file: FileRef,
    document_type: ParsedDocumentType,
    invoice: Option<ParsedInvoices>,
}

async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field.file_name().unwrap_or("unknown").to_string();
        let bytes = field.bytes().await.map_err(map_multipart_error)?;

        let processed = state
            .vendor_reconciliation_service
            .upload_and_process(&bytes)
            .await?;

        tracing::info!(
            "upload complete: filename={} size={} key={} kind={:?} document_type={:?}",
            filename,
            bytes.len(),
            processed.file_ref.key,
            processed.file_ref.kind,
            processed.document_type
        );

        return Ok(Json(UploadResponse {
            filename,
            size: bytes.len(),
            file: processed.file_ref,
            document_type: processed.document_type,
            invoice: processed.invoice,
        }));
    }

    tracing::warn!("upload request missing file field");
    Err(Error::BadRequest)
}

fn map_multipart_error(error: axum::extract::multipart::MultipartError) -> Error {
    tracing::error!("failed to read upload field: {}", error);

    let message = error.to_string().to_lowercase();
    if message.contains("limit") || message.contains("too large") || message.contains("length") {
        Error::PayloadTooLarge
    } else {
        Error::BadRequest
    }
}
