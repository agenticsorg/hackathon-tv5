use actix_web::{
    http::header,
    test,
    web::{self, Data},
    App,
};
use media_gateway_auth::{
    admin::{
        delete_user, get_user_detail, impersonate_user, list_users, update_user, AdminMiddleware,
        AdminUpdateUserRequest, ListUsersQuery,
    },
    jwt::JwtManager,
    middleware::UserContext,
    rbac::Role,
    session::SessionManager,
};
use sqlx::PgPool;
use std::{rc::Rc, sync::Arc};
use uuid::Uuid;

// ============================================================================
// Test Helpers
// ============================================================================

fn create_test_jwt_manager() -> Arc<JwtManager> {
    let private_key = include_bytes!("../../tests/fixtures/test_private_key.pem");
    let public_key = include_bytes!("../../tests/fixtures/test_public_key.pem");

    Arc::new(
        JwtManager::new(
            private_key,
            public_key,
            "https://api.mediagateway.io".to_string(),
            "mediagateway-users".to_string(),
        )
        .unwrap(),
    )
}

async fn create_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost/media_gateway_test".to_string()
    });

    PgPool::connect(&database_url).await.unwrap()
}

async fn create_test_session_manager() -> Arc<SessionManager> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let redis_client = redis::Client::open(redis_url).unwrap();

    Arc::new(SessionManager::new(redis_client))
}

async fn setup_test_data(pool: &PgPool) -> (Uuid, Uuid, String) {
    // Create admin user
    let admin_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, role, email_verified, suspended, created_at, password_hash)
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(admin_id)
    .bind("admin@example.com")
    .bind("Admin User")
    .bind("admin")
    .bind(true)
    .bind(false)
    .bind("$2b$12$dummy_hash_for_testing")
    .execute(pool)
    .await
    .unwrap();

    // Create regular user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, role, email_verified, suspended, created_at, password_hash)
         VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(user_id)
    .bind("user@example.com")
    .bind("Regular User")
    .bind("user")
    .bind(true)
    .bind(false)
    .bind("$2b$12$dummy_hash_for_testing")
    .execute(pool)
    .await
    .unwrap();

    // Create audit logs table if not exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS audit_logs (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            admin_user_id UUID NOT NULL,
            action TEXT NOT NULL,
            target_user_id UUID,
            details JSONB,
            timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            event_type TEXT,
            ip_address TEXT,
            user_agent TEXT
        )",
    )
    .execute(pool)
    .await
    .unwrap();

    // Generate admin token
    let jwt_manager = create_test_jwt_manager();
    let admin_token = jwt_manager
        .create_access_token(
            admin_id.to_string(),
            Some("admin@example.com".to_string()),
            vec!["admin".to_string()],
            vec!["admin:*".to_string()],
        )
        .unwrap();

    (admin_id, user_id, admin_token)
}

async fn cleanup_test_data(pool: &PgPool, admin_id: Uuid, user_id: Uuid) {
    sqlx::query("DELETE FROM audit_logs WHERE admin_user_id = $1 OR target_user_id = $1 OR target_user_id = $2")
        .bind(admin_id)
        .bind(user_id)
        .execute(pool)
        .await
        .ok();

    sqlx::query("DELETE FROM users WHERE id = $1 OR id = $2")
        .bind(admin_id)
        .bind(user_id)
        .execute(pool)
        .await
        .ok();
}

// ============================================================================
// Integration Tests - Admin Middleware
// ============================================================================

