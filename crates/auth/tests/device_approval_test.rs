use actix_web::{test, web, App};
use auth::{
    error::AuthError,
    jwt::JwtManager,
    oauth::OAuthConfig,
    oauth::OAuthManager,
    rbac::RbacManager,
    scopes::ScopeManager,
    server::{start_server, AppState},
    session::SessionManager,
    storage::AuthStorage,
};
use serde_json::json;
use std::sync::Arc;

/// Test helper to create app state
async fn create_test_app_state() -> web::Data<AppState> {
    // Generate test RSA keys
    let private_key = include_bytes!("../../tests/fixtures/test_private_key.pem");
    let public_key = include_bytes!("../../tests/fixtures/test_public_key.pem");

    let jwt_manager = Arc::new(
        JwtManager::new(
            private_key,
            public_key,
            "https://api.mediagateway.io".to_string(),
            "mediagateway-users".to_string(),
        )
        .expect("Failed to create JWT manager"),
    );

    let storage =
        Arc::new(AuthStorage::new("redis://127.0.0.1:6379").expect("Failed to connect to Redis"));

    let session_manager = Arc::new(
        SessionManager::new(storage.clone())
            .await
            .expect("Failed to create session manager"),
    );

    let oauth_config = OAuthConfig {
        providers: std::collections::HashMap::new(),
    };

    web::Data::new(AppState {
        jwt_manager,
        session_manager,
        oauth_manager: Arc::new(OAuthManager::new(oauth_config)),
        rbac_manager: Arc::new(RbacManager::new()),
        scope_manager: Arc::new(ScopeManager::new()),
        storage,
    })
}

#[actix_web::test]
#[ignore] // Requires Redis and test keys
async fn test_full_device_authorization_flow() {
    let app_state = create_test_app_state().await;

    // Step 1: Device requests authorization
    let device_req = json!({
        "client_id": "test-client",
        "scope": "read:content write:content"
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route(
                "/auth/device",
                web::post().to(auth::server::device_authorization),
            )
            .route(
                "/auth/device/approve",
                web::post().to(auth::server::approve_device),
            )
            .route(
                "/auth/device/poll",
                web::get().to(auth::server::device_poll),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/device")
        .set_form(&device_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let device_response: serde_json::Value = test::read_body_json(resp).await;
    let device_code = device_response["device_code"].as_str().unwrap();
    let user_code = device_response["user_code"].as_str().unwrap();

    // Step 2: Create a user token for approval
    let access_token = app_state
        .jwt_manager
        .create_access_token(
            "test-user-123".to_string(),
            Some("test@example.com".to_string()),
            vec!["free_user".to_string()],
            vec!["read:content".to_string()],
        )
        .unwrap();

    // Step 3: User approves the device
    let approval_req = json!({
        "user_code": user_code
    });

    let req = test::TestRequest::post()
        .uri("/auth/device/approve")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&approval_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let approval_response: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        approval_response["message"],
        "Device authorization approved"
    );

    // Step 4: Device polls and receives tokens
    let req = test::TestRequest::get()
        .uri(&format!("/auth/device/poll?device_code={}", device_code))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let token_response: serde_json::Value = test::read_body_json(resp).await;
    assert!(token_response["access_token"].is_string());
    assert!(token_response["refresh_token"].is_string());
    assert_eq!(token_response["token_type"], "Bearer");
    assert_eq!(token_response["expires_in"], 3600);
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_device_approval_invalid_user_code() {
    let app_state = create_test_app_state().await;

    // Create a user token
    let access_token = app_state
        .jwt_manager
        .create_access_token(
            "test-user-123".to_string(),
            Some("test@example.com".to_string()),
            vec!["free_user".to_string()],
            vec!["read:content".to_string()],
        )
        .unwrap();

    let app = test::init_service(App::new().app_data(app_state.clone()).route(
        "/auth/device/approve",
        web::post().to(auth::server::approve_device),
    ))
    .await;

    // Try to approve with invalid user code
    let approval_req = json!({
        "user_code": "INVALID-CODE"
    });

    let req = test::TestRequest::post()
        .uri("/auth/device/approve")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&approval_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_device_approval_missing_authorization() {
    let app_state = create_test_app_state().await;

    let app = test::init_service(App::new().app_data(app_state.clone()).route(
        "/auth/device/approve",
        web::post().to(auth::server::approve_device),
    ))
    .await;

    // Try to approve without authorization header
    let approval_req = json!({
        "user_code": "TEST-CODE"
    });

    let req = test::TestRequest::post()
        .uri("/auth/device/approve")
        .set_json(&approval_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401); // Unauthorized
}

#[actix_web::test]
#[ignore] // Requires Redis and test keys
async fn test_device_approval_already_approved() {
    let app_state = create_test_app_state().await;

    // Step 1: Create device authorization
    let device_req = json!({
        "client_id": "test-client",
        "scope": "read:content"
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route(
                "/auth/device",
                web::post().to(auth::server::device_authorization),
            )
            .route(
                "/auth/device/approve",
                web::post().to(auth::server::approve_device),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/device")
        .set_form(&device_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let device_response: serde_json::Value = test::read_body_json(resp).await;
    let user_code = device_response["user_code"].as_str().unwrap();

    // Step 2: Create user token
    let access_token = app_state
        .jwt_manager
        .create_access_token(
            "test-user-123".to_string(),
            Some("test@example.com".to_string()),
            vec!["free_user".to_string()],
            vec!["read:content".to_string()],
        )
        .unwrap();

    // Step 3: Approve device
    let approval_req = json!({
        "user_code": user_code
    });

    let req = test::TestRequest::post()
        .uri("/auth/device/approve")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&approval_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Step 4: Try to approve again
    let req = test::TestRequest::post()
        .uri("/auth/device/approve")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .set_json(&approval_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request - already approved
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_device_poll_authorization_pending() {
    let app_state = create_test_app_state().await;

    // Step 1: Create device authorization
    let device_req = json!({
        "client_id": "test-client",
        "scope": "read:content"
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route(
                "/auth/device",
                web::post().to(auth::server::device_authorization),
            )
            .route(
                "/auth/device/poll",
                web::get().to(auth::server::device_poll),
            ),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/auth/device")
        .set_form(&device_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    let device_response: serde_json::Value = test::read_body_json(resp).await;
    let device_code = device_response["device_code"].as_str().unwrap();

    // Step 2: Poll before approval
    let req = test::TestRequest::get()
        .uri(&format!("/auth/device/poll?device_code={}", device_code))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request - authorization_pending
}

#[actix_web::test]
#[ignore] // Requires Redis
async fn test_device_poll_invalid_device_code() {
    let app_state = create_test_app_state().await;

    let app = test::init_service(App::new().app_data(app_state.clone()).route(
        "/auth/device/poll",
        web::get().to(auth::server::device_poll),
    ))
    .await;

    // Poll with invalid device code
    let req = test::TestRequest::get()
        .uri("/auth/device/poll?device_code=INVALID")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400); // Bad Request
}
