use vendrtk_ocr::traits::client::OCRClient;
use vendrtk_storage::models::ocr_processed_document::DocumentIntelligenceOcrProcessedDocument;
use vendrtk_storage::traits::document_store::DocumentStore;
use vendrtk_storage::traits::store::ProcessedDocumentStore;

use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::classify::ClassifyStage;
use crate::traits::pipeline_stage::PipelineStage;
use crate::error::Result;

pub struct OcrStage {
    pub key: String,
}

impl<DS, OS, PIS, PSS> PipelineStage<VendorReconciliationContext<DS, OS, PIS, PSS>> for OcrStage
where
    DS: DocumentStore + Send,
    OS: ProcessedDocumentStore<DocumentIntelligenceOcrProcessedDocument> + Send,
    PIS: Send,
    PSS: Send,
{
    type Output = ClassifyStage;

    fn run(
        self,
        ctx: &mut VendorReconciliationContext<DS, OS, PIS, PSS>,
    ) -> impl std::future::Future<Output = Result<Self::Output>> + Send {
        async move {
            let key = self.key;

            if ctx.ocr_store.load_payload(&key)?.is_none() {
                tracing::debug!(stage = "ocr", %key, "cache miss");
                let bytes = ctx.document_store.load_bytes(&key)?;
                let response = ctx.ocr_client.analyze_bytes(&bytes).await?;

                let ocr = DocumentIntelligenceOcrProcessedDocument {
                    key: key.clone(),
                    analyze_operation_response: response,
                };

                ctx.ocr_store.save(ocr)?;
            } else {
                tracing::debug!(stage = "ocr", %key, "cache hit");
            }

            tracing::debug!(stage = "ocr", %key, "pipeline advanced");

            Ok(ClassifyStage { key })
        }
    }
}
