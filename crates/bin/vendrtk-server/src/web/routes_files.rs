use axum::{
    Json, Router,
    extract::{State, Multipart},
    routing,
};
use tracing::info;
use vendrtk_core::parsers::models::{ParsedInvoices, ParsedSoWs};

use std::sync::Arc;
use crate::state::AppState;
use crate::web::error::{Error, Result};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/upload/invoice", routing::post(upload_invoice_handler))
        .route("/upload/sow", routing::post(upload_sow_handler))
        .with_state(state)
}

async fn upload_invoice_handler(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<Json<ParsedInvoices>> {
    let (filename, bytes) = read_upload_file(multipart).await?;
    info!(%filename, size = bytes.len(), document_type = "invoice", "received upload");

    let parsed = state
        .vendor_reconciliation_service
        .lock()
        .await
        .save_pdf(&filename, &bytes)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok(Json(parsed))
}

async fn upload_sow_handler(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<Json<ParsedSoWs>> {
    let (filename, bytes) = read_upload_file(multipart).await?;
    info!(%filename, size = bytes.len(), document_type = "sow", "received upload");

    let parsed = state
        .vendor_reconciliation_service
        .lock()
        .await
        .save_sow_pdf(&filename, &bytes)
        .await
        .map_err(|e| Error::BadRequest(e.to_string()))?;

    Ok(Json(parsed))
}

async fn read_upload_file(mut multipart: Multipart) -> Result<(String, Vec<u8>)> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field.file_name().unwrap_or("unknown").to_string();
        let bytes = field
            .bytes()
            .await
            .map_err(|_| Error::BadRequest("failed to read file".into()))?;

        return Ok((filename, bytes.into()));
    }

    Err(Error::BadRequest("missing file field".into()))
}
