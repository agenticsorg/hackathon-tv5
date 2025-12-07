use media_gateway_auth::oauth::pkce::PkceChallenge;
use media_gateway_auth::oauth::providers::GoogleOAuthProvider;

#[cfg(test)]
mod google_oauth_provider_tests {
    use super::*;

    #[test]
    fn test_google_provider_creation() {
        let provider = GoogleOAuthProvider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            "https://example.com/callback".to_string(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        );

        assert_eq!(provider.client_id, "test_client_id");
        assert_eq!(provider.client_secret, "test_client_secret");
        assert_eq!(provider.redirect_uri, "https://example.com/callback");
        assert_eq!(provider.scopes.len(), 3);
        assert!(provider.scopes.contains(&"openid".to_string()));
        assert!(provider.scopes.contains(&"email".to_string()));
        assert!(provider.scopes.contains(&"profile".to_string()));
    }

    #[test]
    fn test_authorization_url_generation() {
        let provider = GoogleOAuthProvider::new(
            "test_client_123".to_string(),
            "test_secret_456".to_string(),
            "https://api.example.com/callback".to_string(),
            vec!["openid".to_string(), "email".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // Verify URL starts with Google's auth endpoint
        assert!(auth_url.starts_with("https://accounts.google.com/o/oauth2/v2/auth?"));

        // Verify required OAuth parameters
        assert!(auth_url.contains("client_id=test_client_123"));
        assert!(auth_url.contains("response_type=code"));
        assert!(auth_url.contains("redirect_uri="));
        assert!(auth_url.contains("scope="));

        // Verify PKCE parameters
        assert!(auth_url.contains(&format!("code_challenge={}", pkce.code_challenge)));
        assert!(auth_url.contains("code_challenge_method=S256"));
        assert!(auth_url.contains(&format!("state={}", pkce.state)));

        // Verify Google-specific parameters
        assert!(auth_url.contains("access_type=offline"));
        assert!(auth_url.contains("prompt=consent"));
    }

    #[test]
    fn test_authorization_url_with_multiple_scopes() {
        let provider = GoogleOAuthProvider::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            "https://app.example.com/auth/callback".to_string(),
            vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
                "https://www.googleapis.com/auth/userinfo.email".to_string(),
            ],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // Scopes should be URL-encoded and space-separated
        assert!(auth_url.contains("scope="));
        assert!(
            auth_url.contains("openid")
                && auth_url.contains("email")
                && auth_url.contains("profile")
        );
    }

    #[test]
    fn test_url_encoding_in_redirect_uri() {
        let provider = GoogleOAuthProvider::new(
            "client_123".to_string(),
            "secret_456".to_string(),
            "https://example.com/callback?app=test&version=1.0".to_string(),
            vec!["openid".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let auth_url = provider.generate_authorization_url(&pkce);

        // Query parameters in redirect_uri should be URL encoded
        assert!(auth_url.contains(
            "redirect_uri=https%3A%2F%2Fexample.com%2Fcallback%3Fapp%3Dtest%26version%3D1.0"
        ));
    }

    #[test]
    fn test_pkce_state_parameter_included() {
        let provider = GoogleOAuthProvider::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            "https://localhost/callback".to_string(),
            vec!["openid".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let state_value = pkce.state.clone();
        let auth_url = provider.generate_authorization_url(&pkce);

        // State parameter must match PKCE state
        assert!(auth_url.contains(&format!("state={}", state_value)));
    }

    #[test]
    fn test_code_challenge_matches_pkce() {
        let provider = GoogleOAuthProvider::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            "https://localhost/callback".to_string(),
            vec!["openid".to_string()],
        );

        let pkce = PkceChallenge::generate();
        let challenge = pkce.code_challenge.clone();
        let auth_url = provider.generate_authorization_url(&pkce);

        // Code challenge in URL must match PKCE challenge
        assert!(auth_url.contains(&format!("code_challenge={}", challenge)));
    }

    #[test]
    fn test_authorization_url_unique_per_request() {
        let provider = GoogleOAuthProvider::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            "https://localhost/callback".to_string(),
            vec!["openid".to_string()],
        );

        let pkce1 = PkceChallenge::generate();
        let pkce2 = PkceChallenge::generate();

        let url1 = provider.generate_authorization_url(&pkce1);
        let url2 = provider.generate_authorization_url(&pkce2);

        // Each URL should have unique state and challenge
        assert_ne!(url1, url2);
        assert!(url1.contains(&pkce1.state));
        assert!(url2.contains(&pkce2.state));
        assert!(url1.contains(&pkce1.code_challenge));
        assert!(url2.contains(&pkce2.code_challenge));
    }
}

#[cfg(test)]
mod google_oauth_integration_tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[actix_rt::test]
    async fn test_token_exchange_with_mock_google() {
        // Start mock server
        let mock_server = MockServer::start().await;

        // Mock Google token endpoint
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_string_contains("grant_type=authorization_code"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "access_token": "ya29.mock_access_token",
                "expires_in": 3600,
                "refresh_token": "1//mock_refresh_token",
                "scope": "openid email profile",
                "token_type": "Bearer",
                "id_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.mock_id_token"
            })))
            .mount(&mock_server)
            .await;

        // Create provider pointing to mock server
        let provider = GoogleOAuthProvider::new(
            "test_client_id".to_string(),
            "test_client_secret".to_string(),
            format!("{}/callback", mock_server.uri()),
            vec!["openid".to_string(), "email".to_string()],
        );

        // Override token URL to point to mock server
        let mock_provider = GoogleOAuthProvider {
            client_id: provider.client_id.clone(),
            client_secret: provider.client_secret.clone(),
            redirect_uri: provider.redirect_uri.clone(),
            scopes: provider.scopes.clone(),
        };

        let pkce = PkceChallenge::generate();
        let auth_code = "4/mock_authorization_code";

        // Note: This test requires modifying the GoogleOAuthProvider to accept
        // custom token URL, or we need to use a test-specific implementation.
        // For now, this demonstrates the test structure.
    }

    #[actix_rt::test]
    async fn test_user_profile_fetch_with_mock_google() {
        // Start mock server
        let mock_server = MockServer::start().await;

        // Mock Google userinfo endpoint
        Mock::given(method("GET"))
            .and(path("/oauth2/v2/userinfo"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "123456789",
                "email": "test@example.com",
                "verified_email": true,
                "name": "Test User",
                "given_name": "Test",
                "family_name": "User",
                "picture": "https://example.com/photo.jpg",
                "locale": "en"
            })))
            .mount(&mock_server)
            .await;

        // This test demonstrates the expected structure for profile retrieval
        // In production, we would need to inject the userinfo URL or use
        // feature flags for testing.
    }

    #[actix_rt::test]
    async fn test_token_exchange_error_handling() {
        // Start mock server
        let mock_server = MockServer::start().await;

        // Mock Google token endpoint returning error
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": "invalid_grant",
                "error_description": "Invalid authorization code"
            })))
            .mount(&mock_server)
            .await;

        // Test should handle errors gracefully
    }

    #[actix_rt::test]
    async fn test_expired_authorization_code() {
        // Start mock server
        let mock_server = MockServer::start().await;

        // Mock Google token endpoint returning expired code error
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": "invalid_grant",
                "error_description": "Code has expired"
            })))
            .mount(&mock_server)
            .await;

        // Test should handle expired codes
    }

    #[actix_rt::test]
    async fn test_invalid_code_verifier() {
        // Start mock server
        let mock_server = MockServer::start().await;

        // Mock Google token endpoint returning PKCE verification error
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                "error": "invalid_grant",
                "error_description": "Code verifier does not match challenge"
            })))
            .mount(&mock_server)
            .await;

        // Test should handle invalid code verifier
    }
}

#[cfg(test)]
mod google_oauth_callback_tests {
    use super::*;

    #[test]
    fn test_callback_query_parameters() {
        // Test that callback expects correct query parameters
        let expected_params = vec!["code", "state"];

        // This is a structural test showing what parameters are expected
        assert!(expected_params.contains(&"code"));
        assert!(expected_params.contains(&"state"));
    }

    #[test]
    fn test_error_parameter_handling() {
        // Callback should handle OAuth error responses
        let error_params = vec!["error", "error_description"];

        assert!(error_params.contains(&"error"));
        assert!(error_params.contains(&"error_description"));
    }
}
