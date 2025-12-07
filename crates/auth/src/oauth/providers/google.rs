use crate::error::{AuthError, Result};
use crate::oauth::pkce::PkceChallenge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";

/// Google OAuth 2.0 provider implementation
#[derive(Debug, Clone)]
pub struct GoogleOAuthProvider {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

/// Google user profile response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserProfile {
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
}

/// OAuth token response from Google
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub scope: String,
    pub token_type: String,
    pub id_token: Option<String>,
}

impl GoogleOAuthProvider {
    /// Create new Google OAuth provider
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_uri,
            scopes,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .map_err(|_| AuthError::Config("GOOGLE_CLIENT_ID not set".to_string()))?;
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .map_err(|_| AuthError::Config("GOOGLE_CLIENT_SECRET not set".to_string()))?;
        let redirect_uri = std::env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| {
            "https://api.mediagateway.io/auth/oauth/google/callback".to_string()
        });

        let scopes = vec![
            "openid".to_string(),
            "email".to_string(),
            "profile".to_string(),
        ];

        Ok(Self::new(client_id, client_secret, redirect_uri, scopes))
    }

    /// Generate authorization URL with PKCE
    pub fn generate_authorization_url(&self, pkce: &PkceChallenge) -> String {
        let scope_string = self.scopes.join(" ");

        let params = vec![
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", &scope_string),
            ("code_challenge", &pkce.code_challenge),
            ("code_challenge_method", &pkce.code_challenge_method),
            ("state", &pkce.state),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ];

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", GOOGLE_AUTH_URL, query_string)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> Result<GoogleTokenResponse> {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.as_str());
        params.insert("code", code);
        params.insert("code_verifier", code_verifier);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", self.redirect_uri.as_str());

        let response = client
            .post(GOOGLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::Internal(format!("Token exchange request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Google token exchange failed: {}", error_text);
            return Err(AuthError::Internal(format!(
                "Token exchange failed: {}",
                error_text
            )));
        }

        let token_response: GoogleTokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response)
    }

    /// Get user profile using access token
    pub async fn get_user_profile(&self, access_token: &str) -> Result<GoogleUserProfile> {
        let client = reqwest::Client::new();

        let response = client
            .get(GOOGLE_USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| AuthError::Internal(format!("User profile request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Google user profile fetch failed: {}", error_text);
            return Err(AuthError::Internal(format!(
                "User profile fetch failed: {}",
                error_text
            )));
        }

        let profile: GoogleUserProfile = response
            .json()
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to parse user profile: {}", e)))?;

        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = GoogleOAuthProvider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "https://example.com/callback".to_string(),
            vec!["openid".to_string(), "email".to_string()],
        );

        assert_eq!(provider.client_id, "test_client_id");
        assert_eq!(provider.client_secret, "test_client_secret");
        assert_eq!(provider.redirect_uri, "https://example.com/callback");
        assert_eq!(provider.scopes.len(), 2);
    }

    #[test]
    fn test_authorization_url_generation() {
        let provider = GoogleOAuthProvider::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://example.com/callback".to_string(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        assert!(auth_url.starts_with(GOOGLE_AUTH_URL));
        assert!(auth_url.contains("client_id=client123"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("code_challenge="));
        assert!(auth_url.contains("code_challenge_method=S256"));
        assert!(auth_url.contains(&format!("state={}", pkce.state)));
        assert!(auth_url.contains("scope=openid"));
        assert!(auth_url.contains("access_type=offline"));
        assert!(auth_url.contains("prompt=consent"));
    }

    #[test]
    fn test_authorization_url_encoding() {
        let provider = GoogleOAuthProvider::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://example.com/callback?foo=bar".to_string(),
            vec!["openid".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // URL should be properly encoded
        assert!(auth_url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback%3Ffoo%3Dbar"));
    }

    #[test]
    fn test_scopes_in_url() {
        let provider = GoogleOAuthProvider::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://example.com/callback".to_string(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // Scopes should be space-separated and URL-encoded
        assert!(
            auth_url.contains("scope=openid+email+profile")
                || auth_url.contains("scope=openid%20email%20profile")
        );
    }
}
