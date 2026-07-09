use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParsedDocumentType {
    Invoice,
    SoW,
    Unknown,
}