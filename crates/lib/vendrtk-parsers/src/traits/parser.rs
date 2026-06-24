use std::future::Future;

use crate::error::Result;
use crate::traits::parsed_document::ParsedPayload;
use vendrtk_ocr::traits::ocr_processed_document::OcrProcessedDocument;

pub trait Parser<O: OcrProcessedDocument> {
    type Output: ParsedPayload;
    fn parse(
        &self,
        ocr_result: Option<O>,
        bytes: Option<&[u8]>,
    ) -> impl Future<Output = Result<Self::Output>> + Send;
}