use crate::{error::Result, traits::OcrProcessedDocument};
use std::future::Future;

pub trait OCRClient<T: OcrProcessedDocument> {
    fn analyze_bytes(&self, bytes: &[u8]) -> impl Future<Output = Result<T>> + Send;
}
