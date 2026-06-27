use vendrtk_parsers::models::sow::ParsedSoWs;
use vendrtk_storage::models::ocr_processed_document::DocumentIntelligenceOcrProcessedDocument;
use vendrtk_storage::traits::store::ProcessedDocumentStore;

use crate::error::Error;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::output::VendorReconciliationOutput;
use crate::traits::pipeline_stage::PipelineStage;
use crate::error::Result;

pub struct ParseSoWStage {
    pub key: String,
}

impl<DS, OS, PIS, PSS> PipelineStage<VendorReconciliationContext<DS, OS, PIS, PSS>> for ParseSoWStage
where
    DS: Send,
    OS: ProcessedDocumentStore<DocumentIntelligenceOcrProcessedDocument> + Send,
    PIS: Send,
    PSS: ProcessedDocumentStore<ParsedSoWs> + Send,
    ParsedSoWs: Clone,
{
    type Output = VendorReconciliationOutput;

    fn run(
        self,
        ctx: &mut VendorReconciliationContext<DS, OS, PIS, PSS>,
    ) -> impl std::future::Future<Output = Result<Self::Output>> + Send {
        async move {
            let key = self.key;

            let output = if let Some(cached) = ctx.parsed_sow_store.load_payload(&key)? {
                VendorReconciliationOutput::Sow(cached)
            } else {
                tracing::debug!(stage = "parse", %key, document_type = "sow", "cache miss");
                let ocr = ctx.ocr_store.load_payload(&key)?.ok_or_else(|| {
                    Error::Storage(vendrtk_storage::error::Error::NotFound(key.clone()))
                })?;
                let parsed = ctx.sow_parser.parse(&ctx.llm_client, ocr).await?;
                ctx.parsed_sow_store.save(parsed.clone())?;
                VendorReconciliationOutput::Sow(parsed)
            };

            tracing::debug!(stage = "parse", %key, document_type = "sow", "pipeline advanced");

            Ok(output)
        }
    }
}
