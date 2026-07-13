use ocr::traits::{OCRClient, OcrProcessedDocument};
use parsers::{
    models::{doc_type::ParsedDocumentType, invoice::ParsedInvoices},
    prebuilt::classification::llm::parser::LLMDocumentClassifier,
    traits::{DocumentClassifier, LLMClient},
};
use storage::{models::FileRef, traits::Store};

use crate::error::{Error, Result};
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::{Done, Parse};
use crate::traits::{Job, Stage};

pub struct DocClassify {
    pub file_ref: FileRef,
}

impl Stage for DocClassify {}

impl<D, C, OS, L, PS> Job<VendorReconciliationContext<D, C, OS, L, PS>, DocClassify>
where
    D: OcrProcessedDocument + Send + Sync,
    C: OCRClient<D> + Send + Sync,
    OS: Store<D> + Send,
    L: LLMClient + Send + Sync,
    PS: Store<ParsedInvoices> + Send,
{
    pub async fn run(self) -> Result<Job<VendorReconciliationContext<D, C, OS, L, PS>, Done>> {
        let file_ref = self.stage.file_ref;

        let ocr = self
            .ctx
            .ocr_store
            .lock()
            .map_err(|_| Error::Pipeline)?
            .get(&file_ref.key)?
            .ok_or(Error::Pipeline)?;

        let document_type =
            DocumentClassifier::classify(&LLMDocumentClassifier, self.ctx.llm_client.as_ref(), ocr)
                .await?;

        match document_type {
            ParsedDocumentType::Invoice => {
                Job::new(
                    self.ctx,
                    Parse {
                        file_ref,
                        document_type,
                    },
                )
                .run()
                .await
            }
            ParsedDocumentType::SoW | ParsedDocumentType::Unknown => Ok(Job::new(
                self.ctx,
                Done {
                    file_ref,
                    document_type,
                    invoice: None,
                },
            )),
        }
    }
}
