use crate::error::Result;
use std::future::Future;
use std::path::Path;

pub trait OCRClient<T> {
    fn analyze_path(&self, path: &Path) -> impl Future<Output = Result<T>> + Send;
    fn analyze_bytes(&self, bytes: &[u8]) -> impl Future<Output = Result<T>> + Send;
}
