use std::path::Path;
use std::sync::Arc;

use crate::config::config;
use vendrtk_core::ocr::azure::{ApiVersion, Auth, Config, Credential, DocumentIntelligenceClient};
use vendrtk_core::ocr::traits::OCRClient;
use vendrtk_core::parsers::models::{ParsedInvoices, ParsedSoWs};
use vendrtk_core::parsers::prebuilt::invoice::SampleInvoiceParser;
use vendrtk_core::parsers::traits::Parser;
use vendrtk_core::storage::local::{
    LocalDocumentStore, LocalOcrProcessedStore, LocalParsedStore,
};
use vendrtk_core::storage::models::{
    pdf_from_bytes, DocumentIntelligenceOcrProcessedDocument, PdfDocument,
};
use vendrtk_core::storage::traits::ProcessedDocumentStore;

pub struct VendorReconciliationService {
    landing_dir: String,
    landing_store: LocalDocumentStore<PdfDocument>,
    processed_store: LocalOcrProcessedStore<DocumentIntelligenceOcrProcessedDocument>,
    parsed_invoice_store: LocalParsedStore<ParsedInvoices>,
    parsed_sow_store: LocalParsedStore<ParsedSoWs>,
    ocr_client: DocumentIntelligenceClient,
    invoice_parser: SampleInvoiceParser,
}

impl VendorReconciliationService {
    pub fn new(
        landing_dir: &str,
        ocr_dir: &str,
        parsed_invoices_dir: &str,
        parsed_sows_dir: &str,
    ) -> std::io::Result<Self> {
        let cfg = config();

        Ok(Self {
            landing_dir: landing_dir.to_string(),
            landing_store: LocalDocumentStore::new(landing_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            processed_store: LocalOcrProcessedStore::new(ocr_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            parsed_invoice_store: LocalParsedStore::new(parsed_invoices_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            parsed_sow_store: LocalParsedStore::new(parsed_sows_dir)
                .map_err(|e| std::io::Error::other(e.to_string()))?,
            ocr_client: DocumentIntelligenceClient::new(
                cfg.azure_cognitive_services_endpoint.clone(),
                ApiVersion::Default,
                Auth::Credential(Arc::new(Credential::new(None, None, None).unwrap())),
                Config::default(),
            ),
            invoice_parser: SampleInvoiceParser::new(
                cfg.azure_openai_endpoint.clone(),
                cfg.azure_openai_deployment.clone(),
                cfg.azure_openai_api_version.clone(),
            ),
        })
    }

    pub async fn save_pdf(
        &mut self,
        filename: &str,
        bytes: &[u8],
    ) -> std::io::Result<ParsedInvoices> {
        let path = Path::new(&self.landing_dir).join(filename);
        std::fs::write(&path, bytes)?;
        self.landing_store
            .register(filename)
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let doc = pdf_from_bytes(&path, bytes).map_err(|e| std::io::Error::other(e.to_string()))?;

        let ocr_doc = match self
            .processed_store
            .load_payload(&doc.key)
            .map_err(|e| std::io::Error::other(e.to_string()))?
        {
            Some(cached) => cached,
            None => {
                let response = self
                    .ocr_client
                    .analyze_bytes(bytes)
                    .await
                    .map_err(|e| std::io::Error::other(e.to_string()))?;

                let ocr_doc = DocumentIntelligenceOcrProcessedDocument {
                    key: doc.key.clone(),
                    analyze_operation_response: response,
                };

                self.processed_store
                    .save(ocr_doc.clone())
                    .map_err(|e| std::io::Error::other(e.to_string()))?;

                ocr_doc
            }
        };

        let parsed_invoices = match self
            .parsed_invoice_store
            .load_payload(&doc.key)
            .map_err(|e| std::io::Error::other(e.to_string()))?
        {
            Some(cached) => cached,
            None => {
                let parsed = self
                    .invoice_parser
                    .parse(Some(ocr_doc.clone()), Some(bytes))
                    .await
                    .map_err(|e| std::io::Error::other(e.to_string()))?;

                self.parsed_invoice_store
                    .save(parsed.clone())
                    .map_err(|e| std::io::Error::other(e.to_string()))?;

                parsed
            }
        };

        Ok(parsed_invoices)
    }
}
