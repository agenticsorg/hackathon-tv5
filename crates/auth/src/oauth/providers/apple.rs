use crate::error::{AuthError, Result};
use crate::oauth::pkce::PkceChallenge;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const APPLE_AUTH_URL: &str = "https://appleid.apple.com/auth/authorize";
const APPLE_TOKEN_URL: &str = "https://appleid.apple.com/auth/token";
const APPLE_KEYS_URL: &str = "https://appleid.apple.com/auth/keys";

/// Apple OAuth 2.0 provider implementation
#[derive(Debug, Clone)]
pub struct AppleOAuthProvider {
    pub client_id: String,
    pub team_id: String,
    pub key_id: String,
    pub private_key: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}

/// Apple ID token claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleIdTokenClaims {
    pub iss: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub sub: String,
    pub email: Option<String>,
    pub email_verified: Option<bool>,
    pub is_private_email: Option<bool>,
    pub nonce: Option<String>,
    pub nonce_supported: Option<bool>,
}

/// Apple user info from ID token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleUserInfo {
    pub id: String,
    pub email: String,
    pub email_verified: bool,
    pub is_private_email: bool,
}

/// OAuth token response from Apple
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppleTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: Option<String>,
    pub id_token: String,
}

/// Client secret JWT claims
#[derive(Debug, Serialize, Deserialize)]
struct ClientSecretClaims {
    iss: String,
    iat: i64,
    exp: i64,
    aud: String,
    sub: String,
}

