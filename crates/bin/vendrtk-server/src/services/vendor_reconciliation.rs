use std::path::Path;
use std::sync::Arc;

use vendrtk_core::storage::local::{
    LocalDocumentStore, LocalOcrProcessedStore, LocalParsedInvoiceStore, LocalParsedSoWStore,
};
use vendrtk_core::storage::models::{
    pdf_from_bytes, DocumentIntelligenceOcrProcessedDocument, PdfDocument,
};

use vendrtk_core::ocr::azure::{ApiVersion, Auth, Config, Credential, DocumentIntelligenceClient};
use vendrtk_core::ocr::traits::OCRClient;
use vendrtk_core::storage::traits::ProcessedDocumentStore;

pub struct VendorReconciliationService {
    landing_dir: String,
    landing_store: LocalDocumentStore<PdfDocument>,
    processed_store: LocalOcrProcessedStore<DocumentIntelligenceOcrProcessedDocument>,
    parsed_invoice_store: LocalParsedInvoiceStore,
    parsed_sow_store: LocalParsedSoWStore,
    ocr_client: DocumentIntelligenceClient,
}

impl VendorReconciliationService {
    pub fn new(
        landing_dir: &str,
        ocr_dir: &str,
        parsed_invoices_dir: &str,
        parsed_sows_dir: &str,
    ) -> std::io::Result<Self> {
        Ok(Self {
            landing_dir: landing_dir.to_string(),
            landing_store: LocalDocumentStore::new(landing_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            processed_store: LocalOcrProcessedStore::new(ocr_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            parsed_invoice_store: LocalParsedInvoiceStore::new(parsed_invoices_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            parsed_sow_store: LocalParsedSoWStore::new(parsed_sows_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            ocr_client: DocumentIntelligenceClient::new(
                std::env::var("AZURE_COGNITIVE_SERVICES_ENDPOINT").unwrap(),
                ApiVersion::Default,
                Auth::Credential(Arc::new(Credential::new(None, None, None).unwrap())),
                Config::default(),
            ),
        })
    }

    pub async fn save_pdf(
        &mut self,
        filename: &str,
        bytes: &[u8],
    ) -> std::io::Result<PdfDocument> {
        let path = Path::new(&self.landing_dir).join(filename);
        std::fs::write(&path, bytes)?;
        self.landing_store
            .register(filename)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let doc = pdf_from_bytes(&path, bytes).map_err(|e| std::io::Error::other(e.to_string()))?;

        match self.ocr_client.analyze_bytes(bytes).await {
            Ok(response) => {
                let ocr_doc = DocumentIntelligenceOcrProcessedDocument {
                    key: doc.key.clone(),
                    analyze_operation_response: response,
                };
                if let Err(e) = self.processed_store.save(ocr_doc) {
                    tracing::warn!(error = %e, key = %doc.key, "failed to save OCR result");
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, key = %doc.key, "OCR analysis failed");
            }
        }

        Ok(doc)
    }
}
