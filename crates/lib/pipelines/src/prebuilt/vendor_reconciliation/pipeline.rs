use ocr::traits::{OCRClient, OcrProcessedDocument};
use parsers::{models::invoice::ParsedInvoices, traits::LLMClient};
use storage::traits::Store;

use crate::error::Result;
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::{Done, Ingest};
use crate::traits::{Job, Pipeline};

/// Vendor-reconciliation pipeline: owns context and runs jobs end-to-end.
pub type VendorReconciliationPipeline<D, C, OS, L, PS> =
    Pipeline<VendorReconciliationContext<D, C, OS, L, PS>>;

impl<D, C, OS, L, PS> Pipeline<VendorReconciliationContext<D, C, OS, L, PS>>
where
    D: OcrProcessedDocument + Send + Sync,
    C: OCRClient<D> + Send + Sync,
    OS: Store<D> + Send,
    L: LLMClient + Send + Sync,
    PS: Store<ParsedInvoices> + Send,
{
    pub async fn run(&self, input: Vec<u8>) -> Result<Done> {
        let done = Job::new(self.ctx(), Ingest { input })
            .run()
            .await?
            .run()
            .await?
            .run()
            .await?;
        Ok(done.stage)
    }

    /// Run many files with at most `concurrency` jobs in flight at once.
    pub async fn run_many(&self, files: Vec<Vec<u8>>, concurrency: usize) -> Vec<Result<Done>> {
        let concurrency = concurrency.max(1);
        let mut results = Vec::with_capacity(files.len());

        for chunk in files.chunks(concurrency) {
            let futs = chunk.iter().cloned().map(|input| self.run(input));
            results.extend(futures::future::join_all(futs).await);
        }

        results
    }
}