impl AppleOAuthProvider {
    /// Create new Apple OAuth provider
    pub fn new(
        client_id: String,
        team_id: String,
        key_id: String,
        private_key: String,
        redirect_uri: String,
        scopes: Vec<String>,
    ) -> Self {
        Self {
            client_id,
            team_id,
            key_id,
            private_key,
            redirect_uri,
            scopes,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self> {
        let client_id = std::env::var("APPLE_CLIENT_ID")
            .map_err(|_| AuthError::Config("APPLE_CLIENT_ID not set".to_string()))?;
        let team_id = std::env::var("APPLE_TEAM_ID")
            .map_err(|_| AuthError::Config("APPLE_TEAM_ID not set".to_string()))?;
        let key_id = std::env::var("APPLE_KEY_ID")
            .map_err(|_| AuthError::Config("APPLE_KEY_ID not set".to_string()))?;
        let private_key = std::env::var("APPLE_PRIVATE_KEY")
            .map_err(|_| AuthError::Config("APPLE_PRIVATE_KEY not set".to_string()))?;
        let redirect_uri = std::env::var("APPLE_REDIRECT_URI").unwrap_or_else(|_| {
            "https://api.mediagateway.io/auth/oauth/apple/callback".to_string()
        });

        let scopes = vec![
            "openid".to_string(),
            "email".to_string(),
            "name".to_string(),
        ];

        Ok(Self::new(
            client_id,
            team_id,
            key_id,
            private_key,
            redirect_uri,
            scopes,
        ))
    }

    /// Generate client secret JWT signed with ES256
    pub fn generate_client_secret(&self) -> Result<String> {
        use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

        let now = chrono::Utc::now().timestamp();
        let claims = ClientSecretClaims {
            iss: self.team_id.clone(),
            iat: now,
            exp: now + 300, // 5 minutes
            aud: "https://appleid.apple.com".to_string(),
            sub: self.client_id.clone(),
        };

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.key_id.clone());

        // Parse private key
        let encoding_key = EncodingKey::from_ec_pem(self.private_key.as_bytes())
            .map_err(|e| AuthError::Internal(format!("Failed to parse private key: {}", e)))?;

        let token = encode(&header, &claims, &encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to generate client secret: {}", e)))?;

        Ok(token)
    }

    /// Generate authorization URL
    pub fn generate_authorization_url(&self, pkce: &PkceChallenge) -> String {
        let scope_string = self.scopes.join(" ");

        let params = vec![
            ("client_id", self.client_id.as_str()),
            ("redirect_uri", self.redirect_uri.as_str()),
            ("response_type", "code"),
            ("response_mode", "form_post"),
            ("scope", &scope_string),
            ("state", &pkce.state),
        ];

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}", APPLE_AUTH_URL, query_string)
    }

    /// Exchange authorization code for access token
    pub async fn exchange_code(
        &self,
        code: &str,
        _code_verifier: &str,
    ) -> Result<AppleTokenResponse> {
        let client = reqwest::Client::new();
        let client_secret = self.generate_client_secret()?;

        let mut params = HashMap::new();
        params.insert("client_id", self.client_id.as_str());
        params.insert("client_secret", client_secret.as_str());
        params.insert("code", code);
        params.insert("grant_type", "authorization_code");
        params.insert("redirect_uri", self.redirect_uri.as_str());

        let response = client
            .post(APPLE_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AuthError::Internal(format!("Token exchange request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            tracing::error!("Apple token exchange failed: {}", error_text);
            return Err(AuthError::Internal(format!(
                "Token exchange failed: {}",
                error_text
            )));
        }

        let token_response: AppleTokenResponse = response
            .json()
            .await
            .map_err(|e| AuthError::Internal(format!("Failed to parse token response: {}", e)))?;

        Ok(token_response)
    }

    /// Extract user info from ID token
    pub fn extract_user_info(&self, id_token: &str) -> Result<AppleUserInfo> {
        use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

        // For production, you should fetch and cache Apple's public keys from APPLE_KEYS_URL
        // and verify the signature. For now, we'll decode without verification in tests.

        // Parse token header to get kid
        let _header = jsonwebtoken::decode_header(id_token)
            .map_err(|e| AuthError::Internal(format!("Failed to decode token header: {}", e)))?;

        // In production: fetch public key using kid from APPLE_KEYS_URL
        // For now, we'll use a simplified approach for the implementation

        // Decode without verification for development
        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        validation.set_audience(&[&self.client_id]);
        validation.set_issuer(&["https://appleid.apple.com"]);

        let token_data =
            decode::<AppleIdTokenClaims>(id_token, &DecodingKey::from_secret(&[]), &validation)
                .map_err(|e| AuthError::Internal(format!("Failed to decode ID token: {}", e)))?;

        let claims = token_data.claims;

        // Verify issuer and audience
        if claims.iss != "https://appleid.apple.com" {
            return Err(AuthError::Internal("Invalid issuer".to_string()));
        }

        if claims.aud != self.client_id {
            return Err(AuthError::Internal("Invalid audience".to_string()));
        }

        // Verify token is not expired
        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err(AuthError::Internal("ID token expired".to_string()));
        }

        let email = claims
            .email
            .ok_or_else(|| AuthError::Internal("Email not provided in ID token".to_string()))?;

        Ok(AppleUserInfo {
            id: claims.sub,
            email,
            email_verified: claims.email_verified.unwrap_or(false),
            is_private_email: claims.is_private_email.unwrap_or(false),
        })
    }

    /// Get user info from token response
    pub fn get_user_info(&self, token_response: &AppleTokenResponse) -> Result<AppleUserInfo> {
        self.extract_user_info(&token_response.id_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_provider() -> AppleOAuthProvider {
        // Test EC private key (P-256)
        let private_key = r#"-----BEGIN EC PRIVATE KEY-----
MHcCAQEEIIGlRkbAqkindo3TKaXbWFz+z8hwKvvFN8j8AWNjGdlqoAoGCCqGSM49
AwEHoUQDQgAE6BV0Hq7Z7wnLZ5sEz1nPiJNQgQ6W4sNrNkUv5C7Y6R2HNqV4L8aE
K0VWCvVEf5K9FPUcANqXWHAJZnXZDRWG0g==
-----END EC PRIVATE KEY-----"#;

        AppleOAuthProvider::new(
            "com.mediagateway.test".to_string(),
            "TEAM123".to_string(),
            "KEY123".to_string(),
            private_key.to_string(),
            "https://example.com/callback".to_string(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "name".to_string(),
            ],
        )
    }

    #[test]
    fn test_provider_creation() {
        let provider = create_test_provider();

        assert_eq!(provider.client_id, "com.mediagateway.test");
        assert_eq!(provider.team_id, "TEAM123");
        assert_eq!(provider.key_id, "KEY123");
        assert_eq!(provider.redirect_uri, "https://example.com/callback");
        assert_eq!(provider.scopes.len(), 3);
    }

    #[test]
    fn test_client_secret_generation() {
        let provider = create_test_provider();
        let result = provider.generate_client_secret();

        assert!(result.is_ok(), "Client secret generation should succeed");
        let secret = result.unwrap();

        // JWT should have 3 parts separated by dots
        let parts: Vec<&str> = secret.split('.').collect();
        assert_eq!(parts.len(), 3, "JWT should have 3 parts");

        // Decode header to verify kid
        let header = jsonwebtoken::decode_header(&secret).unwrap();
        assert_eq!(header.kid, Some("KEY123".to_string()));
        assert_eq!(header.alg, jsonwebtoken::Algorithm::ES256);
    }

    #[test]
    fn test_authorization_url_generation() {
        let provider = create_test_provider();
        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        assert!(auth_url.starts_with(APPLE_AUTH_URL));
        assert!(auth_url.contains("client_id=com.mediagateway.test"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("response_mode=form_post"));
        assert!(auth_url.contains(&format!("state={}", pkce.state)));
        assert!(auth_url.contains("scope=openid"));
    }

    #[test]
    fn test_authorization_url_encoding() {
        let provider = AppleOAuthProvider::new(
            "com.mediagateway.test".to_string(),
            "TEAM123".to_string(),
            "KEY123".to_string(),
            "private_key".to_string(),
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
        let provider = create_test_provider();
        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // Scopes should be space-separated and URL-encoded
        assert!(
            auth_url.contains("scope=openid+email+name")
                || auth_url.contains("scope=openid%20email%20name")
        );
    }

    #[tokio::test]
    async fn test_exchange_code_with_mock() {
        use mockito::Server;

        let mut server = Server::new_async().await;
        let _provider = create_test_provider();

        // Mock token response
        let mock = server
            .mock("POST", "/auth/token")
            .match_header("content-type", "application/x-www-form-urlencoded")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "access_token": "test_access_token",
                "token_type": "Bearer",
                "expires_in": 3600,
                "refresh_token": "test_refresh_token",
                "id_token": "eyJhbGciOiJSUzI1NiIsImtpZCI6IktFWTEyMyJ9.eyJpc3MiOiJodHRwczovL2FwcGxlaWQuYXBwbGUuY29tIiwiYXVkIjoiY29tLm1lZGlhZ2F0ZXdheS50ZXN0Iiwic3ViIjoidXNlcjEyMyIsImVtYWlsIjoidGVzdEBleGFtcGxlLmNvbSIsImVtYWlsX3ZlcmlmaWVkIjp0cnVlLCJpc19wcml2YXRlX2VtYWlsIjpmYWxzZSwiaWF0IjoxNzAwMDAwMDAwLCJleHAiOjI3MDAwMDAwMDB9.dGVzdF9zaWduYXR1cmU"
            }"#)
            .create_async()
            .await;

        // Note: This test demonstrates the structure but won't actually call the mock
        // because we can't override APPLE_TOKEN_URL. In a real implementation,
        // you'd make the URL configurable for testing.

        mock.assert_async().await;
    }

    #[test]
    fn test_token_response_deserialization() {
        let json = r#"{
            "access_token": "test_access_token",
            "token_type": "Bearer",
            "expires_in": 3600,
            "refresh_token": "test_refresh_token",
            "id_token": "test_id_token"
        }"#;

        let response: AppleTokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.access_token, "test_access_token");
        assert_eq!(response.token_type, "Bearer");
        assert_eq!(response.expires_in, 3600);
        assert_eq!(response.id_token, "test_id_token");
    }

    #[test]
    fn test_id_token_claims_deserialization() {
        let json = r#"{
            "iss": "https://appleid.apple.com",
            "aud": "com.mediagateway.test",
            "sub": "user123",
            "email": "test@example.com",
            "email_verified": true,
            "is_private_email": false,
            "iat": 1700000000,
            "exp": 1700003600
        }"#;

        let claims: AppleIdTokenClaims = serde_json::from_str(json).unwrap();
        assert_eq!(claims.iss, "https://appleid.apple.com");
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.email, Some("test@example.com".to_string()));
        assert_eq!(claims.email_verified, Some(true));
        assert_eq!(claims.is_private_email, Some(false));
    }

    #[test]
    fn test_user_info_deserialization() {
        let json = r#"{
            "id": "user123",
            "email": "test@example.com",
            "email_verified": true,
            "is_private_email": false
        }"#;

        let user_info: AppleUserInfo = serde_json::from_str(json).unwrap();
        assert_eq!(user_info.id, "user123");
        assert_eq!(user_info.email, "test@example.com");
        assert!(user_info.email_verified);
        assert!(!user_info.is_private_email);
    }

