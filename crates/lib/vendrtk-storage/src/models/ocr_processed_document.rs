use serde::{Deserialize, Serialize};
use vendrtk_ocr::error::Result;
use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;
use vendrtk_ocr::azure_document_intelligence::models::AnalyzeOperationResponse;

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
        Ok(self.analyze_operation_response.analyze_result.as_ref().unwrap().content.as_ref().unwrap().clone())
    }
    fn pages(&self) -> Result<Vec<String>> {
        Ok(self.analyze_operation_response.analyze_result.as_ref().unwrap().pages.iter().map(|page| page.words.iter().map(|word| word.content.clone()).collect::<Vec<String>>().join(" ")).collect())
    }
}
