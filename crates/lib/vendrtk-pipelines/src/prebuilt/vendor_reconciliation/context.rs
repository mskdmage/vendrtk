use vendrtk_azure::document_intelligence::client::DocumentIntelligenceClient;
use vendrtk_azure::foundry::client::FoundryClient;
use vendrtk_parsers::prebuilt::classify::SampleDocumentClassifier;
use vendrtk_parsers::prebuilt::invoice::llm::parser::SampleInvoiceParser;
use vendrtk_parsers::prebuilt::sow::llm::parser::SampleSoWParser;

use crate::traits::pipeline_context::PipelineContext;

pub struct VendorReconciliationContext<DS, OS, PIS, PSS> {
    pub document_store: DS,
    pub ocr_store: OS,
    pub parsed_invoice_store: PIS,
    pub parsed_sow_store: PSS,
    pub ocr_client: DocumentIntelligenceClient,
    pub llm_client: FoundryClient,
    pub document_classifier: SampleDocumentClassifier,
    pub invoice_parser: SampleInvoiceParser,
    pub sow_parser: SampleSoWParser,
}

impl<DS, OS, PIS, PSS> VendorReconciliationContext<DS, OS, PIS, PSS> {
    pub fn new(
        document_store: DS,
        ocr_store: OS,
        parsed_invoice_store: PIS,
        parsed_sow_store: PSS,
        ocr_client: DocumentIntelligenceClient,
        llm_client: FoundryClient,
    ) -> Self {
        Self {
            document_store,
            ocr_store,
            parsed_invoice_store,
            parsed_sow_store,
            ocr_client,
            llm_client,
            document_classifier: SampleDocumentClassifier::new(),
            invoice_parser: SampleInvoiceParser::new(),
            sow_parser: SampleSoWParser::new(),
        }
    }
}

impl<DS, OS, PIS, PSS> PipelineContext for VendorReconciliationContext<DS, OS, PIS, PSS>
where
    DS: Send,
    OS: Send,
    PIS: Send,
    PSS: Send,
{
}
