use crate::error::Result;
use serde::{Serialize, de::DeserializeOwned};

pub trait OcrProcessedDocument: Serialize + DeserializeOwned {
    fn key(&self) -> &str;
    fn raw_content(&self) -> Result<String>;
    fn pages(&self) -> Result<Vec<String>>;
}