#[actix_web::test]
async fn test_admin_middleware_allows_admin_user() {
    let pool = create_test_db().await;
    let session_manager = create_test_session_manager().await;
    let jwt_manager = create_test_jwt_manager();
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new().app_data(Data::new(pool.clone())).service(
            web::scope("/api/v1/admin")
                .wrap(AdminMiddleware::new(
                    Rc::new((*jwt_manager).clone()),
                    Rc::new((*session_manager).clone()),
                ))
                .service(list_users),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?page=1&per_page=20")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Admin user should be able to access admin endpoints"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_admin_middleware_rejects_non_admin_user() {
    let pool = create_test_db().await;
    let session_manager = create_test_session_manager().await;
    let jwt_manager = create_test_jwt_manager();
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    // Create token for non-admin user
    let user_token = jwt_manager
        .create_access_token(
            user_id.to_string(),
            Some("user@example.com".to_string()),
            vec!["user".to_string()],
            vec!["read:content".to_string()],
        )
        .unwrap();

    let app = test::init_service(
        App::new().app_data(Data::new(pool.clone())).service(
            web::scope("/api/v1/admin")
                .wrap(AdminMiddleware::new(
                    Rc::new((*jwt_manager).clone()),
                    Rc::new((*session_manager).clone()),
                ))
                .service(list_users),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", user_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_client_error(),
        "Non-admin user should be rejected from admin endpoints"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_admin_middleware_rejects_missing_token() {
    let pool = create_test_db().await;
    let session_manager = create_test_session_manager().await;
    let jwt_manager = create_test_jwt_manager();

    let app = test::init_service(
        App::new().app_data(Data::new(pool.clone())).service(
            web::scope("/api/v1/admin")
                .wrap(AdminMiddleware::new(
                    Rc::new((*jwt_manager).clone()),
                    Rc::new((*session_manager).clone()),
                ))
                .service(list_users),
        ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_client_error(),
        "Request without token should be rejected"
    );
}

// ============================================================================
// Integration Tests - List Users Endpoint
// ============================================================================

#[actix_web::test]
async fn test_list_users_with_pagination() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    // Create additional test users
    for i in 0..5 {
        let test_user_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO users (id, email, display_name, role, email_verified, suspended, created_at, password_hash)
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), $7)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind(test_user_id)
        .bind(format!("testuser{}@example.com", i))
        .bind(format!("Test User {}", i))
        .bind("user")
        .bind(true)
        .bind(false)
        .bind("$2b$12$dummy_hash_for_testing")
        .execute(&pool)
        .await
        .unwrap();
    }

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    // Simulate admin context
    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?page=1&per_page=3")
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["users"].is_array());
    assert!(body["total"].as_i64().unwrap() >= 2);
    assert_eq!(body["page"], 1);
    assert_eq!(body["per_page"], 3);

    // Cleanup additional users
    sqlx::query("DELETE FROM users WHERE email LIKE 'testuser%@example.com'")
        .execute(&pool)
        .await
        .ok();

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_list_users_with_search() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?search=admin@example.com")
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["users"].is_array());
    let users = body["users"].as_array().unwrap();
    assert!(users.iter().any(|u| u["email"] == "admin@example.com"));

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_list_users_with_sorting() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?sort_by=email&sort_order=asc")
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_list_users_with_status_filter() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    // Suspend the regular user
    sqlx::query("UPDATE users SET suspended = true WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?status=suspended")
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    let users = body["users"].as_array().unwrap();
    assert!(users
        .iter()
        .all(|u| u["suspended"].as_bool().unwrap_or(false)));

    cleanup_test_data(&pool, admin_id, user_id).await;
}

// ============================================================================
// Integration Tests - Get User Detail Endpoint
// ============================================================================

#[actix_web::test]
async fn test_get_user_detail_returns_user_info() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(get_user_detail),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["email"], "user@example.com");
    assert!(body["recent_activity"].is_array());
    assert!(body["session_count"].is_number());

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_get_user_detail_logs_audit_action() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(get_user_detail),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify audit log was created
    let audit_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_logs WHERE admin_user_id = $1 AND action = 'get_user_detail'",
    )
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        audit_count.0 > 0,
        "Audit log should be created for get_user_detail action"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

// ============================================================================
// Integration Tests - Update User Endpoint
// ============================================================================

