use std::sync::{Arc, Mutex};

use vendrtk::ocr::prebuilt::azure::{
    client::DocumentIntelligenceClient, models::AnalyzeOperationResponse,
};
use vendrtk::parsers::{
    models::{doc_type::ParsedDocumentType, invoice::ParsedInvoices},
    prebuilt::clients::azure::{self, FoundryClient},
};
use vendrtk::pipelines::prebuilt::vendor_reconciliation::{
    Done, VendorReconciliationContext, VendorReconciliationPipeline,
};
use vendrtk::pipelines::traits::Pipeline;
use vendrtk::storage::prebuilt::local::{
    LocalOcrProcessedStore, LocalParsedStore, LocalRepository,
};

use crate::web::error::{Error, Result};

pub struct ProcessedUpload {
    pub file_ref: vendrtk::storage::models::FileRef,
    pub document_type: ParsedDocumentType,
    pub invoice: Option<ParsedInvoices>,
}

type VrPipeline = VendorReconciliationPipeline<
    AnalyzeOperationResponse,
    DocumentIntelligenceClient,
    LocalOcrProcessedStore<AnalyzeOperationResponse>,
    FoundryClient,
    LocalParsedStore<ParsedInvoices>,
>;

pub struct VendorReconciliationService {
    pipeline: VrPipeline,
}

impl VendorReconciliationService {
    pub async fn new() -> Result<Self> {
        let landing = LocalRepository::new(".landing").map_err(|error| {
            tracing::error!("storage error: {error}");
            Error::InternalServerError
        })?;

        let ocr_client = DocumentIntelligenceClient::from_env(None)
            .map_err(vendrtk::ocr::error::Error::from)
            .map_err(|error| {
                tracing::error!("ocr init error: {error}");
                Error::InternalServerError
            })?;

        let llm_client = azure::connect_from_env().await.map_err(|error| {
            tracing::error!("parser error: {error}");
            Error::InternalServerError
        })?;

        let ocr_store =
            LocalOcrProcessedStore::<AnalyzeOperationResponse>::new(".ocr").map_err(|error| {
                tracing::error!("storage error: {error}");
                Error::InternalServerError
            })?;

        let parsed_store =
            LocalParsedStore::<ParsedInvoices>::new(".parsed/invoices").map_err(|error| {
                tracing::error!("storage error: {error}");
                Error::InternalServerError
            })?;

        let ctx = VendorReconciliationContext::new(
            Arc::new(landing),
            Arc::new(ocr_client),
            Arc::new(Mutex::new(ocr_store)),
            Arc::new(llm_client),
            Arc::new(Mutex::new(parsed_store)),
        );

        Ok(Self {
            pipeline: Pipeline::new(ctx),
        })
    }

    pub async fn upload_and_process(&self, bytes: &[u8]) -> Result<ProcessedUpload> {
        let Done {
            file_ref,
            document_type,
            invoice,
        } = self.pipeline.run(bytes.to_vec()).await.map_err(|error| {
            tracing::error!("pipeline error: {error}");
            Error::InternalServerError
        })?;

        tracing::info!(
            "file processed: key={} kind={:?} document_type={:?}",
            file_ref.key,
            file_ref.kind,
            document_type
        );

        Ok(ProcessedUpload {
            file_ref,
            document_type,
            invoice,
        })
    }
}
