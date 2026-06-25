use std::future::Future;

use crate::error::Result;

pub trait LLMClient {
    fn extract<T>(&self, preamble: &str, content: &str) -> impl Future<Output = Result<T>> + Send
    where
        T: serde::de::DeserializeOwned
            + serde::Serialize
            + schemars::JsonSchema
            + Send
            + Sync
            + 'static;
}
