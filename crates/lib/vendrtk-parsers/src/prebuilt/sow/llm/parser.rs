use std::future::Future;

use crate::error::{Error, Result};
use crate::models::sow::ParsedSoWs;
use crate::prebuilt::sow::llm::schemas::ExtractedSow;
use crate::traits::llm_client::LLMClient;
use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;

const EXTRACTION_PREAMBLE: &str = "Extract structured statement-of-work data from the OCR text. \
Return agreement header fields and one row per billable rate in the rate schedule. \
Put agreement-level dates in the header; do not repeat them on each rate line.";

pub struct LLMSoWParser;

impl LLMSoWParser {
    pub const fn new() -> Self {
        Self
    }

    pub fn parse<C, O>(
        &self,
        client: &C,
        ocr_result: O,
    ) -> impl Future<Output = Result<ParsedSoWs>> + Send
    where
        C: LLMClient + Send + Sync,
        O: OcrProcessedDocument + Send,
    {
        async move {
            let key = ocr_result.key().to_string();
            let content = ocr_result
                .raw_content()
                .map_err(|e| Error::Parsing(e.to_string()))?;

            let extracted: ExtractedSow = client
                .extract(EXTRACTION_PREAMBLE, &content)
                .await?;

            Ok(extracted.into_parsed_sows(key))
        }
    }
}

pub use LLMSoWParser as SampleSoWParser;
