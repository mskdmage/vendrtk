pub mod document_store;
pub mod ocr_processed_store;
pub mod parsed_invoice_store;
pub mod parsed_sow_store;

pub use ocr_processed_store::LocalOcrProcessedStore;
pub use parsed_invoice_store::LocalParsedInvoiceStore;
pub use parsed_sow_store::LocalParsedSoWStore;
