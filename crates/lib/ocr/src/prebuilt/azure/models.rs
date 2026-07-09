pub use providers::azure::document_intelligence::models::*;

use crate::error::{Error, Result};
use crate::traits::OcrProcessedDocument;

impl OcrProcessedDocument for AnalyzeOperationResponse {
    fn key(&self) -> &str {
        self.analyze_result
            .as_ref()
            .and_then(|r| r.model_id.as_deref())
            .unwrap_or(self.status.as_str())
    }

    fn raw_content(&self) -> Result<String> {
        self.analyze_result
            .as_ref()
            .and_then(|r| r.content.clone())
            .ok_or(Error::InvalidResult)
    }

    fn pages(&self) -> Result<Vec<String>> {
        let result = self.analyze_result.as_ref().ok_or(Error::InvalidResult)?;
        Ok(result
            .pages
            .iter()
            .map(|page| {
                page.lines
                    .iter()
                    .map(|line| line.content.as_str())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .collect())
    }
}
