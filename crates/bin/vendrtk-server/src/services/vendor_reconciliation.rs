use std::path::Path;
use std::sync::Arc;

use crate::config::config;
use crate::services::error::Result;
use vendrtk_core::azure::foundry::FoundryClient;
use vendrtk_core::ocr::azure::{ApiVersion, Auth, Config, Credential, DocumentIntelligenceClient};
use vendrtk_core::ocr::traits::OCRClient;
use vendrtk_core::parsers::models::{ParsedInvoices, ParsedSoWs};
use vendrtk_core::parsers::prebuilt::invoice::SampleInvoiceParser;
use vendrtk_core::parsers::prebuilt::sow::SampleSoWParser;
use vendrtk_core::storage::local::{LocalDocumentStore, LocalOcrProcessedStore, LocalParsedStore};
use vendrtk_core::storage::models::{
    DocumentIntelligenceOcrProcessedDocument, PdfDocument, pdf_from_bytes,
};
use vendrtk_core::storage::traits::ProcessedDocumentStore;

pub struct VendorReconciliationService {
    landing_dir: String,
    landing_store: LocalDocumentStore<PdfDocument>,
    processed_store: LocalOcrProcessedStore<DocumentIntelligenceOcrProcessedDocument>,
    parsed_invoice_store: LocalParsedStore<ParsedInvoices>,
    parsed_sow_store: LocalParsedStore<ParsedSoWs>,
    ocr_client: DocumentIntelligenceClient,
    llm_client: FoundryClient,
    invoice_parser: SampleInvoiceParser,
    sow_parser: SampleSoWParser,
}

impl VendorReconciliationService {
    pub async fn new(
        landing_dir: &str,
        ocr_dir: &str,
        parsed_invoices_dir: &str,
        parsed_sows_dir: &str,
    ) -> Result<Self> {
        let cfg = config();

        let llm_client = FoundryClient::connect(
            &cfg.azure_openai_endpoint,
            &cfg.azure_openai_api_version,
            cfg.azure_openai_deployment.clone(),
        )
        .await?;

        Ok(Self {
            landing_dir: landing_dir.to_string(),
            landing_store: LocalDocumentStore::new(landing_dir)?,
            processed_store: LocalOcrProcessedStore::new(ocr_dir)?,
            parsed_invoice_store: LocalParsedStore::new(parsed_invoices_dir)?,
            parsed_sow_store: LocalParsedStore::new(parsed_sows_dir)?,
            ocr_client: DocumentIntelligenceClient::new(
                cfg.azure_cognitive_services_endpoint.clone(),
                ApiVersion::Default,
                Auth::Credential(Arc::new(Credential::new(None, None, None)?)),
                Config::default(),
            ),
            llm_client,
            invoice_parser: SampleInvoiceParser::new(),
            sow_parser: SampleSoWParser::new(),
        })
    }

    pub async fn save_pdf(&mut self, filename: &str, bytes: &[u8]) -> Result<ParsedInvoices> {
        let (doc, ocr_doc) = self.stage_pdf(filename, bytes).await?;

        let parsed_invoices = match self.parsed_invoice_store.load_payload(&doc.key)? {
            Some(cached) => cached,
            None => {
                let parsed = self
                    .invoice_parser
                    .parse(&self.llm_client, ocr_doc.clone())
                    .await?;

                self.parsed_invoice_store.save(parsed.clone())?;

                parsed
            }
        };

        Ok(parsed_invoices)
    }

    pub async fn save_sow_pdf(&mut self, filename: &str, bytes: &[u8]) -> Result<ParsedSoWs> {
        let (doc, ocr_doc) = self.stage_pdf(filename, bytes).await?;

        let parsed_sows = match self.parsed_sow_store.load_payload(&doc.key)? {
            Some(cached) => cached,
            None => {
                let parsed = self
                    .sow_parser
                    .parse(&self.llm_client, ocr_doc.clone())
                    .await?;

                self.parsed_sow_store.save(parsed.clone())?;

                parsed
            }
        };

        Ok(parsed_sows)
    }

    async fn stage_pdf(
        &mut self,
        filename: &str,
        bytes: &[u8],
    ) -> Result<(PdfDocument, DocumentIntelligenceOcrProcessedDocument)> {
        let path = Path::new(&self.landing_dir).join(filename);
        std::fs::write(&path, bytes)?;
        self.landing_store.register(filename)?;

        let doc = pdf_from_bytes(&path, bytes)?;

        let ocr_doc = match self.processed_store.load_payload(&doc.key)? {
            Some(cached) => cached,
            None => {
                let response = self.ocr_client.analyze_bytes(bytes).await?;

                let ocr_doc = DocumentIntelligenceOcrProcessedDocument {
                    key: doc.key.clone(),
                    analyze_operation_response: response,
                };

                self.processed_store.save(ocr_doc.clone())?;

                ocr_doc
            }
        };

        Ok((doc, ocr_doc))
    }
}
