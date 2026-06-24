use crate::error::{Error, Result};
use rig::providers::azure;
use vendrtk_ocr::azure_document_intelligence::auth::Credential;

const COGNITIVE_SERVICES_SCOPE: &str = "https://cognitiveservices.azure.com/.default";

pub const DEFAULT_API_VERSION: &str = "2025-04-01-preview";
pub const DEFAULT_DEPLOYMENT: &str = "gpt-5.2";

pub async fn azure_openai_client(endpoint: &str, api_version: &str) -> Result<azure::Client> {
    let credential = Credential::new(None, None, None).map_err(|e| Error::Client(e.to_string()))?;
    let access_token = credential
        .get_token(&[COGNITIVE_SERVICES_SCOPE], None)
        .await
        .map_err(|e| Error::Client(e.to_string()))?;

    azure::Client::builder()
        .api_key(access_token.token.secret().to_string())
        .azure_endpoint(endpoint.to_string())
        .api_version(api_version)
        .build()
        .map_err(|e| Error::Client(e.to_string()))
}
