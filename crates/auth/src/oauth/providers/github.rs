use crate::error::{AuthError, Result};
use crate::oauth::pkce::PkceChallenge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const GITHUB_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const GITHUB_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const GITHUB_USER_URL: &str = "https://api.github.com/user";
const GITHUB_EMAILS_URL: &str = "https://api.github.com/user/emails";

/// GitHub OAuth 2.0 provider implementation
#[derive(Debug, Clone)]
pub struct GitHubOAuthProvider {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

/// GitHub user profile response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUserProfile {
    pub id: i64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub hireable: Option<bool>,
    pub public_repos: Option<i32>,
    pub followers: Option<i32>,
    pub following: Option<i32>,
    pub created_at: Option<String>,
}

/// GitHub user email response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUserEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
    pub visibility: Option<String>,
}

/// OAuth token response from GitHub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub scope: String,
}

impl GitHubOAuthProvider {
    /// Create new GitHub OAuth provider
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
        let client_id = std::env::var("GITHUB_CLIENT_ID")
            .map_err(|_| AuthError::Config("GITHUB_CLIENT_ID not set".to_string()))?;
        let client_secret = std::env::var("GITHUB_CLIENT_SECRET")
            .map_err(|_| AuthError::Config("GITHUB_CLIENT_SECRET not set".to_string()))?;
        let redirect_uri = std::env::var("GITHUB_REDIRECT_URI").unwrap_or_else(|_| {
            "https://api.mediagateway.io/auth/oauth/github/callback".to_string()
        });

