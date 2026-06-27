use vendrtk_storage::traits::document_store::DocumentStore;

use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::input::VendorReconciliationInput;
use crate::prebuilt::vendor_reconciliation::stages::ocr::OcrStage;
use crate::traits::pipeline_stage::PipelineStage;
use crate::error::Result;

pub struct IngestStage {
    pub filename: String,
    pub bytes: Vec<u8>,
}

impl From<VendorReconciliationInput> for IngestStage {
    fn from(input: VendorReconciliationInput) -> Self {
        Self {
            filename: input.filename,
            bytes: input.bytes,
        }
    }
}

impl<DS, OS, PIS, PSS> PipelineStage<VendorReconciliationContext<DS, OS, PIS, PSS>> for IngestStage
where
    DS: DocumentStore + Send,
    OS: Send,
    PIS: Send,
    PSS: Send,
{
    type Output = OcrStage;

    fn run(
        self,
        ctx: &mut VendorReconciliationContext<DS, OS, PIS, PSS>,
    ) -> impl std::future::Future<Output = Result<Self::Output>> + Send {
        async move {
            tracing::debug!(
                stage = "ingest",
                filename = %self.filename,
                size = self.bytes.len(),
                "starting"
            );

            let doc = ctx
                .document_store
                .save_upload(&self.filename, &self.bytes)?;
            let key = doc.key.clone();

            tracing::debug!(stage = "ingest", %key, "pipeline advanced");

            Ok(OcrStage { key })
        }
    }
}
