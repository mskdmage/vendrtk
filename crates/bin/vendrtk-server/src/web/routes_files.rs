use axum::{
    Json, Router,
    extract::{Multipart, State},
    routing,
};
use serde::Serialize;
use tracing::info;
use vendrtk_parsers::models::invoice::ParsedInvoices;
use vendrtk_parsers::models::sow::ParsedSoWs;
use vendrtk_pipelines::prebuilt::vendor_reconciliation::input::VendorReconciliationInput;
use vendrtk_pipelines::prebuilt::vendor_reconciliation::output::VendorReconciliationOutput;

use std::sync::Arc;

use crate::state::AppState;
use crate::web::error::{Error, Result};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/upload", routing::post(upload_handler))
        .with_state(state)
}

#[derive(Debug, Serialize)]
#[serde(tag = "document_type", content = "parsed", rename_all = "snake_case")]
enum UploadResponse {
    Invoice(ParsedInvoices),
    Sow(ParsedSoWs),
}

impl From<VendorReconciliationOutput> for UploadResponse {
    fn from(output: VendorReconciliationOutput) -> Self {
        match output {
            VendorReconciliationOutput::Invoice(parsed) => Self::Invoice(parsed),
            VendorReconciliationOutput::Sow(parsed) => Self::Sow(parsed),
        }
    }
}

async fn upload_handler(
    State(state): State<Arc<AppState>>,
    multipart: Multipart,
) -> Result<Json<UploadResponse>> {
    let (filename, bytes) = read_upload_file(multipart).await?;
    info!(%filename, size = bytes.len(), "received upload");

    let output = state
        .vendor_reconciliation_service
        .lock()
        .await
        .run(VendorReconciliationInput { filename, bytes })
        .await?;

    Ok(Json(output.into()))
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
