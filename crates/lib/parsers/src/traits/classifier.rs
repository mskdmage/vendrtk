use std::future::Future;

use crate::error::Result;
use crate::models::doc_type::ParsedDocumentType;
use crate::traits::llm_client::LLMClient;
use ocr::traits::ocr_processed_document::OcrProcessedDocument;

pub trait DocumentClassifier {
    fn classify<L: LLMClient + Send + Sync, O: OcrProcessedDocument + Send>(
        &self,
        client: &L,
        ocr_result: O,
    ) -> impl Future<Output = Result<ParsedDocumentType>> + Send;
}