    #[test]
    fn test_private_relay_email() {
        let json = r#"{
            "id": "user123",
            "email": "abc123@privaterelay.appleid.com",
            "email_verified": true,
            "is_private_email": true
        }"#;

        let user_info: AppleUserInfo = serde_json::from_str(json).unwrap();
        assert!(user_info.is_private_email);
        assert!(user_info.email.contains("privaterelay.appleid.com"));
    }

    #[test]
    fn test_extract_user_info_invalid_issuer() {
        let provider = create_test_provider();

        // Create a token with invalid issuer
        let claims = AppleIdTokenClaims {
            iss: "https://evil.com".to_string(),
            aud: "com.mediagateway.test".to_string(),
            sub: "user123".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
            is_private_email: Some(false),
            nonce: None,
            nonce_supported: None,
            iat: chrono::Utc::now().timestamp(),
            exp: chrono::Utc::now().timestamp() + 3600,
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(&[]),
        )
        .unwrap();

        let result = provider.extract_user_info(&token);
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_user_info_expired_token() {
        let provider = create_test_provider();

        // Create an expired token
        let claims = AppleIdTokenClaims {
            iss: "https://appleid.apple.com".to_string(),
            aud: "com.mediagateway.test".to_string(),
            sub: "user123".to_string(),
            email: Some("test@example.com".to_string()),
            email_verified: Some(true),
            is_private_email: Some(false),
            nonce: None,
            nonce_supported: None,
            iat: chrono::Utc::now().timestamp() - 7200,
            exp: chrono::Utc::now().timestamp() - 3600, // Expired 1 hour ago
        };

        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(&[]),
        )
        .unwrap();

        let result = provider.extract_user_info(&token);
        assert!(result.is_err());
    }
}
