use actix_web::{test, App};
use media_gateway_auth::{
    jwt::JwtManager, mfa::MfaManager, server::AppState, session::SessionManager,
    storage::AuthStorage, token_family::TokenFamilyManager,
};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;

async fn setup_test_app(
    pool: PgPool,
) -> (
    actix_web::dev::Service<actix_http::Request, actix_web::dev::ServiceResponse, actix_web::Error>,
    Arc<JwtManager>,
) {
    let jwt_manager = Arc::new(JwtManager::new("test-secret".to_string(), "MediaGateway"));
    let session_manager = Arc::new(SessionManager::new(
        AuthStorage::new("redis://127.0.0.1:6379").unwrap(),
    ));
    let token_family_manager = Arc::new(TokenFamilyManager::new(
        AuthStorage::new("redis://127.0.0.1:6379").unwrap(),
    ));
    let storage = Arc::new(AuthStorage::new("redis://127.0.0.1:6379").unwrap());
    let mfa_manager = Arc::new(MfaManager::new(pool.clone(), &[42u8; 32]));

    let app_state = actix_web::web::Data::new(AppState {
        jwt_manager: jwt_manager.clone(),
        session_manager,
        oauth_manager: Arc::new(media_gateway_auth::oauth::OAuthManager::new(
            media_gateway_auth::oauth::OAuthConfig {
                google_client_id: "test-client-id".to_string(),
                google_client_secret: "test-client-secret".to_string(),
                google_redirect_uri: "http://localhost:8080/auth/google/callback".to_string(),
                allowed_redirect_uris: vec!["http://localhost:3000/callback".to_string()],
            },
        )),
        rbac_manager: Arc::new(media_gateway_auth::rbac::RbacManager::new()),
        scope_manager: Arc::new(media_gateway_auth::scopes::ScopeManager::new()),
        storage,
        token_family_manager,
        mfa_manager: Some(mfa_manager),
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route(
                "/api/v1/auth/mfa/enroll",
                actix_web::web::post().to(media_gateway_auth::server::mfa_enroll),
            )
            .route(
                "/api/v1/auth/mfa/verify",
                actix_web::web::post().to(media_gateway_auth::server::mfa_verify),
            )
            .route(
                "/api/v1/auth/mfa/challenge",
                actix_web::web::post().to(media_gateway_auth::server::mfa_challenge),
            ),
    )
    .await;

    (app, jwt_manager)
}

#[sqlx::test]
async fn test_mfa_enroll_endpoint(pool: PgPool) -> sqlx::Result<()> {
    let (app, jwt_manager) = setup_test_app(pool).await;

    // Create test JWT
    let access_token = jwt_manager
        .create_access_token(
            "test_user".to_string(),
            Some("test@example.com".to_string()),
            vec!["user".to_string()],
            vec!["profile".to_string()],
        )
        .unwrap();

    // Make enrollment request
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/mfa/enroll")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["qr_code"].as_str().is_some());
    assert!(body["backup_codes"].as_array().is_some());
    assert_eq!(body["backup_codes"].as_array().unwrap().len(), 10);

    Ok(())
}

#[sqlx::test]
async fn test_mfa_verify_endpoint_success(pool: PgPool) -> sqlx::Result<()> {
    let (app, jwt_manager) = setup_test_app(pool.clone()).await;

    // Setup: Enroll user first
    let user_id = "test_user_verify";
    let mfa_manager = MfaManager::new(pool.clone(), &[42u8; 32]);
    let (secret, _qr, _codes) = mfa_manager
        .initiate_enrollment(user_id.to_string())
        .await
        .unwrap();

    // Generate valid code
    let totp_manager = media_gateway_auth::mfa::totp::TotpManager::new(&[42u8; 32]);
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    // Create JWT
    let access_token = jwt_manager
        .create_access_token(
            user_id.to_string(),
            Some("test@example.com".to_string()),
            vec!["user".to_string()],
            vec!["profile".to_string()],
        )
        .unwrap();

    // Make verify request
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/mfa/verify")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(json!({ "code": valid_code }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["message"].as_str().unwrap(),
        "MFA enrollment verified successfully"
    );

    Ok(())
}

