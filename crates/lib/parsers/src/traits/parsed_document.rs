use serde::{Serialize, de::DeserializeOwned};
use crate::error::Result;

pub trait ParsedPayload: Serialize + DeserializeOwned {
    fn key(&self) -> &str;
}

pub trait ParsedDocument<T>: ParsedPayload {
    fn results(&self) -> Result<Vec<T>>;
}
