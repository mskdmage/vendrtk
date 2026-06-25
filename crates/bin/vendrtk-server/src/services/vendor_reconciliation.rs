use std::path::Path;
use std::sync::Arc;

use tokio::sync::OnceCell;

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
    ocr_client: OnceCell<DocumentIntelligenceClient>,
    llm_client: OnceCell<FoundryClient>,
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
        Ok(Self {
            landing_dir: landing_dir.to_string(),
            landing_store: LocalDocumentStore::new(landing_dir)?,
            processed_store: LocalOcrProcessedStore::new(ocr_dir)?,
            parsed_invoice_store: LocalParsedStore::new(parsed_invoices_dir)?,
            parsed_sow_store: LocalParsedStore::new(parsed_sows_dir)?,
            ocr_client: OnceCell::new(),
            llm_client: OnceCell::new(),
            invoice_parser: SampleInvoiceParser::new(),
            sow_parser: SampleSoWParser::new(),
        })
    }

    async fn ocr_client(&self) -> Result<&DocumentIntelligenceClient> {
        if let Some(client) = self.ocr_client.get() {
            return Ok(client);
        }

        let cfg = config();
        let auth = if cfg.azure_cognitive_services_key.is_empty() {
            Auth::Credential(Arc::new(Credential::new(None, None, None)?))
        } else {
            Auth::ApiKey(cfg.azure_cognitive_services_key.clone())
        };
        let client = DocumentIntelligenceClient::new(
            cfg.azure_cognitive_services_endpoint.clone(),
            ApiVersion::Default,
            auth,
            Config::default(),
        )?;

        let _ = self.ocr_client.set(client);
        Ok(self.ocr_client.get().expect("ocr client just initialized"))
    }

    async fn llm_client(&self) -> Result<&FoundryClient> {
        if let Some(client) = self.llm_client.get() {
            return Ok(client);
        }

        let cfg = config();
        let client = FoundryClient::connect(
            &cfg.azure_openai_endpoint,
            &cfg.azure_openai_api_version,
            cfg.azure_openai_deployment.clone(),
        )
        .await?;

        let _ = self.llm_client.set(client);
        Ok(self.llm_client.get().expect("llm client just initialized"))
    }

    pub async fn save_pdf(&mut self, filename: &str, bytes: &[u8]) -> Result<ParsedInvoices> {
        let (doc, ocr_doc) = self.stage_pdf(filename, bytes).await?;

        let parsed_invoices = match self.parsed_invoice_store.load_payload(&doc.key)? {
            Some(cached) => cached,
            None => {
                let llm = self.llm_client().await?;
                let parsed = self
                    .invoice_parser
                    .parse(llm, ocr_doc.clone())
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
                let llm = self.llm_client().await?;
                let parsed = self
                    .sow_parser
                    .parse(llm, ocr_doc.clone())
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
                let ocr = self.ocr_client().await?;
                let response = ocr.analyze_bytes(bytes).await?;

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
