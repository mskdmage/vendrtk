use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client as HttpClient, Response, StatusCode};
use serde_json::json;
use std::path::Path;

use crate::auth::{Auth, Credential};
use crate::document_intelligence::api_version::ApiVersion;
use crate::document_intelligence::config::Config;
use crate::document_intelligence::models::AnalyzeOperationResponse;
use crate::document_intelligence::prebuilt_model::PrebuiltModel;
use crate::error::{Error, Result};
use vendrtk_ocr::error::{Error as OCRError, Result as OCRResult};
use vendrtk_ocr::traits::client::OCRClient;

const COGNITIVE_SERVICES_SCOPE: &str = "https://cognitiveservices.azure.com/.default";
const SUBSCRIPTION_KEY_HEADER: HeaderName = HeaderName::from_static("ocp-apim-subscription-key");

pub struct DocumentIntelligenceClient {
    http_client: HttpClient,
    endpoint: String,
    api_version: ApiVersion,
    auth: Auth,
    config: Config,
}

impl DocumentIntelligenceClient {
    pub fn new(endpoint: String, api_version: ApiVersion, auth: Auth, config: Config) -> Self {
        Self {
            http_client: HttpClient::new(),
            endpoint: endpoint.trim_end_matches('/').to_string(),
            api_version,
            auth,
            config,
        }
    }

    pub fn from_env(config: Option<Config>) -> Result<Self> {
        let endpoint = std::env::var("AZURE_COGNITIVE_SERVICES_ENDPOINT")
            .map_err(|_| Error::Config("AZURE_COGNITIVE_SERVICES_ENDPOINT is not set".into()))?;

        let auth = if let Ok(key) = std::env::var("AZURE_COGNITIVE_SERVICES_KEY") {
            Auth::ApiKey(key)
        } else {
            Auth::Credential(Arc::new(Credential::new(None, None, None).map_err(
                |e| {
                    Error::Auth(format!(
                        "set AZURE_COGNITIVE_SERVICES_KEY or use Entra (az login): {e}"
                    ))
                },
            )?))
        };

        Ok(Self::new(
            endpoint,
            ApiVersion::Default,
            auth,
            config.unwrap_or_default(),
        ))
    }

    pub fn with_api_key(
        endpoint: impl Into<String>,
        api_key: impl Into<String>,
        config: Option<Config>,
    ) -> Self {
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
    ) -> Self {
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
            .await?;

        let status = response.status();
        if status == StatusCode::ACCEPTED {
            return operation_location(response.headers());
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
                .await?;

            let status = response.status();
            if !status.is_success() {
                let message = response.text().await.unwrap_or_default();
                return Err(Error::Api {
                    status: status.as_u16(),
                    message,
                });
            }

            let body: AnalyzeOperationResponse = response.json().await?;

            match body.status.as_str() {
                "succeeded" => return Ok(body),
                "failed" => {
                    let message = body
                        .error
                        .and_then(|e| e.message)
                        .unwrap_or_else(|| "analyze operation failed".into());
                    return Err(Error::AnalyzeFailed(message));
                }
                _ => tokio::time::sleep(self.config.interval()).await,
            }
        }

        Err(Error::PollTimeout {
            attempts: self.config.max_attempts(),
        })
    }
}

impl OCRClient<AnalyzeOperationResponse> for DocumentIntelligenceClient {
    async fn analyze_path(&self, path: &Path) -> OCRResult<AnalyzeOperationResponse> {
        let bytes = std::fs::read(path)?;
        self.analyze_bytes(&bytes).await
    }

    async fn analyze_bytes(&self, bytes: &[u8]) -> OCRResult<AnalyzeOperationResponse> {
        tracing::debug!(
            model_id = PrebuiltModel::Invoice.as_ref(),
            bytes_len = bytes.len(),
            "document intelligence analyze — start"
        );

        let body = json!({ "base64Source": STANDARD.encode(bytes) });
        let operation_url = self
            .start_analyze(PrebuiltModel::Invoice.as_ref(), body)
            .await?;
        let result = self.poll_until_done(&operation_url).await?;

        let page_count = result
            .analyze_result
            .as_ref()
            .map(|r| r.pages.len())
            .unwrap_or(0);
        tracing::debug!(
            model_id = PrebuiltModel::Invoice.as_ref(),
            page_count,
            "document intelligence analyze — done"
        );

        Ok(result)
    }
}

impl From<Error> for OCRError {
    fn from(value: Error) -> Self {
        OCRError::Service(value.to_string())
    }
}

fn operation_location(headers: &HeaderMap) -> Result<String> {
    headers
        .get("operation-location")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .ok_or_else(|| Error::MissingHeader("operation-location".into()))
}
