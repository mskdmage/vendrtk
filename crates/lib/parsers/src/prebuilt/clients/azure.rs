use providers::azure::foundry::client::FoundryClient;
use rig::client::CompletionClient;
use crate::error::{Error, Result};
use crate::traits::LLMClient;

impl LLMClient for FoundryClient {
    fn extract<T>(
        &self,
        preamble: &str,
        content: &str,
    ) -> impl std::future::Future<Output = Result<T>> + Send
    where
        T: serde::de::DeserializeOwned
            + serde::Serialize
            + schemars::JsonSchema
            + Send
            + Sync
            + 'static,
    {
        let client = self.client.clone();
        let deployment = self.deployment.clone();
        let preamble = preamble.to_string();
        let content = content.to_string();

        async move {
            let extractor = client
                .extractor::<T>(deployment.as_ref())
                .preamble(&preamble)
                .build();

            extractor
                .extract(content)
                .await
                .map_err(|e| Error::LlmRequestFailed(e.to_string()))
        }
    }
}