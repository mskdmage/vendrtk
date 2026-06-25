use serde::{Deserialize, Serialize};
use vendrtk_azure::document_intelligence::models::AnalyzeOperationResponse;
use vendrtk_ocr::error::{Error, Result};
use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentIntelligenceOcrProcessedDocument {
    pub key: String,
    pub analyze_operation_response: AnalyzeOperationResponse,
}

impl OcrProcessedDocument for DocumentIntelligenceOcrProcessedDocument {
    fn key(&self) -> &str {
        &self.key
    }

    fn raw_content(&self) -> Result<String> {
        let result = self
            .analyze_operation_response
            .analyze_result
            .as_ref()
            .ok_or(Error::InvalidResult)?;
        let content = result.content.as_ref().ok_or(Error::InvalidResult)?;
        if content.is_empty() {
            return Err(Error::InvalidResult);
        }
        Ok(content.clone())
    }

    fn pages(&self) -> Result<Vec<String>> {
        let result = self
            .analyze_operation_response
            .analyze_result
            .as_ref()
            .ok_or(Error::InvalidResult)?;
        Ok(result
            .pages
            .iter()
            .map(|page| {
                page.words
                    .iter()
                    .map(|word| word.content.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect())
    }
}
