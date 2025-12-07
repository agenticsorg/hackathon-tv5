use actix_web::{test, web, App};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use media_gateway_auth::{
    error::Result,
    password_reset::{ResetPasswordRequest, ResetPasswordResponse},
    password_reset_handlers::{reset_password, AppState},
    session::SessionManager,
    storage::AuthStorage,
    token_family::TokenFamilyManager,
    user::{PasswordHasher, PostgresUserRepository, UserRepository},
};

async fn setup_test_user(pool: &PgPool) -> Result<(Uuid, String)> {
    let user_repo = PostgresUserRepository::new(pool.clone());
    let email = format!("test-{}@example.com", Uuid::new_v4());
    let password_hash = PasswordHasher::hash_password("OldPassword123")?;

    let user_id = user_repo.create_user(&email, &password_hash).await?;
    Ok((user_id, email))
}

#[sqlx::test]
async fn test_password_reset_invalidates_sessions(pool: PgPool) -> Result<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Setup dependencies
    let storage = Arc::new(AuthStorage::new(&redis_url)?);
    let session_manager = Arc::new(SessionManager::new(&redis_url)?);
    let token_family_manager = Arc::new(TokenFamilyManager::new(&redis_url)?);

    // Create test user
    let (user_id, email) = setup_test_user(&pool).await?;

    // Create multiple sessions for the user
    let session1 = session_manager
        .create_session(
            user_id.to_string(),
            "jti1".to_string(),
            Some("device1".to_string()),
        )
        .await?;
    let session2 = session_manager
        .create_session(
            user_id.to_string(),
            "jti2".to_string(),
            Some("device2".to_string()),
        )
        .await?;
    let session3 = session_manager
        .create_session(user_id.to_string(), "jti3".to_string(), None)
        .await?;

    // Verify sessions exist
    assert!(session_manager
        .get_session(&session1.session_id)
        .await
        .is_ok());
    assert!(session_manager
        .get_session(&session2.session_id)
        .await
        .is_ok());
    assert!(session_manager
        .get_session(&session3.session_id)
        .await
        .is_ok());

    // Create token families (refresh tokens)
    let family1 = token_family_manager.create_family(user_id).await?;
    token_family_manager
        .add_token_to_family(family1, "token1")
        .await?;

    let family2 = token_family_manager.create_family(user_id).await?;
    token_family_manager
        .add_token_to_family(family2, "token2")
        .await?;

    // Create password reset token
    let reset_token = media_gateway_auth::password_reset::PasswordResetToken::new(
        user_id.to_string(),
        email.clone(),
    );
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await?;

    // Setup Actix app
    let app_state = AppState {
        storage: storage.clone(),
        session_manager: session_manager.clone(),
        token_family_manager: token_family_manager.clone(),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .app_data(web::Data::new(pool.clone()))
            .service(reset_password),
    )
    .await;

    // Execute password reset
    let reset_request = ResetPasswordRequest {
        token: reset_token.token.clone(),
        new_password: "NewPassword123".to_string(),
        keep_current_session: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/reset")
        .set_json(&reset_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: ResetPasswordResponse = test::read_body_json(resp).await;
    assert_eq!(body.sessions_invalidated, 3);
    assert_eq!(body.tokens_revoked, 2);

    // Verify all sessions are invalidated
    assert!(session_manager
        .get_session(&session1.session_id)
        .await
        .is_err());
    assert!(session_manager
        .get_session(&session2.session_id)
        .await
        .is_err());
    assert!(session_manager
        .get_session(&session3.session_id)
        .await
        .is_err());

    // Verify token families are revoked
    assert!(!token_family_manager.family_exists(family1).await?);
    assert!(!token_family_manager.family_exists(family2).await?);

    Ok(())
}

#[sqlx::test]
async fn test_password_reset_with_no_sessions(pool: PgPool) -> Result<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());

    // Setup dependencies
    let storage = Arc::new(AuthStorage::new(&redis_url)?);
    let session_manager = Arc::new(SessionManager::new(&redis_url)?);
    let token_family_manager = Arc::new(TokenFamilyManager::new(&redis_url)?);

    // Create test user
    let (user_id, email) = setup_test_user(&pool).await?;

    // Create password reset token
    let reset_token = media_gateway_auth::password_reset::PasswordResetToken::new(
        user_id.to_string(),
        email.clone(),
    );
    storage
        .store_password_reset_token(&reset_token.token, &reset_token)
        .await?;

    // Setup Actix app
    let app_state = AppState {
        storage: storage.clone(),
        session_manager: session_manager.clone(),
        token_family_manager: token_family_manager.clone(),
    };

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(app_state))
            .app_data(web::Data::new(pool.clone()))
            .service(reset_password),
    )
    .await;

    // Execute password reset
    let reset_request = ResetPasswordRequest {
        token: reset_token.token.clone(),
        new_password: "NewPassword123".to_string(),
        keep_current_session: false,
    };

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/password/reset")
        .set_json(&reset_request)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: ResetPasswordResponse = test::read_body_json(resp).await;
    assert_eq!(body.sessions_invalidated, 0);
    assert_eq!(body.tokens_revoked, 0);

    Ok(())
}

