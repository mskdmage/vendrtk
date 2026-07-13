use ocr::traits::{OCRClient, OcrProcessedDocument};
use parsers::{models::invoice::ParsedInvoices, traits::LLMClient};
use storage::traits::Store;

use crate::error::Result;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::Ocr;
use crate::traits::{Job, Stage};

pub struct Ingest {
    pub input: Vec<u8>,
}

impl Stage for Ingest {}

impl<D, C, OS, L, PS> Job<VendorReconciliationContext<D, C, OS, L, PS>, Ingest>
where
    D: OcrProcessedDocument + Send + Sync,
    C: OCRClient<D> + Send + Sync,
    OS: Store<D> + Send,
    L: LLMClient + Send + Sync,
    PS: Store<ParsedInvoices> + Send,
{
    pub async fn run(self) -> Result<Job<VendorReconciliationContext<D, C, OS, L, PS>, Ocr>> {
        let file_ref = self.ctx.landing.put(&self.stage.input)?;
        Ok(Job::new(self.ctx, Ocr { file_ref }))
    }
}
