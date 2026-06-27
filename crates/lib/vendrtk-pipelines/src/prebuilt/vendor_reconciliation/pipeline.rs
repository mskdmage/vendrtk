use vendrtk_parsers::models::invoice::ParsedInvoices;
use vendrtk_parsers::models::sow::ParsedSoWs;
use vendrtk_storage::local::document_store::LocalDocumentStore;
use vendrtk_storage::local::{LocalOcrProcessedStore, LocalParsedStore};
use vendrtk_storage::models::documents::PdfDocument;
use vendrtk_storage::models::ocr_processed_document::DocumentIntelligenceOcrProcessedDocument;
use vendrtk_storage::traits::document_store::DocumentStore;
use vendrtk_storage::traits::store::ProcessedDocumentStore;

use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::input::VendorReconciliationInput;
use crate::prebuilt::vendor_reconciliation::output::VendorReconciliationOutput;
use crate::prebuilt::vendor_reconciliation::stages::classify::ClassifiedStage;
use crate::prebuilt::vendor_reconciliation::stages::ingest::IngestStage;
use crate::traits::pipeline::Pipeline;
use crate::traits::pipeline_stage::PipelineStage;
use crate::error::Result;

pub struct VendorReconciliationPipeline<DS, OS, PIS, PSS> {
    ctx: VendorReconciliationContext<DS, OS, PIS, PSS>,
}

pub type LocalVendorReconciliationPipeline = VendorReconciliationPipeline<
    LocalDocumentStore<PdfDocument>,
    LocalOcrProcessedStore<DocumentIntelligenceOcrProcessedDocument>,
    LocalParsedStore<ParsedInvoices>,
    LocalParsedStore<ParsedSoWs>,
>;

impl<DS, OS, PIS, PSS> VendorReconciliationPipeline<DS, OS, PIS, PSS> {
    pub fn new(ctx: VendorReconciliationContext<DS, OS, PIS, PSS>) -> Self {
        Self { ctx }
    }
}

impl<DS, OS, PIS, PSS> Pipeline for VendorReconciliationPipeline<DS, OS, PIS, PSS>
where
    DS: DocumentStore + Send,
    OS: ProcessedDocumentStore<DocumentIntelligenceOcrProcessedDocument> + Send,
    PIS: ProcessedDocumentStore<ParsedInvoices> + Send,
    PSS: ProcessedDocumentStore<ParsedSoWs> + Send,
    ParsedInvoices: Clone,
    ParsedSoWs: Clone,
{
    type Input = VendorReconciliationInput;
    type Output = VendorReconciliationOutput;

    fn run(&mut self, input: Self::Input) -> impl std::future::Future<Output = Result<Self::Output>> + Send {
        async {
            let ocr = IngestStage::from(input).run(&mut self.ctx).await?;
            let classify = ocr.run(&mut self.ctx).await?;
            match classify.run(&mut self.ctx).await? {
                ClassifiedStage::Invoice(parse) => parse.run(&mut self.ctx).await,
                ClassifiedStage::Sow(parse) => parse.run(&mut self.ctx).await,
            }
        }
    }
}