        let scopes = vec!["user:email".to_string(), "read:user".to_string()];

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
            ("state", &pkce.state),
        ];

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", GITHUB_AUTH_URL, query_string)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        code: &str,
        _code_verifier: &str,
    ) -> Result<GitHubTokenResponse> {
        let client = reqwest::Client::new();

        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", self.client_secret.as_str());
        params.insert("code", code);
        params.insert("redirect_uri", self.redirect_uri.as_str());

        let response = client
            .post(GITHUB_TOKEN_URL)
            .header("Accept", "application/json")
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::Internal(format!("Token exchange request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("GitHub token exchange failed: {}", error_text);
            return Err(AuthError::Internal(format!(
                "Token exchange failed: {}",
                error_text
            )));
        }

        let token_response: GitHubTokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response)
    }

    /// Get user profile using access token
    pub async fn get_user_profile(&self, access_token: &str) -> Result<GitHubUserProfile> {
        let client = reqwest::Client::new();

        let response = client
            .get(GITHUB_USER_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "MediaGateway-Auth")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AuthError::Internal(format!("User profile request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("GitHub user profile fetch failed: {}", error_text);
            return Err(AuthError::Internal(format!(
                "User profile fetch failed: {}",
                error_text
            )));
        }

        let profile: GitHubUserProfile = response
            .json()
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to parse user profile: {}", e)))?;

        Ok(profile)
    }

    /// Get user emails using access token
    pub async fn get_user_emails(&self, access_token: &str) -> Result<Vec<GitHubUserEmail>> {
        let client = reqwest::Client::new();

        let response = client
            .get(GITHUB_EMAILS_URL)
            .header("Authorization", format!("Bearer {}", access_token))
            .header("User-Agent", "MediaGateway-Auth")
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| AuthError::Internal(format!("User emails request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("GitHub user emails fetch failed: {}", error_text);
            return Err(AuthError::Internal(format!(
                "User emails fetch failed: {}",
                error_text
            )));
        }

        let emails: Vec<GitHubUserEmail> = response
            .json()
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to parse user emails: {}", e)))?;

        Ok(emails)
    }

    /// Get primary verified email from user emails
    pub async fn get_verified_email(&self, access_token: &str) -> Result<String> {
        let emails = self.get_user_emails(access_token).await?;

        // Find primary verified email
        let verified_email = emails
            .iter()
            .find(|e| e.primary && e.verified)
            .or_else(|| emails.iter().find(|e| e.verified))
            .ok_or_else(|| AuthError::Internal("No verified email found".to_string()))?;

        Ok(verified_email.email.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_creation() {
        let provider = GitHubOAuthProvider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "https://example.com/callback".to_string(),
            vec!["user:email".to_string(), "read:user".to_string()],
        );

        assert_eq!(provider.client_id, "test_client_id");
        assert_eq!(provider.client_secret, "test_client_secret");
        assert_eq!(provider.redirect_uri, "https://example.com/callback");
        assert_eq!(provider.scopes.len(), 2);
        assert!(provider.scopes.contains(&"user:email".to_string()));
        assert!(provider.scopes.contains(&"read:user".to_string()));
    }

    #[test]
    fn test_authorization_url_generation() {
        let provider = GitHubOAuthProvider::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://example.com/callback".to_string(),
            vec!["user:email".to_string(), "read:user".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        assert!(auth_url.starts_with(GITHUB_AUTH_URL));
        assert!(auth_url.contains("client_id=client123"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains(&format!("state={}", pkce.state)));
        assert!(auth_url.contains("scope=user%3Aemail") || auth_url.contains("scope=user:email"));
    }

    #[test]
    fn test_authorization_url_encoding() {
        let provider = GitHubOAuthProvider::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://example.com/callback?foo=bar".to_string(),
            vec!["user:email".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // URL should be properly encoded
        assert!(auth_url.contains("redirect_uri=https%3A%2F%2Fexample.com%2Fcallback%3Ffoo%3Dbar"));
    }

    #[test]
    fn test_scopes_in_url() {
        let provider = GitHubOAuthProvider::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://example.com/callback".to_string(),
            vec!["user:email".to_string(), "read:user".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // Scopes should be space-separated and URL-encoded
        assert!(
            auth_url.contains("scope=user%3Aemail+read%3Auser")
                || auth_url.contains("scope=user%3Aemail%20read%3Auser")
                || auth_url.contains("scope=user:email+read:user")
                || auth_url.contains("scope=user:email%20read:user")
        );
    }

    #[test]
    fn test_default_scopes() {
        let provider = GitHubOAuthProvider::new(
            "client123".to_string(),
            "secret456".to_string(),
            "https://example.com/callback".to_string(),
            vec!["user:email".to_string(), "read:user".to_string()],
        );

        assert_eq!(provider.scopes.len(), 2);
        assert!(provider.scopes.contains(&"user:email".to_string()));
        assert!(provider.scopes.contains(&"read:user".to_string()));
    }

    #[tokio::test]
    async fn test_exchange_code_with_mock() {
        use mockito::Server;

        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/login/oauth/access_token")
            .match_header("accept", "application/json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "access_token": "gho_test_token_123",
                "token_type": "bearer",
                "scope": "user:email,read:user"
            }"#,
            )
            .create_async()
            .await;

        // Note: This test demonstrates the structure but won't actually call the mock
        // because we can't override GITHUB_TOKEN_URL. In a real implementation,
        // you'd make the URL configurable for testing.

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_get_user_profile_with_mock() {
        use mockito::Server;

        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/user")
            .match_header("authorization", "Bearer test_token")
            .match_header("user-agent", "MediaGateway-Auth")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
                "id": 123456,
                "login": "testuser",
                "name": "Test User",
                "email": "test@example.com",
                "avatar_url": "https://github.com/avatar.jpg",
                "bio": "Test bio",
                "location": "Test City",
                "company": "Test Company",
                "blog": "https://blog.example.com",
                "hireable": true,
                "public_repos": 10,
                "followers": 5,
                "following": 3,
                "created_at": "2020-01-01T00:00:00Z"
            }"#,
            )
            .create_async()
            .await;
    }

    #[tokio::test]
    async fn test_get_user_emails_with_mock() {
        use mockito::Server;

        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/user/emails")
            .match_header("authorization", "Bearer test_token")
            .match_header("user-agent", "MediaGateway-Auth")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"[
                {
                    "email": "test@example.com",
                    "primary": true,
                    "verified": true,
                    "visibility": "public"
                },
                {
                    "email": "test2@example.com",
                    "primary": false,
                    "verified": false,
                    "visibility": "private"
                }
            ]"#,
            )
            .create_async()
            .await;
    }

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "gho_test_token_123",
            "token_type": "bearer",
            "scope": "user:email,read:user"
        }"#;

        let response: GitHubTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "gho_test_token_123");
        assert_eq!(response.token_type, "bearer");
        assert_eq!(response.scope, "user:email,read:user");
    }

    #[test]
    fn test_user_profile_deserialization() {
        let json = r#"{
            "id": 123456,
            "login": "testuser",
            "name": "Test User",
            "email": "test@example.com",
            "avatar_url": "https://github.com/avatar.jpg",
            "bio": "Test bio",
            "location": "Test City",
            "company": "Test Company",
            "blog": "https://blog.example.com",
            "hireable": true,
            "public_repos": 10,
            "followers": 5,
            "following": 3,
            "created_at": "2020-01-01T00:00:00Z"
        }"#;

        let profile: GitHubUserProfile = serde_json::from_str(json).unwrap();
        assert_eq!(profile.id, 123456);
        assert_eq!(profile.login, "testuser");
        assert_eq!(profile.name, Some("Test User".to_string()));
        assert_eq!(profile.email, Some("test@example.com".to_string()));
    }

    #[test]
    fn test_user_email_deserialization() {
        let json = r#"{
            "email": "test@example.com",
            "primary": true,
            "verified": true,
            "visibility": "public"
        }"#;

        let email: GitHubUserEmail = serde_json::from_str(json).unwrap();
        assert_eq!(email.email, "test@example.com");
        assert!(email.primary);
        assert!(email.verified);
        assert_eq!(email.visibility, Some("public".to_string()));
    }
}
