use crate::error::{Error, Result};
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::DocClassify;
use crate::traits::{Job, Stage};
use ocr::traits::{OCRClient, OcrProcessedDocument};
use parsers::{models::invoice::ParsedInvoices, traits::LLMClient};
use storage::{
    models::FileRef,
    traits::{Blob, Store},
};

pub struct Ocr {
    pub file_ref: FileRef,
}

impl Stage for Ocr {}

impl<D, C, OS, L, PS> Job<VendorReconciliationContext<D, C, OS, L, PS>, Ocr>
where
    D: OcrProcessedDocument + Send + Sync,
    C: OCRClient<D> + Send + Sync,
    OS: Store<D> + Send,
    L: LLMClient + Send + Sync,
    PS: Store<ParsedInvoices> + Send,
{
    pub async fn run(
        self,
    ) -> Result<Job<VendorReconciliationContext<D, C, OS, L, PS>, DocClassify>> {
        let file_ref = self.stage.file_ref;

        let cached = self
            .ctx
            .ocr_store
            .lock()
            .map_err(|_| Error::Pipeline)?
            .exists(&file_ref.key)?;

        if !cached {
            let file = self
                .ctx
                .landing
                .get(file_ref.kind, &file_ref.key)?
                .ok_or(Error::Pipeline)?;

            let doc = self.ctx.ocr_client.analyze_bytes(&file.bytes()).await?;

            match self
                .ctx
                .ocr_store
                .lock()
                .map_err(|_| Error::Pipeline)?
                .create(&file_ref.key, doc)
            {
                Ok(()) => {}
                Err(storage::error::Error::Repository(message))
                    if message.contains("key already exists") => {}
                Err(error) => return Err(error.into()),
            }
        }

        Ok(Job::new(self.ctx, DocClassify { file_ref }))
    }
}