#[sqlx::test]
async fn test_session_invalidation_atomic(pool: PgPool) -> Result<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let session_manager = SessionManager::new(&redis_url)?;

    // Create test user
    let user_id = Uuid::new_v4();

    // Create sessions
    let session1 = session_manager
        .create_session(user_id.to_string(), "jti1".to_string(), None)
        .await?;
    let session2 = session_manager
        .create_session(user_id.to_string(), "jti2".to_string(), None)
        .await?;

    // Invalidate all sessions
    let count = session_manager
        .invalidate_all_user_sessions(&user_id, None)
        .await?;

    assert_eq!(count, 2);

    // Verify sessions are gone
    assert!(session_manager
        .get_session(&session1.session_id)
        .await
        .is_err());
    assert!(session_manager
        .get_session(&session2.session_id)
        .await
        .is_err());

    Ok(())
}

#[sqlx::test]
async fn test_session_invalidation_except_current(pool: PgPool) -> Result<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let session_manager = SessionManager::new(&redis_url)?;

    // Create test user
    let user_id = Uuid::new_v4();

    // Create sessions
    let session1 = session_manager
        .create_session(user_id.to_string(), "jti1".to_string(), None)
        .await?;
    let session2 = session_manager
        .create_session(user_id.to_string(), "jti2".to_string(), None)
        .await?;
    let session3 = session_manager
        .create_session(user_id.to_string(), "jti3".to_string(), None)
        .await?;

    // Invalidate all except session2
    let count = session_manager
        .invalidate_all_user_sessions(&user_id, Some(&session2.session_id))
        .await?;

    assert_eq!(count, 2);

    // Verify session1 and session3 are gone
    assert!(session_manager
        .get_session(&session1.session_id)
        .await
        .is_err());
    assert!(session_manager
        .get_session(&session3.session_id)
        .await
        .is_err());

    // Verify session2 still exists
    assert!(session_manager
        .get_session(&session2.session_id)
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn test_revoke_all_user_tokens() -> Result<()> {
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    let token_family_manager = TokenFamilyManager::new(&redis_url)?;

    let user_id = Uuid::new_v4();
    let other_user_id = Uuid::new_v4();

    // Create families for target user
    let family1 = token_family_manager.create_family(user_id).await?;
    token_family_manager
        .add_token_to_family(family1, "token1")
        .await?;

    let family2 = token_family_manager.create_family(user_id).await?;
    token_family_manager
        .add_token_to_family(family2, "token2")
        .await?;

    // Create family for other user (should not be affected)
    let other_family = token_family_manager.create_family(other_user_id).await?;
    token_family_manager
        .add_token_to_family(other_family, "other_token")
        .await?;

    // Revoke all tokens for target user
    let revoked = token_family_manager
        .revoke_all_user_tokens(&user_id)
        .await?;
    assert_eq!(revoked, 2);

    // Verify target user's families are gone
    assert!(!token_family_manager.family_exists(family1).await?);
    assert!(!token_family_manager.family_exists(family2).await?);

    // Verify other user's family still exists
    assert!(token_family_manager.family_exists(other_family).await?);

    Ok(())
}
