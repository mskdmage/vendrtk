mod file_repository;
pub use file_repository::LocalRepository;

mod ocr_processed_store;
pub use ocr_processed_store::LocalOcrProcessedStore;


mod parsed_store;
pub use parsed_store::LocalParsedStore;