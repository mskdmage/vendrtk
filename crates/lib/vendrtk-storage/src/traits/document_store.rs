use crate::error::{Error, Result};
use crate::models::documents::PdfDocument;
use crate::traits::document::Document;
use crate::traits::store::Store;

/// Landing store: raw uploaded files keyed by content hash.
pub trait DocumentStore: Store<PdfDocument> {
    fn save_upload(
        &mut self,
        filename: impl AsRef<std::path::Path>,
        bytes: &[u8],
    ) -> Result<PdfDocument>;

    fn load_bytes(&self, key: &str) -> Result<Vec<u8>> {
        let doc = self
            .get(key)?
            .ok_or_else(|| Error::NotFound(key.to_string()))?;
        std::fs::read(doc.path()).map_err(Error::Io)
    }
}
