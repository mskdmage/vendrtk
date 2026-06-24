use std::future::Future;

use crate::error::Result;
use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;

pub trait Parser<T, O: OcrProcessedDocument> {
    fn parse(&self, ocr_result: Option<O>, bytes: Option<&[u8]>) -> impl Future<Output = Result<T>>;
}