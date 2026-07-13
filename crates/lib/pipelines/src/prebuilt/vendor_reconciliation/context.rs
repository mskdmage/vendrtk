use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

use ocr::traits::{OCRClient, OcrProcessedDocument};
use parsers::{models::invoice::ParsedInvoices, traits::LLMClient};
use storage::{
    models::File,
    traits::{Repository, Store},
};

use crate::traits::Context;

pub struct VendorReconciliationContext<D, C, OS, L, PS>
where
    D: OcrProcessedDocument,
    C: OCRClient<D>,
    OS: Store<D>,
    L: LLMClient,
    PS: Store<ParsedInvoices>,
{
    pub landing: Arc<dyn Repository<File>>,
    pub ocr_client: Arc<C>,
    pub ocr_store: Arc<Mutex<OS>>,
    pub llm_client: Arc<L>,
    pub parsed_store: Arc<Mutex<PS>>,
    _doc: PhantomData<D>,
}

impl<D, C, OS, L, PS> VendorReconciliationContext<D, C, OS, L, PS>
where
    D: OcrProcessedDocument,
    C: OCRClient<D>,
    OS: Store<D>,
    L: LLMClient,
    PS: Store<ParsedInvoices>,
{
    pub fn new(
        landing: Arc<dyn Repository<File>>,
        ocr_client: Arc<C>,
        ocr_store: Arc<Mutex<OS>>,
        llm_client: Arc<L>,
        parsed_store: Arc<Mutex<PS>>,
    ) -> Self {
        Self {
            landing,
            ocr_client,
            ocr_store,
            llm_client,
            parsed_store,
            _doc: PhantomData,
        }
    }
}

impl<D, C, OS, L, PS> Context for VendorReconciliationContext<D, C, OS, L, PS>
where
    D: OcrProcessedDocument + Send + Sync,
    C: OCRClient<D> + Send + Sync,
    OS: Store<D> + Send,
    L: LLMClient + Send + Sync,
    PS: Store<ParsedInvoices> + Send,
{
}
