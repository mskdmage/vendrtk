use axum::{
    Json, Router,
    extract::{State, Multipart},
    routing,
};
use tracing::info;
use vendrtk_core::parsers::models::ParsedInvoices;

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
) -> Result<Json<ParsedInvoices>> {
    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() != Some("file") {
            continue;
        }

        let filename = field.file_name().unwrap_or("unknown").to_string();

        let bytes = field
            .bytes()
            .await
            .map_err(|_| Error::BadRequest("failed to read file".into()))?;

        info!(%filename, size = bytes.len(), "received upload");

        let parsed_invoices = state
            .vendor_reconciliation_service
            .lock()
            .await
            .save_pdf(&filename, &bytes)
            .await
            .map_err(|e| Error::BadRequest(e.to_string()))?;

        return Ok(Json(parsed_invoices));
    }

    Err(Error::BadRequest("missing file field".into()))
}
