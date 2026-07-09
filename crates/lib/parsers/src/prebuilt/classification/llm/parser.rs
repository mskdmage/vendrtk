use std::future::Future;

use crate::error::{Error, Result};
use crate::models::doc_type::ParsedDocumentType;
use crate::prebuilt::classification::schemas::ClassifierVerdict;
use crate::traits::LLMClient;
use ocr::traits::OcrProcessedDocument;

const CLASSIFICATION_PREAMBLE: &str = "Classify the document type from the OCR text. \
Return invoice for bills requesting payment for goods or services. \
Return SoW for rate schedules, pricing agreements, or statements of work. \
Return unknown when the document type cannot be determined with confidence.";

pub struct LLMDocumentClassifier;

impl LLMDocumentClassifier {
    pub const fn new() -> Self {
        Self
    }

    pub fn classify<C, O>(
        &self,
        client: &C,
        ocr_result: O,
    ) -> impl Future<Output = Result<ParsedDocumentType>> + Send
    where
        C: LLMClient + Send + Sync,
        O: OcrProcessedDocument + Send,
    {
        async move {
            let content = ocr_result
                .raw_content()
                .map_err(|_e| Error::EmptyOcrContent)?;
            if content.trim().is_empty() {
                return Err(Error::EmptyOcrContent);
            }

            let verdict: ClassifierVerdict =
                client.extract(CLASSIFICATION_PREAMBLE, &content).await?;

            Ok(verdict.document_type.into())
        }
    }
}

pub use LLMDocumentClassifier as SampleDocumentClassifier;
