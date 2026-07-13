use crate::error::{Error, Result};
use crate::prebuilt::vendor_reconciliation::context::VendorReconciliationContext;
use crate::prebuilt::vendor_reconciliation::stages::Done;
use crate::traits::{Job, Stage};
use ocr::traits::{OCRClient, OcrProcessedDocument};
use parsers::{
    models::{doc_type::ParsedDocumentType, invoice::ParsedInvoices},
    prebuilt::invoice::llm::parser::LLMInvoiceParser,
    traits::{LLMClient, Parser},
};
use storage::{models::FileRef, traits::Store};

pub struct Parse {
    pub file_ref: FileRef,
    pub document_type: ParsedDocumentType,
}

impl Stage for Parse {}

impl<D, C, OS, L, PS> Job<VendorReconciliationContext<D, C, OS, L, PS>, Parse>
where
    D: OcrProcessedDocument + Send + Sync,
    C: OCRClient<D> + Send + Sync,
    OS: Store<D> + Send,
    L: LLMClient + Send + Sync,
    PS: Store<ParsedInvoices> + Send,
{
    pub async fn run(self) -> Result<Job<VendorReconciliationContext<D, C, OS, L, PS>, Done>> {
        let Parse {
            file_ref,
            document_type,
        } = self.stage;

        {
            let store = self.ctx.parsed_store.lock().map_err(|_| Error::Pipeline)?;
            if store.exists(&file_ref.key)? {
                let invoice = store.get(&file_ref.key)?;
                return Ok(Job::new(
                    self.ctx.clone(),
                    Done {
                        file_ref,
                        document_type,
                        invoice,
                    },
                ));
            }
        }

        let ocr = self
            .ctx
            .ocr_store
            .lock()
            .map_err(|_| Error::Pipeline)?
            .get(&file_ref.key)?
            .ok_or(Error::Pipeline)?;

        let mut invoice = Parser::parse(
            &LLMInvoiceParser,
            self.ctx.llm_client.as_ref(),
            Some(ocr),
            None,
        )
        .await?;
        invoice.key = file_ref.key.clone();

        self.ctx
            .parsed_store
            .lock()
            .map_err(|_| Error::Pipeline)?
            .create(&file_ref.key, invoice.clone())?;

        Ok(Job::new(
            self.ctx,
            Done {
                file_ref,
                document_type,
                invoice: Some(invoice),
            },
        ))
    }
}
