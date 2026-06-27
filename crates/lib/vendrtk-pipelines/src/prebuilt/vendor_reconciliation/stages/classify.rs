use vendrtk_parsers::models::doc_type::DocumentType;
use vendrtk_parsers::models::invoice::ParsedInvoices;
use vendrtk_parsers::models::sow::ParsedSoWs;
use vendrtk_storage::models::ocr_processed_document::DocumentIntelligenceOcrProcessedDocument;
use vendrtk_storage::traits::store::ProcessedDocumentStore;

use crate::error::Error;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::parse_invoice::ParseInvoiceStage;
use crate::prebuilt::vendor_reconciliation::stages::parse_sow::ParseSoWStage;
use crate::traits::pipeline_stage::PipelineStage;
use crate::error::Result;

pub struct ClassifyStage {
    pub key: String,
}

pub enum ClassifiedStage {
    Invoice(ParseInvoiceStage),
    Sow(ParseSoWStage),
}

impl<DS, OS, PIS, PSS> PipelineStage<VendorReconciliationContext<DS, OS, PIS, PSS>> for ClassifyStage
where
    DS: Send,
    OS: ProcessedDocumentStore<DocumentIntelligenceOcrProcessedDocument> + Send,
    PIS: ProcessedDocumentStore<ParsedInvoices> + Send,
    PSS: ProcessedDocumentStore<ParsedSoWs> + Send,
{
    type Output = ClassifiedStage;

    fn run(
        self,
        ctx: &mut VendorReconciliationContext<DS, OS, PIS, PSS>,
    ) -> impl std::future::Future<Output = Result<Self::Output>> + Send {
        async move {
            let key = self.key;

            let invoice_cached = ctx.parsed_invoice_store.exists(&key)?;
            let sow_cached = ctx.parsed_sow_store.exists(&key)?;

            let stage = match (invoice_cached, sow_cached) {
                (true, true) => return Err(Error::ConflictingParsedCache),
                (true, false) => {
                    tracing::debug!(stage = "classify", %key, document_type = "invoice", "cache hit");
                    ClassifiedStage::Invoice(ParseInvoiceStage { key })
                }
                (false, true) => {
                    tracing::debug!(stage = "classify", %key, document_type = "sow", "cache hit");
                    ClassifiedStage::Sow(ParseSoWStage { key })
                }
                (false, false) => {
                    tracing::debug!(stage = "classify", %key, "cache miss");
                    let ocr = ctx.ocr_store.load_payload(&key)?.ok_or_else(|| {
                        Error::Storage(vendrtk_storage::error::Error::NotFound(key.clone()))
                    })?;

                    match ctx
                        .document_classifier
                        .classify(&ctx.llm_client, ocr)
                        .await?
                    {
                        DocumentType::Invoice => {
                            ClassifiedStage::Invoice(ParseInvoiceStage { key })
                        }
                        DocumentType::SoW => ClassifiedStage::Sow(ParseSoWStage { key }),
                        DocumentType::Unknown => return Err(Error::UnknownDocumentType),
                    }
                }
            };

            Ok(stage)
        }
    }
}
