use std::sync::Arc;
use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use crate::error::{Result, Error};

#[derive(Clone)]
pub enum Auth {
    ApiKey(String),
    Credential(Arc<Credential>),
}

#[derive(Debug)]
pub struct Credential(Arc<dyn TokenCredential>);

impl Credential {
    pub fn new(
        tenant_id: Option<String>,
        client_id: Option<String>,
        secret: Option<String>,
    ) -> Result<Self> {
        if let (Some(t), Some(c), Some(s)) = (tenant_id, client_id, secret) {
            return Ok(Self(
                azure_identity::ClientSecretCredential::new(&t, c, s.into(), None).map_err(
                    |_| Error::Azure("Unable to authenticate using client secret.".into()),
                )?,
            ));
        }

        match azure_identity::DeveloperToolsCredential::new(None) {
            Ok(tc) => {
                tracing::debug!("AzureCredential: using DeveloperToolsCredential");
                return Ok(Self(tc));
            }
            Err(e) => {
                tracing::debug!("AzureCredential: DeveloperToolsCredential unavailable: {e:?}");
            }
        }

        Err(Error::Azure(
            "Unable to authenticate (tried developer tools). Use az login or set tenant/client/secret."
                .into(),
        ))
    }

    pub fn token_credential(&self) -> Arc<dyn TokenCredential> {
        Arc::clone(&self.0)
    }

    pub async fn get_token(
        &self,
        auth_scopes: &[&str],
        options: Option<TokenRequestOptions<'_>>,
    ) -> Result<AccessToken> {
        self.0
            .get_token(auth_scopes, options)
            .await
            .map_err(|e| Error::Azure(format!("Could not retrieve token: {e}")))
    }
}