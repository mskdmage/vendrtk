use crate::error::Result;

pub trait OcrProcessedDocument {
    fn key(&self) -> &str;
    fn raw_content(&self) -> Result<String>;
    fn pages(&self) -> Result<Vec<String>>;
}
