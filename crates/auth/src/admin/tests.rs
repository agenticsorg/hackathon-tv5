use crate::{
    admin::{
        delete_user, get_audit_logs, get_user_detail, impersonate_user, list_users, update_user,
        AdminUpdateUserRequest, AuditLogsQuery, ListUsersQuery,
    },
    error::AuthError,
    jwt::JwtManager,
    middleware::UserContext,
    rbac::Role,
    session::SessionManager,
};
use actix_web::{
    http::header,
    test,
    web::{self, Data},
    App, HttpMessage,
};
use chrono::Utc;
use media_gateway_core::audit::{AuditAction, AuditEvent, AuditLogger, PostgresAuditLogger};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

fn create_test_jwt_manager() -> Arc<JwtManager> {
    let private_key = include_bytes!("../../../../tests/fixtures/test_private_key.pem");
    let public_key = include_bytes!("../../../../tests/fixtures/test_public_key.pem");

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

async fn setup_test_data(pool: &PgPool) -> (Uuid, Uuid, String) {
    // Create admin user
    let admin_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, role, email_verified, suspended, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW())",
    )
    .bind(admin_id)
    .bind("admin@example.com")
    .bind("Admin User")
    .bind("admin")
    .bind(true)
    .bind(false)
    .execute(pool)
    .await
    .unwrap();

    // Create regular user
    let user_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO users (id, email, display_name, role, email_verified, suspended, created_at)
         VALUES ($1, $2, $3, $4, $5, $6, NOW())",
    )
    .bind(user_id)
    .bind("user@example.com")
    .bind("Regular User")
    .bind("user")
    .bind(true)
    .bind(false)
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
            timestamp TIMESTAMPTZ NOT NULL,
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

#[actix_web::test]
async fn test_list_users_with_admin_token() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?page=1&per_page=20")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
        .to_request();

    // Insert admin context manually for test
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
async fn test_get_user_detail_with_admin_token() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(get_user_detail),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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
async fn test_update_user_suspend() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

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
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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

    // Verify user is suspended
    let user: (bool,) = sqlx::query_as("SELECT suspended FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user.0, true);

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_update_user_role() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

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
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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

    // Verify role changed
    let user: (String,) = sqlx::query_as("SELECT role FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user.0, "premium");

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_delete_user_hard_delete() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(delete_user),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{}", user_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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

    // Verify user is deleted
    let user_exists: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(user_exists.is_none());

    cleanup_test_data(&pool, admin_id, user_id).await;
}

#[actix_web::test]
async fn test_delete_user_cannot_delete_self() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(delete_user),
    )
    .await;

    // Try to delete admin's own account
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/admin/users/{}", admin_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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

#[actix_web::test]
async fn test_impersonate_user_generates_token() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;
    let jwt_manager = create_test_jwt_manager();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(jwt_manager.clone()))
            .service(impersonate_user),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/admin/users/{}/impersonate", user_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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
async fn test_impersonate_user_cannot_impersonate_self() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;
    let jwt_manager = create_test_jwt_manager();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .app_data(Data::new(jwt_manager.clone()))
            .service(impersonate_user),
    )
    .await;

    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/admin/users/{}/impersonate", admin_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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

#[actix_web::test]
async fn test_list_users_pagination() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?page=1&per_page=10&sort_by=email&sort_order=asc")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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
async fn test_list_users_search() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?search=user@example.com")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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
async fn test_list_users_filter_by_status() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(list_users),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/users?status=active")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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

#[test]
fn test_list_users_query_validation() {
    let mut query = ListUsersQuery {
        page: 0,
        per_page: 0,
        sort_by: None,
        sort_order: None,
        search: None,
        status: None,
    };

    query.validate().unwrap();

    assert_eq!(query.page, 1);
    assert_eq!(query.per_page, 20);
}

#[test]
fn test_list_users_query_max_per_page() {
    let mut query = ListUsersQuery {
        page: 1,
        per_page: 200,
        sort_by: None,
        sort_order: None,
        search: None,
        status: None,
    };

    query.validate().unwrap();

    assert_eq!(query.per_page, 100);
}

#[test]
fn test_admin_update_request_validation() {
    let valid_req = AdminUpdateUserRequest {
        role: Some("premium".to_string()),
        suspended: Some(true),
        email_verified: Some(true),
    };

    assert!(valid_req.validate().is_ok());
}

#[test]
fn test_admin_update_request_invalid_role() {
    let invalid_req = AdminUpdateUserRequest {
        role: Some("superadmin".to_string()),
        suspended: None,
        email_verified: None,
    };

    assert!(invalid_req.validate().is_err());
}

#[actix_web::test]
async fn test_get_audit_logs_with_filters() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    // Create audit logger and insert test data
    let audit_logger = PostgresAuditLogger::new(pool.clone());

    let event1 = AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
        .with_user_id(user_id)
        .with_ip_address("192.168.1.1".to_string());

    let event2 = AuditEvent::new(AuditAction::UserCreated, "user".to_string())
        .with_user_id(admin_id)
        .with_ip_address("192.168.1.2".to_string());

    audit_logger.log_batch(vec![event1, event2]).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(get_audit_logs),
    )
    .await;

    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/admin/audit-logs?user_id={}", user_id))
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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
async fn test_get_audit_logs_pagination() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    // Create audit logger and insert test data
    let audit_logger = PostgresAuditLogger::new(pool.clone());

    let mut events = vec![];
    for i in 0..5 {
        events.push(
            AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
                .with_user_id(user_id)
                .with_resource_id(format!("login-{}", i)),
        );
    }

    audit_logger.log_batch(events).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(get_audit_logs),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/audit-logs?page=1&per_page=3")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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
async fn test_get_audit_logs_action_filter() {
    let pool = create_test_db().await;
    let (admin_id, user_id, admin_token) = setup_test_data(&pool).await;

    // Create audit logger and insert test data
    let audit_logger = PostgresAuditLogger::new(pool.clone());

    let events = vec![
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(user_id),
        AuditEvent::new(AuditAction::UserCreated, "user".to_string()).with_user_id(admin_id),
        AuditEvent::new(AuditAction::AuthLogin, "user".to_string()).with_user_id(admin_id),
    ];

    audit_logger.log_batch(events).await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(pool.clone()))
            .service(get_audit_logs),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/v1/admin/audit-logs?action=AUTH_LOGIN")
        .insert_header((header::AUTHORIZATION, format!("Bearer {}", admin_token)))
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

#[test]
fn test_audit_logs_query_validation() {
    let mut query = AuditLogsQuery {
        start_date: None,
        end_date: None,
        user_id: None,
        action: None,
        resource_type: None,
        page: 0,
        per_page: 0,
    };

    query.validate().unwrap();

    assert_eq!(query.page, 1);
    assert_eq!(query.per_page, 50);
}

#[test]
fn test_audit_logs_query_max_per_page() {
    let mut query = AuditLogsQuery {
        start_date: None,
        end_date: None,
        user_id: None,
        action: None,
        resource_type: None,
        page: 1,
        per_page: 300,
    };

    query.validate().unwrap();

    assert_eq!(query.per_page, 200);
}
