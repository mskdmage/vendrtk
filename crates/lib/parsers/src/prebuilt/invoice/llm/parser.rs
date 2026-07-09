use std::future::Future;

use crate::error::{Error, Result};
use crate::models::invoice::ParsedInvoices;
use crate::prebuilt::invoice::schemas::ExtractedInvoiceList;
use crate::traits::LLMClient;
use ocr::traits::OcrProcessedDocument;

const EXTRACTION_PREAMBLE: &str = "Extract structured invoice data from the OCR text. \
Return one invoice per document unless the text clearly contains multiple distinct invoices. \
Use header fields for invoice-level data; put each billable row in details.";

pub struct LLMInvoiceParser;

impl LLMInvoiceParser {
    pub const fn new() -> Self {
        Self
    }

    pub fn parse<C, O>(
        &self,
        client: &C,
        ocr_result: O,
    ) -> impl Future<Output = Result<ParsedInvoices>> + Send
    where
        C: LLMClient + Send + Sync,
        O: OcrProcessedDocument + Send,
    {
        async move {
            let key = ocr_result.key().to_string();
            let content = ocr_result
                .raw_content()
                .map_err(|_e| Error::EmptyOcrContent)?;
            if content.trim().is_empty() {
                return Err(Error::EmptyOcrContent);
            }

            let extracted: ExtractedInvoiceList =
                client.extract(EXTRACTION_PREAMBLE, &content).await?;

            Ok(extracted.into_parsed_invoices(key))
        }
    }
}

pub use LLMInvoiceParser as SampleInvoiceParser;