#[sqlx::test]
async fn test_mfa_verify_endpoint_invalid_code(pool: PgPool) -> sqlx::Result<()> {
    let (app, jwt_manager) = setup_test_app(pool.clone()).await;

    // Setup: Enroll user first
    let user_id = "test_user_invalid";
    let mfa_manager = MfaManager::new(pool.clone(), &[42u8; 32]);
    let (_secret, _qr, _codes) = mfa_manager
        .initiate_enrollment(user_id.to_string())
        .await
        .unwrap();

    // Create JWT
    let access_token = jwt_manager
        .create_access_token(
            user_id.to_string(),
            Some("test@example.com".to_string()),
            vec!["user".to_string()],
            vec!["profile".to_string()],
        )
        .unwrap();

    // Make verify request with invalid code
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/mfa/verify")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(json!({ "code": "000000" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    Ok(())
}

#[sqlx::test]
async fn test_mfa_challenge_endpoint_with_totp(pool: PgPool) -> sqlx::Result<()> {
    let (app, jwt_manager) = setup_test_app(pool.clone()).await;

    // Setup: Enroll and verify user
    let user_id = "test_user_challenge";
    let mfa_manager = MfaManager::new(pool.clone(), &[42u8; 32]);
    let (secret, _qr, _codes) = mfa_manager
        .initiate_enrollment(user_id.to_string())
        .await
        .unwrap();

    let totp_manager = media_gateway_auth::mfa::totp::TotpManager::new(&[42u8; 32]);
    let enroll_code = totp_manager.generate_current_code(&secret).unwrap();
    mfa_manager
        .verify_enrollment(user_id, &enroll_code)
        .await
        .unwrap();

    // Generate challenge code
    let challenge_code = totp_manager.generate_current_code(&secret).unwrap();

    // Create JWT
    let access_token = jwt_manager
        .create_access_token(
            user_id.to_string(),
            Some("test@example.com".to_string()),
            vec!["user".to_string()],
            vec!["profile".to_string()],
        )
        .unwrap();

    // Make challenge request
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/mfa/challenge")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(json!({ "code": challenge_code }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["message"].as_str().unwrap(), "MFA challenge passed");
    assert_eq!(body["authenticated"].as_bool().unwrap(), true);

    Ok(())
}

#[sqlx::test]
async fn test_mfa_challenge_endpoint_with_backup_code(pool: PgPool) -> sqlx::Result<()> {
    let (app, jwt_manager) = setup_test_app(pool.clone()).await;

    // Setup: Enroll and verify user
    let user_id = "test_user_backup";
    let mfa_manager = MfaManager::new(pool.clone(), &[42u8; 32]);
    let (secret, _qr, backup_codes) = mfa_manager
        .initiate_enrollment(user_id.to_string())
        .await
        .unwrap();

    let totp_manager = media_gateway_auth::mfa::totp::TotpManager::new(&[42u8; 32]);
    let enroll_code = totp_manager.generate_current_code(&secret).unwrap();
    mfa_manager
        .verify_enrollment(user_id, &enroll_code)
        .await
        .unwrap();

    // Create JWT
    let access_token = jwt_manager
        .create_access_token(
            user_id.to_string(),
            Some("test@example.com".to_string()),
            vec!["user".to_string()],
            vec!["profile".to_string()],
        )
        .unwrap();

    // Make challenge request with backup code
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/mfa/challenge")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(json!({ "code": backup_codes[0] }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify backup code is single-use
    let req2 = test::TestRequest::post()
        .uri("/api/v1/auth/mfa/challenge")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(json!({ "code": backup_codes[0] }))
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 401);

    Ok(())
}

#[sqlx::test]
async fn test_mfa_rate_limiting(pool: PgPool) -> sqlx::Result<()> {
    let (app, jwt_manager) = setup_test_app(pool.clone()).await;

    // Setup: Enroll user
    let user_id = "test_user_rate_limit";
    let mfa_manager = MfaManager::new(pool.clone(), &[42u8; 32]);
    let (_secret, _qr, _codes) = mfa_manager
        .initiate_enrollment(user_id.to_string())
        .await
        .unwrap();

    // Create JWT
    let access_token = jwt_manager
        .create_access_token(
            user_id.to_string(),
            Some("test@example.com".to_string()),
            vec!["user".to_string()],
            vec!["profile".to_string()],
        )
        .unwrap();

    // Make 5 failed attempts
    for _ in 0..5 {
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/mfa/verify")
            .insert_header(("Authorization", format!("Bearer {}", access_token)))
            .set_json(json!({ "code": "000000" }))
            .to_request();

        let _ = test::call_service(&app, req).await;
    }

    // 6th attempt should be rate limited
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/mfa/verify")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(json!({ "code": "000000" }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 429);

    Ok(())
}
