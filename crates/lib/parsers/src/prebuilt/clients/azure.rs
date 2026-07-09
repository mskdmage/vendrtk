pub use providers::azure::foundry::client::FoundryClient;

use providers::azure::foundry::{api_version::ApiVersion, deployment::Deployment};
use rig::client::CompletionClient;

use crate::error::{Error, Result};
use crate::traits::LLMClient;

pub async fn connect_from_env() -> Result<FoundryClient> {
    let endpoint = std::env::var("AZURE_OPENAI_ENDPOINT")
        .map_err(|_| Error::LlmRequestFailed("AZURE_OPENAI_ENDPOINT is not set".into()))?;

    FoundryClient::connect(
        &endpoint,
        ApiVersion::Default.as_ref(),
        &Deployment::Default,
    )
    .await
    .map_err(|error| Error::LlmRequestFailed(error.to_string()))
}

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
        let deployment = self.deployment;
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
