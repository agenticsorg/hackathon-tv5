use crate::{
    email::EmailManager,
    error::{AuthError, Result},
    password_reset::{ForgotPasswordRequest, ForgotPasswordResponse, PasswordResetToken, ResetPasswordRequest, ResetPasswordResponse, PasswordValidator},
    session::SessionManager,
    storage::AuthStorage,
    token_family::TokenFamilyManager,
    user::{PasswordHasher, PostgresUserRepository, UserRepository},
};
use actix_web::{post, web::{self, Data}, HttpResponse, Responder};
use std::sync::Arc;
use uuid::Uuid;

pub struct AppState {
    pub storage: Arc<AuthStorage>,
    pub email_manager: Arc<EmailManager>,
    pub session_manager: Arc<SessionManager>,
    pub token_family_manager: Arc<TokenFamilyManager>,
}

#[post("/api/v1/auth/password/forgot")]
pub async fn forgot_password(
    req: web::Json<ForgotPasswordRequest>,
    state: Data<AppState>,
    db_pool: Data<sqlx::PgPool>,
) -> Result<impl Responder> {
    let user_repo = PostgresUserRepository::new(db_pool.get_ref().clone());

    // Check rate limit
    let remaining = state.storage.check_password_reset_rate_limit(&req.email).await?;
    if remaining == 0 {
        // Return success even when rate limited to prevent enumeration
        return Ok(HttpResponse::Ok().json(ForgotPasswordResponse {
            message: "If an account exists with this email, a password reset link has been sent.".to_string(),
        }));
    }

    // Find user by email
    let user = user_repo.find_by_email(&req.email).await?;

    // Always return success to prevent email enumeration
    if let Some(user) = user {
        // Generate reset token
        let reset_token = PasswordResetToken::new(user.id.to_string(), user.email.clone());

        // Store token in Redis
        state.storage.store_password_reset_token(&reset_token.token, &reset_token).await?;

        // Send password reset email
        if let Err(e) = state.email_manager.send_password_reset_email(
            user.email.clone(),
            reset_token.token.clone(),
        ).await {
            tracing::error!("Failed to send password reset email: {}", e);
            // Continue anyway to prevent email enumeration
        } else {
            tracing::info!("Password reset email sent to: {}", user.email);
        }
    }

    Ok(HttpResponse::Ok().json(ForgotPasswordResponse {
        message: "If an account exists with this email, a password reset link has been sent.".to_string(),
    }))
}

#[post("/api/v1/auth/password/reset")]
pub async fn reset_password(
    req: web::Json<ResetPasswordRequest>,
    state: Data<AppState>,
    db_pool: Data<sqlx::PgPool>,
) -> Result<impl Responder> {
    // Validate new password
    PasswordValidator::validate(&req.new_password)?;

    // Get reset token from Redis
    let reset_token = state.storage.get_password_reset_token(&req.token).await?
        .ok_or(AuthError::InvalidToken("Invalid or expired reset token".to_string()))?;

    // Check if token is expired
    if reset_token.is_expired() {
        state.storage.delete_password_reset_token(&req.token).await?;
        return Err(AuthError::InvalidToken("Reset token expired".to_string()));
    }

    let user_repo = PostgresUserRepository::new(db_pool.get_ref().clone());

    // Parse user_id
    let user_id = Uuid::parse_str(&reset_token.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user ID: {}", e)))?;

    // Hash new password
    let new_password_hash = PasswordHasher::hash_password(&req.new_password)?;

    // Update password in database
    user_repo.update_password(user_id, &new_password_hash).await?;

    // Delete reset token (single-use)
    state.storage.delete_password_reset_token(&req.token).await?;

    // Invalidate all existing sessions for this user (except current if requested)
    let sessions_invalidated = state.session_manager
        .invalidate_all_user_sessions(&user_id, None)
        .await?;

    // Revoke all refresh tokens for this user
    let tokens_revoked = state.token_family_manager
        .revoke_all_user_tokens(&user_id)
        .await
        .unwrap_or(0);

    // TODO: Emit sessions-invalidated event to Kafka
    tracing::info!(
        user_id = %user_id,
        email = %reset_token.email,
        sessions_invalidated = %sessions_invalidated,
        tokens_revoked = %tokens_revoked,
        "Password reset successful"
    );

    // Send "password changed" notification email
    if let Err(e) = state.email_manager.send_password_changed_notification(
        reset_token.email.clone(),
    ).await {
        tracing::error!("Failed to send password changed notification: {}", e);
        // Don't fail the request, password was already changed
    } else {
        tracing::info!("Password changed notification sent to: {}", reset_token.email);
    }

    Ok(HttpResponse::Ok().json(ResetPasswordResponse {
        message: "Password has been reset successfully. All sessions have been invalidated.".to_string(),
        sessions_invalidated,
        tokens_revoked,
    }))
}
