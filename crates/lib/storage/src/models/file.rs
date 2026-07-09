use crate::traits::Blob;
use crate::utils::hashing::calculate_hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileKind {
    Pdf,
    Jpeg,
    Xlsx,
    Xls,
    Unknown,
}

// TODO: might need a more robust way to determine the folder name
impl FileKind {
    pub fn folder(&self) -> &'static str {
        match self {
            FileKind::Pdf => "pdf",
            FileKind::Jpeg => "jpeg",
            FileKind::Xlsx => "xlsx",
            FileKind::Xls => "xls",
            FileKind::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FileRef {
    pub key: String,
    pub kind: FileKind,
}

#[derive(Debug, PartialEq)]
pub struct File {
    kind: FileKind,
    bytes: Vec<u8>,
}


// TODO: might need a more robust way to determine the file type
impl File {
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        let kind = match bytes.as_slice() {
            // Magic bytes for PDF files
            [0x25, 0x50, 0x44, 0x46, 0x2D, ..] => FileKind::Pdf,
            // Magic bytes for JPEG files
            [0xFF, 0xD8, 0xFF, ..] => FileKind::Jpeg,
            // Magic bytes for XLSX files
            [0x50, 0x4B, 0x03, 0x04, ..] => FileKind::Xlsx,
            // Magic bytes for XLS files
            [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, ..] => FileKind::Xls,
            _ => FileKind::Unknown,
        };
        Self { kind, bytes }
    }

    pub fn kind(&self) -> FileKind {
        self.kind
    }
}

impl Blob for File {
    fn key(&self) -> String {
        calculate_hash(&self.bytes)
    }

    fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_from_bytes() {
        let bytes = b"%PDF-1.4 test".to_vec();
        let file = File::from_bytes(bytes);
        assert_eq!(file.kind(), FileKind::Pdf);
    }
}