use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::models::doc_type::DocumentType as ParsedDocumentType;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum DocumentType {
    #[schemars(
        title = "Invoice",
        description = "A bill or invoice requesting payment for goods or services."
    )]
    Invoice,
    #[serde(alias = "Statement of Work", alias = "SOW", alias = "statement of work")]
    #[schemars(
        title = "SoW",
        description = "A rate schedule, pricing agreement, or statement of work (SoW)."
    )]
    SoW,
    #[schemars(
        title = "Unknown",
        description = "Document type could not be determined with confidence."
    )]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClassifierVerdict {
    #[schemars(
        title = "Document Type",
        description = "The type of document: invoice, statement of work (SoW), or unknown."
    )]
    pub document_type: DocumentType,
}

impl From<DocumentType> for ParsedDocumentType {
    fn from(value: DocumentType) -> Self {
        match value {
            DocumentType::Invoice => Self::Invoice,
            DocumentType::SoW => Self::SoW,
            DocumentType::Unknown => Self::Unknown,
        }
    }
}
