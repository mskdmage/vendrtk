use rig::client::CompletionClient;
use rig::providers::azure;

use crate::auth::Credential;
use crate::error::{Error, Result};
use crate::scope::COGNITIVE_SERVICES_SCOPE;
use vendrtk_parsers::error::{Error as ParserError, Result as ParserResult};
use vendrtk_parsers::traits::llm_client::LLMClient;

pub struct FoundryClient {
    client: azure::Client,
    deployment: String,
}

impl FoundryClient {
    pub fn new(client: azure::Client, deployment: impl Into<String>) -> Self {
        Self {
            client,
            deployment: deployment.into(),
        }
    }

    pub async fn connect(
        endpoint: &str,
        api_version: &str,
        deployment: impl Into<String>,
    ) -> Result<Self> {
        let client = azure_openai_client(endpoint, api_version).await?;
        Ok(Self::new(client, deployment))
    }
}

impl LLMClient for FoundryClient {
    fn extract<T>(
        &self,
        preamble: &str,
        content: &str,
    ) -> impl std::future::Future<Output = ParserResult<T>> + Send
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
                .extractor::<T>(deployment)
                .preamble(&preamble)
                .build();

            extractor
                .extract(content)
                .await
                .map_err(|e| ParserError::LlmRequestFailed(e.to_string()))
        }
    }
}

pub async fn azure_openai_client(endpoint: &str, api_version: &str) -> Result<azure::Client> {
    let credential = Credential::new(None, None, None)?;
    let access_token = credential
        .get_token(&[COGNITIVE_SERVICES_SCOPE], None)
        .await?;

    azure::Client::builder()
        .api_key(access_token.token.secret().to_string())
        .azure_endpoint(endpoint.to_string())
        .api_version(api_version)
        .build()
        .map_err(|e| Error::Client(e.to_string()))
}
