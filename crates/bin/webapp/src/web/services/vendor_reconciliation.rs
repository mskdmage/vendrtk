use std::sync::Mutex;

use vendrtk::ocr::{
    prebuilt::azure::{
        client::DocumentIntelligenceClient,
        models::AnalyzeOperationResponse,
    },
    traits::OCRClient,
};
use vendrtk::parsers::{
    models::{
        doc_type::ParsedDocumentType,
        invoice::ParsedInvoices,
    },
    prebuilt::{
        classification::llm::parser::LLMDocumentClassifier,
        clients::azure::{self, FoundryClient},
        invoice::llm::parser::LLMInvoiceParser,
    },
};
use vendrtk::storage::{
    models::FileRef,
    prebuilt::local::{LocalOcrProcessedStore, LocalParsedStore, LocalRepository},
    traits::{Repository, Store},
};

use crate::web::error::{Error, Result};

pub struct ProcessedUpload {
    pub file_ref: FileRef,
    pub document_type: ParsedDocumentType,
    pub invoice: Option<ParsedInvoices>,
}

pub struct VendorReconciliationService {
    pub file_repository: LocalRepository,
    pub ocr_client: DocumentIntelligenceClient,
    pub llm_client: FoundryClient,
    pub ocr_processed_store: Mutex<LocalOcrProcessedStore<AnalyzeOperationResponse>>,
    pub invoice_parsed_store: Mutex<LocalParsedStore<ParsedInvoices>>,
}

impl VendorReconciliationService {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            file_repository: LocalRepository::new(".landing").map_err(|error| {
                tracing::error!("storage error: {error}");
                Error::InternalServerError
            })?,
            ocr_client: DocumentIntelligenceClient::from_env(None)
                .map_err(vendrtk::ocr::error::Error::from)
                .map_err(|error| {
                    tracing::error!("ocr init error: {error}");
                    Error::InternalServerError
                })?,
            llm_client: azure::connect_from_env().await.map_err(|error| {
                tracing::error!("parser error: {error}");
                Error::InternalServerError
            })?,
            ocr_processed_store: Mutex::new(
                LocalOcrProcessedStore::<AnalyzeOperationResponse>::new(".ocr").map_err(
                    |error| {
                        tracing::error!("storage error: {error}");
                        Error::InternalServerError
                    },
                )?,
            ),
            invoice_parsed_store: Mutex::new(
                LocalParsedStore::<ParsedInvoices>::new(".parsed/invoices").map_err(|error| {
                    tracing::error!("storage error: {error}");
                    Error::InternalServerError
                })?,
            ),
        })
    }

    pub async fn upload_and_process(&self, bytes: &[u8]) -> Result<ProcessedUpload> {
        let file_ref = self.file_repository.put(bytes).map_err(|error| {
            tracing::error!("storage error: {error}");
            Error::InternalServerError
        })?;

        tracing::info!(
            "file registered: key={} kind={:?}",
            file_ref.key,
            file_ref.kind
        );

        self.ensure_ocr(&file_ref, bytes).await?;
        let ocr = self.load_ocr(&file_ref.key)?;

        if let Some(invoice) = self.load_cached_invoice(&file_ref.key)? {
            tracing::info!("invoice parse cache hit: key={}", file_ref.key);
            return Ok(ProcessedUpload {
                file_ref,
                document_type: ParsedDocumentType::Invoice,
                invoice: Some(invoice),
            });
        }

        let document_type = LLMDocumentClassifier::new()
            .classify(&self.llm_client, ocr.clone())
            .await
            .map_err(|error| {
                tracing::error!("parser error: {error}");
                Error::InternalServerError
            })?;

        tracing::info!(
            "document classified: key={} type={:?}",
            file_ref.key,
            document_type
        );

        let invoice = match document_type {
            ParsedDocumentType::Invoice => {
                let parsed = self.parse_and_store_invoice(&file_ref.key, ocr).await?;
                Some(parsed)
            }
            ParsedDocumentType::SoW | ParsedDocumentType::Unknown => None,
        };

        Ok(ProcessedUpload {
            file_ref,
            document_type,
            invoice,
        })
    }

    async fn ensure_ocr(&self, file_ref: &FileRef, bytes: &[u8]) -> Result<()> {
        let already_cached = self
            .ocr_processed_store
            .lock()
            .map_err(|_| Error::InternalServerError)?
            .exists(&file_ref.key)
            .map_err(|error| {
                tracing::error!("storage error: {error}");
                Error::InternalServerError
            })?;

        if already_cached {
            return Ok(());
        }

        let response = self
            .ocr_client
            .analyze_bytes(bytes)
            .await
            .map_err(|error| {
                tracing::error!("ocr error: {error}");
                Error::InternalServerError
            })?;

        match self
            .ocr_processed_store
            .lock()
            .map_err(|_| Error::InternalServerError)?
            .create(&file_ref.key, response)
        {
            Ok(()) => {}
            Err(vendrtk::storage::error::Error::Repository(message))
                if message.contains("key already exists") => {}
            Err(error) => {
                tracing::error!("storage error: {error}");
                return Err(Error::InternalServerError);
            }
        }

        Ok(())
    }

    fn load_ocr(&self, key: &str) -> Result<AnalyzeOperationResponse> {
        self.ocr_processed_store
            .lock()
            .map_err(|_| Error::InternalServerError)?
            .get(key)
            .map_err(|error| {
                tracing::error!("storage error: {error}");
                Error::InternalServerError
            })?
            .ok_or_else(|| {
                tracing::error!("ocr result missing after ensure_ocr: key={key}");
                Error::InternalServerError
            })
    }

    fn load_cached_invoice(&self, key: &str) -> Result<Option<ParsedInvoices>> {
        let store = self
            .invoice_parsed_store
            .lock()
            .map_err(|_| Error::InternalServerError)?;

        if !store.exists(key).map_err(|error| {
            tracing::error!("storage error: {error}");
            Error::InternalServerError
        })? {
            return Ok(None);
        }

        store.get(key).map_err(|error| {
            tracing::error!("storage error: {error}");
            Error::InternalServerError
        })
    }

    async fn parse_and_store_invoice(
        &self,
        key: &str,
        ocr: AnalyzeOperationResponse,
    ) -> Result<ParsedInvoices> {
        let mut parsed = LLMInvoiceParser::new()
            .parse(&self.llm_client, ocr)
            .await
            .map_err(|error| {
                tracing::error!("parser error: {error}");
                Error::InternalServerError
            })?;
        parsed.key = key.to_string();

        self.invoice_parsed_store
            .lock()
            .map_err(|_| Error::InternalServerError)?
            .create(key, parsed.clone())
            .map_err(|error| {
                tracing::error!("storage error: {error}");
                Error::InternalServerError
            })?;

        tracing::info!(
            "invoice parsed and stored: key={} invoice_count={}",
            key,
            parsed.results.len()
        );

        Ok(parsed)
    }
}
