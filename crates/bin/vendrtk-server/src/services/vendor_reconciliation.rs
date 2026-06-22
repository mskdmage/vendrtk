use std::path::Path;

use vendrtk_core::storage::local::{
    LocalDocumentStore, LocalOcrProcessedStore, LocalParsedInvoiceStore, LocalParsedSoWStore,
};
use vendrtk_core::storage::models::{
    pdf_from_bytes, DocumentIntelligenceOcrProcessedDocument, PdfDocument,
};

pub struct VendorReconciliationService {
    landing_dir: String,
    landing_store: LocalDocumentStore<PdfDocument>,
    processed_store: LocalOcrProcessedStore<DocumentIntelligenceOcrProcessedDocument>,
    parsed_invoice_store: LocalParsedInvoiceStore,
    parsed_sow_store: LocalParsedSoWStore,
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
        })
    }

    pub fn save_pdf(
        &mut self,
        filename: &str,
        bytes: &[u8],
    ) -> std::io::Result<PdfDocument> {
        let path = Path::new(&self.landing_dir).join(filename);
        std::fs::write(&path, bytes)?;
        self.landing_store
            .register(filename)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        pdf_from_bytes(&path, bytes).map_err(|e| std::io::Error::other(e.to_string()))
    }
}
