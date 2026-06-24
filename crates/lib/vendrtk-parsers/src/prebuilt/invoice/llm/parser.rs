use std::future::Future;

use crate::error::{Error, Result};
use crate::models::invoice::ParsedInvoices;
use crate::prebuilt::invoice::llm::client::azure_openai_client;
use crate::prebuilt::invoice::llm::schemas::ExtractedInvoiceList;
use crate::traits::parser::Parser;
use rig::client::CompletionClient;
use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;

const EXTRACTION_PREAMBLE: &str = "Extract structured invoice data from the OCR text. \
Return one invoice per document unless the text clearly contains multiple distinct invoices. \
Use header fields for invoice-level data; put each billable row in details.";

pub struct LLMInvoiceParser {
    endpoint: String,
    deployment: String,
    api_version: String,
}

impl LLMInvoiceParser {
    pub fn new(
        endpoint: impl Into<String>,
        deployment: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            deployment: deployment.into(),
            api_version: api_version.into(),
        }
    }
}

impl<O> Parser<O> for LLMInvoiceParser
where
    O: OcrProcessedDocument + Send,
{
    type Output = ParsedInvoices;

    fn parse(
        &self,
        ocr_result: Option<O>,
        _bytes: Option<&[u8]>,
    ) -> impl Future<Output = Result<ParsedInvoices>> + Send {
        let endpoint = self.endpoint.clone();
        let deployment = self.deployment.clone();
        let api_version = self.api_version.clone();

        async move {
            let ocr = ocr_result
                .ok_or_else(|| Error::Parsing("OCR result is required".into()))?;

            let key = ocr.key().to_string();
            let content = ocr
                .raw_content()
                .map_err(|e| Error::Parsing(e.to_string()))?;

            let client = azure_openai_client(&endpoint, &api_version).await?;
            let extractor = client
                .extractor::<ExtractedInvoiceList>(deployment)
                .preamble(EXTRACTION_PREAMBLE)
                .build();

            let extracted = extractor
                .extract(content)
                .await
                .map_err(|e| Error::Parsing(e.to_string()))?;

            Ok(extracted.into_parsed_invoices(key))
        }
    }
}

pub use LLMInvoiceParser as SampleInvoiceParser;
