use parsers::models::{doc_type::ParsedDocumentType, invoice::ParsedInvoices};
use storage::models::FileRef;

use crate::traits::Stage;

pub struct Done {
    pub file_ref: FileRef,
    pub document_type: ParsedDocumentType,
    pub invoice: Option<ParsedInvoices>,
}

impl Stage for Done {}
