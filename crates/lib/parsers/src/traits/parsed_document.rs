use crate::error::Result;
use serde::{Serialize, de::DeserializeOwned};

pub trait ParsedPayload: Serialize + DeserializeOwned {
    fn key(&self) -> &str;
}

pub trait ParsedDocument<T>: ParsedPayload {
    fn results(&self) -> Result<Vec<T>>;
}
