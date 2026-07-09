use crate::error::Result;
use std::future::Future;

// TODO: Currently coupled to RIG return type for structured output.
// Might need to decouple this to support other structured output providers.
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
