use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    traits::document::Document,
    utils::hash_bytes,
};

mod serde_path {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::path::PathBuf;

    pub fn serialize<S: Serializer>(path: &PathBuf, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&path.to_string_lossy().replace('\\', "/"))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<PathBuf, D::Error> {
        Ok(PathBuf::from(String::deserialize(d)?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PdfDocument {
    pub key: String,
    #[serde(with = "serde_path")]
    pub path: PathBuf,
    pub size: u64,
    pub extension: String,
}

impl Document for PdfDocument {
    fn key(&self) -> &str {
        &self.key
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn size(&self) -> u64 {
        self.size
    }

    fn extension(&self) -> &str {
        &self.extension
    }
}

pub fn pdf_from_bytes(original_path: impl AsRef<Path>, bytes: &[u8]) -> Result<PdfDocument> {
    let path = original_path.as_ref();
    validate_pdf_path(path)?;
    validate_pdf_bytes(bytes)?;

    Ok(PdfDocument {
        key: hash_bytes(bytes)?,
        path: path.to_path_buf(),
        size: bytes.len() as u64,
        extension: "pdf".into(),
    })
}

fn validate_pdf_path(path: &Path) -> Result<()> {
    if path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
    {
        Ok(())
    } else {
        Err(Error::InvalidExtension {
            path: path.to_path_buf(),
        })
    }
}

fn validate_pdf_bytes(bytes: &[u8]) -> Result<()> {
    if bytes.starts_with(b"%PDF") {
        Ok(())
    } else {
        Err(Error::InvalidMagicBytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pdf_from_bytes_sets_fingerprint_and_metadata() {
        let bytes = b"%PDF-1.4 sample content";
        let doc = pdf_from_bytes("uploads/vendor/invoice.pdf", bytes).unwrap();

        assert_eq!(doc.key(), hash_bytes(bytes).unwrap());
        assert_eq!(doc.path(), "uploads/vendor/invoice.pdf");
        assert_eq!(doc.extension(), "pdf");
        assert_eq!(doc.size(), bytes.len() as u64);
    }

    #[test]
    fn pdf_document_round_trips_through_json() {
        let doc = pdf_from_bytes("file.PDF", b"%PDF-1.4").unwrap();
        let restored: PdfDocument =
            serde_json::from_str(&serde_json::to_string(&doc).unwrap()).unwrap();

        assert_eq!(doc, restored);
    }

    #[test]
    fn pdf_from_bytes_rejects_non_pdf_extension() {
        let err = pdf_from_bytes("uploads/invoice.txt", b"%PDF-1.4").unwrap_err();

        assert!(matches!(err, Error::InvalidExtension { path: _ }));
    }

    #[test]
    fn pdf_from_bytes_rejects_missing_extension() {
        let err = pdf_from_bytes("uploads/invoice", b"%PDF-1.4").unwrap_err();

        assert!(matches!(err, Error::InvalidExtension { path: _ }));
    }

    #[test]
    fn pdf_from_bytes_rejects_invalid_pdf_content() {
        let err = pdf_from_bytes("uploads/invoice.pdf", b"not a pdf").unwrap_err();

        assert!(matches!(err, Error::InvalidMagicBytes));
    }
}
