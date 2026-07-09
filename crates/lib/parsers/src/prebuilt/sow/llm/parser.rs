use crate::error::{Error, Result};
use crate::models::sow::ParsedSoWs;
use crate::prebuilt::sow::schemas::ExtractedSow;
use crate::traits::LLMClient;
use ocr::traits::OcrProcessedDocument;

const EXTRACTION_PREAMBLE: &str = "Extract structured statement-of-work data from the OCR text. \
Return agreement header fields and one row per billable rate in the rate schedule. \
Put agreement-level dates in the header; do not repeat them on each rate line.";

pub struct LLMSoWParser;

impl LLMSoWParser {
    pub async fn parse<C, O>(&self, client: &C, ocr_result: O) -> Result<ParsedSoWs>
    where
        C: LLMClient + Send + Sync,
        O: OcrProcessedDocument + Send,
    {
        let key = ocr_result.key().to_string();
        let content = ocr_result
            .raw_content()
            .map_err(|_e| Error::EmptyOcrContent)?;
        if content.trim().is_empty() {
            return Err(Error::EmptyOcrContent);
        }

        let extracted: ExtractedSow = client.extract(EXTRACTION_PREAMBLE, &content).await?;

        Ok(extracted.into_parsed_sows(key))
    }
}

impl Default for LLMSoWParser {
    fn default() -> Self {
        Self
    }
}
