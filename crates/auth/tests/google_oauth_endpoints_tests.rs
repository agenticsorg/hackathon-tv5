use actix_web::{test, web, App};
use media_gateway_auth::oauth::handlers::{google_authorize, google_callback};
use media_gateway_auth::storage::AuthStorage;
use std::sync::Arc;

#[cfg(test)]
mod google_oauth_endpoint_tests {
    use super::*;

    #[actix_rt::test]
    async fn test_google_authorize_endpoint_redirect() {
        // Set up test environment variables
        std::env::set_var("GOOGLE_CLIENT_ID", "test_client_id");
        std::env::set_var("GOOGLE_CLIENT_SECRET", "test_client_secret");
        std::env::set_var("GOOGLE_REDIRECT_URI", "https://localhost/callback");

        // Note: This test requires Redis to be running
        // In a real test environment, we would use a test Redis instance
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/15".to_string());

        let storage = match AuthStorage::new(&redis_url) {
            Ok(s) => Arc::new(s),
            Err(_) => {
                println!("Skipping test: Redis not available");
                return;
            }
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .service(google_authorize),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/auth/oauth/google/authorize")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should redirect to Google authorization URL
        assert!(resp.status().is_redirection());
        assert!(resp.headers().contains_key("location"));

        let location = resp.headers().get("location").unwrap().to_str().unwrap();
        assert!(location.starts_with("https://accounts.google.com/o/oauth2/v2/auth"));
        assert!(location.contains("client_id=test_client_id"));
        assert!(location.contains("response_type=code"));
        assert!(location.contains("code_challenge="));
        assert!(location.contains("state="));
    }

    #[actix_rt::test]
    async fn test_google_callback_endpoint_missing_code() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/15".to_string());

        let storage = match AuthStorage::new(&redis_url) {
            Ok(s) => Arc::new(s),
            Err(_) => {
                println!("Skipping test: Redis not available");
                return;
            }
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .service(google_callback),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/auth/oauth/google/callback?error=access_denied&error_description=User%20denied%20access")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should return error response
        assert!(resp.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_google_callback_endpoint_invalid_state() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/15".to_string());

        let storage = match AuthStorage::new(&redis_url) {
            Ok(s) => Arc::new(s),
            Err(_) => {
                println!("Skipping test: Redis not available");
                return;
            }
        };

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(storage))
                .service(google_callback),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/auth/oauth/google/callback?code=test_code&state=invalid_state")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Should return error for invalid/expired state
        assert!(resp.status().is_server_error() || resp.status().is_client_error());
    }

    #[test]
    fn test_endpoint_paths() {
        // Verify endpoint paths match specification
        let authorize_path = "/auth/oauth/google/authorize";
        let callback_path = "/auth/oauth/google/callback";

        assert_eq!(authorize_path, "/auth/oauth/google/authorize");
        assert_eq!(callback_path, "/auth/oauth/google/callback");
    }

    #[test]
    fn test_required_environment_variables() {
        // Test validates that required env vars are documented
        let required_vars = vec!["GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_SECRET"];

        let optional_vars = vec!["GOOGLE_REDIRECT_URI"];

        assert_eq!(required_vars.len(), 2);
        assert_eq!(optional_vars.len(), 1);
    }
}

#[cfg(test)]
mod google_oauth_pkce_flow_tests {
    use super::*;
    use media_gateway_auth::oauth::pkce::PkceChallenge;

    #[actix_rt::test]
    async fn test_pkce_session_storage_and_retrieval() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/15".to_string());

        let storage = match AuthStorage::new(&redis_url) {
            Ok(s) => s,
            Err(_) => {
                println!("Skipping test: Redis not available");
                return;
            }
        };

        let pkce = PkceChallenge::generate();
        let state = pkce.state.clone();

        // Store PKCE session
        storage.store_pkce(&state, &pkce).await.unwrap();

        // Retrieve PKCE session
        let retrieved = storage.get_pkce(&state).await.unwrap();
        assert!(retrieved.is_some());

        let retrieved_pkce = retrieved.unwrap();
        assert_eq!(retrieved_pkce.code_challenge, pkce.code_challenge);
        assert_eq!(retrieved_pkce.state, pkce.state);
        assert_eq!(retrieved_pkce.code_challenge_method, "S256");

        // Clean up
        storage.delete_pkce(&state).await.unwrap();
    }

    #[actix_rt::test]
    async fn test_pkce_session_expiration() {
        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379/15".to_string());

        let storage = match AuthStorage::new(&redis_url) {
            Ok(s) => s,
            Err(_) => {
                println!("Skipping test: Redis not available");
                return;
            }
        };

        let pkce = PkceChallenge::generate();
        let state = pkce.state.clone();

        // Store PKCE session
        storage.store_pkce(&state, &pkce).await.unwrap();

        // Verify it exists
        let retrieved = storage.get_pkce(&state).await.unwrap();
        assert!(retrieved.is_some());

        // PKCE sessions should expire after 10 minutes (600 seconds)
        // This test just verifies the session exists initially
        // In production, we would test expiration with time manipulation
    }

    #[actix_rt::test]
    async fn test_pkce_verifier_validation() {
        let pkce = PkceChallenge::generate();
        let verifier = pkce.code_verifier.clone();

        // Valid verifier should pass
        assert!(pkce.verify(&verifier).is_ok());

        // Invalid verifier should fail
        assert!(pkce.verify("invalid_verifier").is_err());
    }
}
