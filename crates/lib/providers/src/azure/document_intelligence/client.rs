use std::sync::Arc;
use serde_json::json;
use base64::{
    Engine,
    engine::general_purpose::STANDARD,
};
use reqwest::{
    Client as HttpClient,
    Response,
    StatusCode,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use crate::azure::{
    auth::{Auth, Credential},
    scope::COGNITIVE_SERVICES_SCOPE,
};
use crate::error::{Error, Result};
use crate::azure::document_intelligence::{
    api_version::ApiVersion,
    config::Config,
    headers::{OPERATION_LOCATION_HEADER, SUBSCRIPTION_KEY_HEADER},
    models::AnalyzeOperationResponse,
};

pub struct DocumentIntelligenceClient {
    http_client: HttpClient,
    endpoint: String,
    api_version: ApiVersion,
    auth: Auth,
    config: Config,
}

impl DocumentIntelligenceClient {
    pub fn new(
        endpoint: String,
        api_version: ApiVersion,
        auth: Auth,
        config: Config,
    ) -> Result<Self> {
        let endpoint = endpoint.trim_end_matches('/').to_string();
        if endpoint.is_empty() {
            return Err(Error::Config(
                "AZURE_COGNITIVE_SERVICES_ENDPOINT is not set".into(),
            ));
        }

        Ok(Self {
            http_client: HttpClient::new(),
            endpoint,
            api_version,
            auth,
            config,
        })
    }

    // TODO: Must think about ergonomics for supporting optional env variables,
    // perhaps we dont support from_env at all, and just use parameters, and make it 
    // responsibility of the caller to pass in the necessary variables,
    pub fn from_env(config: Option<Config>) -> Result<Self> {
        let endpoint = std::env::var("AZURE_COGNITIVE_SERVICES_ENDPOINT")
            .map_err(|_| Error::Config("AZURE_COGNITIVE_SERVICES_ENDPOINT is not set".into()))?;

        let auth = if let Ok(key) = std::env::var("AZURE_COGNITIVE_SERVICES_KEY") {
            Auth::ApiKey(key)
        } else {
            Auth::Credential(Arc::new(Credential::new(None, None, None).map_err(|e| {
                Error::Auth(format!(
                    "set AZURE_COGNITIVE_SERVICES_KEY or use Entra (az login): {e}"
                ))
            })?))
        };

        Self::new(
            endpoint,
            ApiVersion::Default,
            auth,
            config.unwrap_or_default(),
        )
    }

    pub fn with_api_key(
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        config: Option<Config>,
    ) -> Result<Self> {
        Self::new(
            endpoint.into(),
            ApiVersion::Default,
            Auth::ApiKey(api_key.into()),
            config.unwrap_or_default(),
        )
    }

    pub fn with_credential(
        endpoint: impl Into<String>,
        credential: Credential,
        config: Option<Config>,
    ) -> Result<Self> {
        Self::new(
            endpoint.into(),
            ApiVersion::Default,
            Auth::Credential(Arc::new(credential)),
            config.unwrap_or_default(),
        )
    }

    async fn apply_auth(&self, headers: &mut HeaderMap) -> Result<()> {
        match &self.auth {
            Auth::ApiKey(key) => {
                headers.insert(
                    SUBSCRIPTION_KEY_HEADER,
                    HeaderValue::from_str(key)
                        .map_err(|e| Error::Auth(format!("invalid API key: {e}")))?,
                );
            }
            Auth::Credential(credential) => {
                let token = credential
                    .get_token(&[COGNITIVE_SERVICES_SCOPE], None)
                    .await?;
                let value = format!("Bearer {}", token.token.secret());
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&value)
                        .map_err(|e| Error::Auth(format!("invalid bearer token: {e}")))?,
                );
            }
        }
        Ok(())
    }

    async fn start_analyze(&self, model_id: &str, body: serde_json::Value) -> Result<String> {
        let url = format!(
            "{}/documentintelligence/documentModels/{}:analyze?api-version={}",
            self.endpoint,
            model_id,
            self.api_version.as_ref()
        );

        let mut headers = HeaderMap::new();
        self.apply_auth(&mut headers).await?;

        let response: Response = self
            .http_client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::request(format!("POST {url}"), e))?;

        let status = response.status();
        if status == StatusCode::ACCEPTED {
            return Self::operation_location(response.headers());
        }

        let message = response.text().await.unwrap_or_default();
        Err(Error::Api {
            status: status.as_u16(),
            message,
        })
    }

    async fn poll_until_done(&self, operation_url: &str) -> Result<AnalyzeOperationResponse> {
        for _ in 0..self.config.max_attempts() {
            let mut headers = HeaderMap::new();
            self.apply_auth(&mut headers).await?;

            let response = self
                .http_client
                .get(operation_url)
                .headers(headers)
                .send()
                .await
                .map_err(|e| Error::request(format!("GET {operation_url}"), e))?;

            let status = response.status();
            if !status.is_success() {
                let message = response.text().await.unwrap_or_default();
                return Err(Error::Api {
                    status: status.as_u16(),
                    message,
                });
            }

            let body: AnalyzeOperationResponse = response.json().await.map_err(|e| {
                Error::request(format!("decode poll response from {operation_url}"), e)
            })?;

            match body.status.as_str() {
                "succeeded" => return Ok(body),
                "failed" => {
                    let message = body
                        .error
                        .and_then(|e| e.message)
                        .unwrap_or_else(|| "analyze operation failed".into());
                    return Err(Error::Api {
                        status: 500,
                        message,
                    });
                }
                _ => tokio::time::sleep(self.config.interval()).await,
            }
        }

        Err(Error::PollTimeout {
            attempts: self.config.max_attempts(),
        })
    }

    fn operation_location(headers: &HeaderMap) -> Result<String> {
        headers
            .get(OPERATION_LOCATION_HEADER)
            .and_then(|value| value.to_str().ok())
            .map(str::to_string)
            .ok_or_else(|| Error::MissingHeader(OPERATION_LOCATION_HEADER.into()))
    }

    pub async fn analyze(&self, model_id: &str, bytes: &[u8]) -> Result<AnalyzeOperationResponse> {
        let body = json!({ "base64Source": STANDARD.encode(bytes) });
        let operation_url = self.start_analyze(model_id, body).await?;
        self.poll_until_done(&operation_url).await
    }
}
