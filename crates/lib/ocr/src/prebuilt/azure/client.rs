pub use providers::azure::document_intelligence::client::DocumentIntelligenceClient;

use providers::azure::document_intelligence::{
    models::AnalyzeOperationResponse, prebuilt_model::PrebuiltModel,
};

use crate::error::Result;
use crate::traits::OCRClient;

impl OCRClient<AnalyzeOperationResponse> for DocumentIntelligenceClient {
    async fn analyze_bytes(&self, bytes: &[u8]) -> Result<AnalyzeOperationResponse> {
        let model_id = PrebuiltModel::Invoice.as_ref();
        tracing::debug!(
            "document intelligence analyze start: model_id={model_id} bytes_len={}",
            bytes.len()
        );

        let result = self.analyze(model_id, bytes).await?;

        let page_count = result
            .analyze_result
            .as_ref()
            .map(|r| r.pages.len())
            .unwrap_or(0);
        tracing::debug!(
            "document intelligence analyze done: model_id={model_id} page_count={page_count}"
        );

        Ok(result)
    }
}