#[actix_web::test]
async fn test_update_user_suspend() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(update_user),
    )
    .await;

    let update_req = AdminUpdateUserRequest {
        role: None,
        suspended: Some(true),
        email_verified: None,
    };

    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&update_req)
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify user is suspended in database
    let user: (bool,) = sqlx::query_as("SELECT suspended FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user.0, true);

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_update_user_change_role() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(update_user),
    )
    .await;

    let update_req = AdminUpdateUserRequest {
        role: Some("premium".to_string()),
        suspended: None,
        email_verified: None,
    };

    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&update_req)
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify role changed in database
    let user: (String,) = sqlx::query_as("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user.0, "premium");

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_update_user_verify_email() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    // Set user as unverified
    sqlx::query("UPDATE users SET email_verified = false WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(update_user),
    )
    .await;

    let update_req = AdminUpdateUserRequest {
        role: None,
        suspended: None,
        email_verified: Some(true),
    };

    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&update_req)
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify email_verified changed in database
    let user: (bool,) = sqlx::query_as("SELECT email_verified FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user.0, true);

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_update_user_invalid_role() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(update_user),
    )
    .await;

    let update_req = AdminUpdateUserRequest {
        role: Some("superadmin".to_string()),
        suspended: None,
        email_verified: None,
    };

    let req = test::TestRequest::patch()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .insert_header((header::CONTENT_TYPE, "application/json"))
        .set_json(&update_req)
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_server_error() || resp.status().is_client_error());

    cleanup_test_data(&pool, admin_id, user_id).await;
}

// ============================================================================
// Integration Tests - Delete User Endpoint
// ============================================================================

#[actix_web::test]
async fn test_delete_user_hard_delete() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(delete_user),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify user is deleted from database
    let user_exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(
        user_exists.is_none(),
        "User should be hard deleted from database"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_delete_user_cannot_delete_self() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(delete_user),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{}", admin_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_server_error() || resp.status().is_client_error(),
        "Admin should not be able to delete their own account"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_delete_user_logs_audit_action() {
    let pool = create_test_db().await;
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(delete_user),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify audit log was created
    let audit_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_logs WHERE admin_user_id = $1 AND action = 'delete_user'",
    )
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        audit_count.0 > 0,
        "Audit log should be created for delete_user action"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

// ============================================================================
// Integration Tests - Impersonate User Endpoint
// ============================================================================

#[actix_web::test]
async fn test_impersonate_user_generates_token() {
    let pool = create_test_db().await;
    let jwt_manager = create_test_jwt_manager();
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(jwt_manager.clone()))
            .service(impersonate_user),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/admin/users/{}/impersonate", user_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["access_token"].is_string());
    assert_eq!(body["expires_in"], 900); // 15 minutes
    assert_eq!(body["original_admin_id"], admin_id.to_string());
    assert_eq!(body["impersonated_user_id"], user_id.to_string());

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_impersonate_user_cannot_impersonate_self() {
    let pool = create_test_db().await;
    let jwt_manager = create_test_jwt_manager();
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(jwt_manager.clone()))
            .service(impersonate_user),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/admin/users/{}/impersonate", admin_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_server_error() || resp.status().is_client_error(),
        "Admin should not be able to impersonate themselves"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_impersonate_user_cannot_impersonate_suspended_user() {
    let pool = create_test_db().await;
    let jwt_manager = create_test_jwt_manager();
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    // Suspend the user
    sqlx::query("UPDATE users SET suspended = true WHERE id = $1")
        .bind(user_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(jwt_manager.clone()))
            .service(impersonate_user),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/admin/users/{}/impersonate", user_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_server_error() || resp.status().is_client_error(),
        "Admin should not be able to impersonate suspended users"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_impersonate_user_logs_audit_action() {
    let pool = create_test_db().await;
    let jwt_manager = create_test_jwt_manager();
    let (admin_id, user_id, _) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(jwt_manager.clone()))
            .service(impersonate_user),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/admin/users/{}/impersonate", user_id))
        .to_request();

    req.extensions_mut().insert(UserContext {
        user_id: admin_id.to_string(),
        email: Some("admin@example.com".to_string()),
        roles: vec![Role::Admin],
        scopes: vec!["admin:*".to_string()],
        jti: "test-jti".to_string(),
    });

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Verify audit log was created
    let audit_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_logs WHERE admin_user_id = $1 AND action = 'impersonate_user'",
    )
    .bind(admin_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        audit_count.0 > 0,
        "Audit log should be created for impersonate_user action"
    );

    cleanup_test_data(&pool, admin_id, user_id).await;
}
