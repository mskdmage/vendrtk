use crate::azure::auth::Credential;
use crate::azure::foundry::deployment::Deployment;
use crate::azure::scope::COGNITIVE_SERVICES_SCOPE;
use crate::error::{Error, Result};
use rig::providers::azure;

pub struct FoundryClient {
    pub client: azure::Client,
    pub deployment: Deployment,
}

impl FoundryClient {
    pub fn new(client: azure::Client, deployment: &Deployment) -> Self {
        Self {
            client,
            deployment: *deployment,
        }
    }

    pub async fn connect(
        endpoint: &str,
        api_version: &str,
        deployment: &Deployment,
    ) -> Result<Self> {
        let client = azure_openai_client(endpoint, api_version).await?;
        Ok(Self::new(client, deployment))
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
