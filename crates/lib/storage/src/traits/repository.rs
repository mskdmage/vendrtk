use crate::error::Result;
use crate::models::{FileKind, FileRef};
use crate::traits::Blob;

// TODO: Distinction between Store and Repository
// Store is a trait for Serializable data (i.e. Json Responses)
// Repository is a trait for storing binary data (i.e. Files)
// Naming is a bit confusing, will need to come up with a better name
pub trait Repository<T: Blob>: Send + Sync {
    fn put(&self, bytes: &[u8]) -> Result<FileRef>;
    fn get(&self, kind: FileKind, key: &str) -> Result<Option<T>>;
    fn exists(&self, kind: FileKind, key: &str) -> Result<bool>;
}
