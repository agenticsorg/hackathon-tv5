pub mod device;
pub mod handlers;
pub mod pkce;
pub mod providers;

use crate::error::{AuthError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub providers: HashMap<String, OAuthProvider>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    pub client_id: String,
    pub client_secret: String,
    pub authorization_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

pub struct OAuthManager {
    config: OAuthConfig,
}

impl OAuthManager {
    pub fn new(config: OAuthConfig) -> Self {
        Self { config }
    }

    pub fn get_provider(&self, name: &str) -> Result<&OAuthProvider> {
        self.config
            .providers
            .get(name)
            .ok_or_else(|| AuthError::InvalidClient)
    }

    pub fn validate_redirect_uri(&self, provider_name: &str, redirect_uri: &str) -> Result<()> {
        let provider = self.get_provider(provider_name)?;
        if provider.redirect_uri != redirect_uri {
            return Err(AuthError::InvalidRedirectUri);
        }
        Ok(())
    }

    pub fn validate_scopes(&self, provider_name: &str, scopes: &[String]) -> Result<()> {
        let provider = self.get_provider(provider_name)?;
        for scope in scopes {
            if !provider.scopes.contains(scope) {
                return Err(AuthError::InvalidScope(scope.clone()));
            }
        }
        Ok(())
    }
}
