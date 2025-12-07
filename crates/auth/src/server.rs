use crate::{
    admin::{
        delete_user, get_audit_logs, get_user_detail, impersonate_user, list_users, update_user,
    },
    api_keys::{ApiKeyManager, CreateApiKeyRequest},
    error::{AuthError, Result},
    jwt::JwtManager,
    mfa::MfaManager,
    middleware::{extract_user_context, RateLimitConfig, RateLimitMiddleware},
    oauth::{
        device::{DeviceAuthorizationResponse, DeviceCode},
        handlers::{apple_authorize, apple_callback, google_authorize, google_callback},
        pkce::PkceChallenge,
        OAuthConfig, OAuthManager,
    },
    parental::{update_parental_controls, verify_parental_pin, ParentalControlsState},
    password_reset::{
        ForgotPasswordRequest, ForgotPasswordResponse, PasswordResetToken, PasswordValidator,
        ResetPasswordRequest, ResetPasswordResponse,
    },
    profile::{
        delete_current_user, get_current_user, handlers::ProfileState, update_current_user,
        upload_avatar, ProfileStorage,
    },
    rate_limit_admin_handlers::{
        delete_rate_limit, get_rate_limit, list_rate_limits, update_rate_limit,
    },
    rate_limit_config::RateLimitConfigStore,
    rbac::RbacManager,
    scopes::ScopeManager,
    session::SessionManager,
    storage::AuthStorage,
    token_family::TokenFamilyManager,
    user::{login, register, PasswordHasher, PostgresUserRepository, UserHandlerState},
};
use actix_web::{
    delete, get, post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Application state shared across handlers
pub struct AppState {
    pub jwt_manager: Arc<JwtManager>,
    pub session_manager: Arc<SessionManager>,
    pub oauth_manager: Arc<OAuthManager>,
    pub rbac_manager: Arc<RbacManager>,
    pub scope_manager: Arc<ScopeManager>,
    pub storage: Arc<AuthStorage>,
    pub token_family_manager: Arc<TokenFamilyManager>,
    pub mfa_manager: Option<Arc<MfaManager>>,
    pub api_key_manager: Option<Arc<ApiKeyManager>>,
    pub email_manager: Option<Arc<crate::email::EmailManager>>,
}

// ============================================================================
// Health Check
// ============================================================================

#[get("/health")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "auth-service",
        "version": "0.1.0"
    }))
}

// ============================================================================
// OAuth 2.0 Authorization Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
struct AuthorizeRequest {
    client_id: String,
    redirect_uri: String,
    response_type: String,
    scope: String,
    code_challenge: String,
    code_challenge_method: String,
    state: Option<String>,
}

#[get("/auth/authorize")]
async fn authorize(
    query: web::Query<AuthorizeRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    // Validate response_type
    if query.response_type != "code" {
        return Err(AuthError::InvalidClient);
    }

    // Validate code_challenge_method
    if query.code_challenge_method != "S256" {
        return Err(AuthError::InvalidPkceVerifier);
    }

    // Validate client and redirect URI
    state
        .oauth_manager
        .validate_redirect_uri(&query.client_id, &query.redirect_uri)?;

    // Parse and validate scopes
    let scopes: Vec<String> = query
        .scope
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // Store PKCE session
    let pkce = PkceChallenge {
        code_verifier: String::new(), // Client keeps this
        code_challenge: query.code_challenge.clone(),
        code_challenge_method: query.code_challenge_method.clone(),
        state: query.state.clone().unwrap_or_else(|| "".to_string()),
    };

    let session_state = pkce.state.clone();
    state.storage.store_pkce(&session_state, &pkce).await?;

    // In a real implementation, redirect to login page
    // For now, return authorization page URL
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Authorization flow initiated",
        "state": session_state,
        "next_step": "User must complete authentication and consent"
    })))
}

// ============================================================================
// Token Exchange Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
struct TokenRequest {
    grant_type: String,
    code: Option<String>,
    code_verifier: Option<String>,
    redirect_uri: Option<String>,
    client_id: Option<String>,
    refresh_token: Option<String>,
    device_code: Option<String>,
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    token_type: String,
    expires_in: i64,
    scope: String,
}

#[post("/auth/token")]
async fn token_exchange(
    form: web::Form<TokenRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    match form.grant_type.as_str() {
        "authorization_code" => exchange_authorization_code(&form, &state).await,
        "refresh_token" => refresh_access_token(&form, &state).await,
        "urn:ietf:params:oauth:grant-type:device_code" => exchange_device_code(&form, &state).await,
        _ => Err(AuthError::Internal("Unsupported grant type".to_string())),
    }
}

async fn exchange_authorization_code(
    form: &TokenRequest,
    state: &AppState,
) -> Result<HttpResponse> {
    let code = form.code.as_ref().ok_or(AuthError::InvalidAuthCode)?;
    let verifier = form
        .code_verifier
        .as_ref()
        .ok_or(AuthError::InvalidPkceVerifier)?;
    let redirect_uri = form
        .redirect_uri
        .as_ref()
        .ok_or(AuthError::InvalidRedirectUri)?;
    let client_id = form.client_id.as_ref().ok_or(AuthError::InvalidClient)?;

    // Retrieve authorization code
    let mut auth_code = state
        .storage
        .get_auth_code(code)
        .await?
        .ok_or(AuthError::InvalidAuthCode)?;

    // Check if already used
    if auth_code.used {
        tracing::error!("Authorization code reuse detected: {}", code);
        return Err(AuthError::AuthCodeReused);
    }

    // Check expiration
    if auth_code.is_expired() {
        state.storage.delete_auth_code(code).await?;
        return Err(AuthError::InvalidAuthCode);
    }

    // Verify PKCE
    auth_code.verify_pkce(verifier)?;

    // Verify client_id and redirect_uri
    if &auth_code.client_id != client_id || &auth_code.redirect_uri != redirect_uri {
        return Err(AuthError::InvalidClient);
    }

    // Mark as used
    auth_code.mark_as_used();
    state.storage.update_auth_code(code, &auth_code).await?;

    // Generate tokens with token family
    let access_token = state.jwt_manager.create_access_token(
        auth_code.user_id.clone(),
        Some(format!("user{}@example.com", auth_code.user_id)),
        vec!["free_user".to_string()],
        auth_code.scopes.clone(),
    )?;

    // Create new token family for this authorization
    let user_uuid = uuid::Uuid::parse_str(&auth_code.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user_id format: {}", e)))?;
    let family_id = state.token_family_manager.create_family(user_uuid).await?;

    let refresh_token = state.jwt_manager.create_refresh_token_with_family(
        auth_code.user_id.clone(),
        Some(format!("user{}@example.com", auth_code.user_id)),
        vec!["free_user".to_string()],
        auth_code.scopes.clone(),
        family_id,
    )?;

    // Create session and add token to family
    let refresh_claims = state.jwt_manager.verify_refresh_token(&refresh_token)?;
    state
        .token_family_manager
        .add_token_to_family(family_id, &refresh_claims.jti)
        .await?;
    state
        .session_manager
        .create_session(auth_code.user_id.clone(), refresh_claims.jti, None)
        .await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: auth_code.scopes.join(" "),
    }))
}

async fn refresh_access_token(form: &TokenRequest, state: &AppState) -> Result<HttpResponse> {
    let refresh_token = form
        .refresh_token
        .as_ref()
        .ok_or(AuthError::InvalidToken("Missing refresh token".to_string()))?;

    // Verify refresh token
    let claims = state.jwt_manager.verify_refresh_token(refresh_token)?;

    // Check if revoked
    if state.session_manager.is_token_revoked(&claims.jti).await? {
        return Err(AuthError::InvalidToken("Token revoked".to_string()));
    }

    // Extract token family ID
    let family_id = claims.token_family_id.ok_or(AuthError::InvalidToken(
        "Token missing family ID (legacy token)".to_string(),
    ))?;

    // SECURITY CHECK: Verify token is in its family
    let is_in_family = state
        .token_family_manager
        .is_token_in_family(family_id, &claims.jti)
        .await?;

    if !is_in_family {
        // SECURITY EVENT: Token reuse detected - revoke entire family
        tracing::error!(
            user_id = %claims.sub,
            family_id = %family_id,
            attempted_jti = %claims.jti,
            "Token reuse detected - revoking entire token family"
        );

        // Revoke all tokens in the family
        state.token_family_manager.revoke_family(family_id).await?;

        return Err(AuthError::InvalidToken(
            "Token reuse detected. All tokens in this family have been revoked.".to_string(),
        ));
    }

    // Generate new tokens with same family
    let new_access_token = state.jwt_manager.create_access_token(
        claims.sub.clone(),
        claims.email.clone(),
        claims.roles.clone(),
        claims.scopes.clone(),
    )?;

    let new_refresh_token = state.jwt_manager.create_refresh_token_with_family(
        claims.sub.clone(),
        claims.email.clone(),
        claims.roles.clone(),
        claims.scopes.clone(),
        family_id,
    )?;

    // Remove old JTI from family and revoke it
    state
        .token_family_manager
        .remove_token_from_family(family_id, &claims.jti)
        .await?;
    state
        .session_manager
        .revoke_token(&claims.jti, 3600)
        .await?;

    // Add new JTI to family
    let new_refresh_claims = state.jwt_manager.verify_refresh_token(&new_refresh_token)?;
    state
        .token_family_manager
        .add_token_to_family(family_id, &new_refresh_claims.jti)
        .await?;

    // Create new session
    let new_jti = new_refresh_claims.jti.clone();
    state
        .session_manager
        .create_session(claims.sub.clone(), new_refresh_claims.jti, None)
        .await?;

    tracing::debug!(
        user_id = %claims.sub,
        family_id = %family_id,
        old_jti = %claims.jti,
        new_jti = %new_jti,
        "Successfully rotated refresh token"
    );

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token: new_access_token,
        refresh_token: new_refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: claims.scopes.join(" "),
    }))
}

async fn exchange_device_code(form: &TokenRequest, state: &AppState) -> Result<HttpResponse> {
    let device_code = form
        .device_code
        .as_ref()
        .ok_or(AuthError::DeviceCodeNotFound)?;

    // Retrieve device code
    let device = state
        .storage
        .get_device_code(device_code)
        .await?
        .ok_or(AuthError::DeviceCodeNotFound)?;

    // Check status - will error if pending
    device.check_status()?;

    let user_id = device
        .user_id
        .clone()
        .ok_or(AuthError::Internal("User ID not found".to_string()))?;

    // Generate tokens with token family
    let access_token = state.jwt_manager.create_access_token(
        user_id.clone(),
        Some(format!("user{}@example.com", user_id)),
        vec!["free_user".to_string()],
        device.scopes.clone(),
    )?;

    // Create new token family for this device authorization
    let user_uuid = uuid::Uuid::parse_str(&user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user_id format: {}", e)))?;
    let family_id = state.token_family_manager.create_family(user_uuid).await?;

    let refresh_token = state.jwt_manager.create_refresh_token_with_family(
        user_id.clone(),
        Some(format!("user{}@example.com", user_id)),
        vec!["free_user".to_string()],
        device.scopes.clone(),
        family_id,
    )?;

    // Create session and add token to family
    let refresh_claims = state.jwt_manager.verify_refresh_token(&refresh_token)?;
    state
        .token_family_manager
        .add_token_to_family(family_id, &refresh_claims.jti)
        .await?;
    state
        .session_manager
        .create_session(user_id.clone(), refresh_claims.jti, None)
        .await?;

    // Delete device code after successful token issuance
    state.storage.delete_device_code(device_code).await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: device.scopes.join(" "),
    }))
}

// ============================================================================
// Token Revocation Endpoint
// ============================================================================

#[derive(Debug, Deserialize)]
struct RevokeRequest {
    token: String,
    token_type_hint: Option<String>,
}

#[post("/auth/revoke")]
async fn revoke_token(
    form: web::Form<RevokeRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    // Try to decode as access or refresh token
    let claims = state
        .jwt_manager
        .verify_token(&form.token)
        .or_else(|_| state.jwt_manager.verify_refresh_token(&form.token))?;

    // Revoke token
    state
        .session_manager
        .revoke_token(&claims.jti, 3600)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Token revoked successfully"
    })))
}

// ============================================================================
// Device Authorization Endpoint (RFC 8628)
// ============================================================================

#[derive(Debug, Deserialize)]
struct DeviceAuthRequest {
    client_id: String,
    scope: Option<String>,
}

#[post("/auth/device")]
async fn device_authorization(
    form: web::Form<DeviceAuthRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let scopes = form
        .scope
        .as_ref()
        .map(|s| s.split_whitespace().map(|x| x.to_string()).collect())
        .unwrap_or_default();

    let device = DeviceCode::new(
        form.client_id.clone(),
        scopes,
        "https://auth.mediagateway.io",
    );

    let response = DeviceAuthorizationResponse::from(&device);

    // Store device code
    state
        .storage
        .store_device_code(&device.device_code, &device)
        .await?;

    Ok(HttpResponse::Ok().json(response))
}

#[derive(Debug, Deserialize)]
struct DeviceApprovalRequest {
    user_code: String,
}

#[post("/auth/device/approve")]
async fn approve_device(
    req: web::Json<DeviceApprovalRequest>,
    auth_header: web::Header<
        actix_web_httpauth::headers::authorization::Authorization<
            actix_web_httpauth::headers::authorization::Bearer,
        >,
    >,
    state: Data<AppState>,
) -> Result<impl Responder> {
    // Extract and verify JWT token
    let token = auth_header.as_ref().token();
    let claims = state.jwt_manager.verify_access_token(token)?;

    // Check if token is revoked
    if state.session_manager.is_token_revoked(&claims.jti).await? {
        return Err(AuthError::Unauthorized);
    }

    let user_id = claims.sub;

    // Look up device code by user_code
    let mut device = state
        .storage
        .get_device_code_by_user_code(&req.user_code)
        .await?
        .ok_or(AuthError::InvalidUserCode)?;

    // Verify device is in Pending state
    if device.is_expired() {
        state
            .storage
            .delete_device_code(&device.device_code)
            .await?;
        return Err(AuthError::DeviceCodeExpired);
    }

    if device.status != crate::oauth::device::DeviceCodeStatus::Pending {
        return Err(AuthError::DeviceAlreadyApproved);
    }

    // Approve device with user_id binding
    device.approve(user_id);

    // Update Redis with new state
    state
        .storage
        .update_device_code(&device.device_code, &device)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "Device authorization approved",
        "user_code": device.user_code
    })))
}

#[get("/auth/device/poll")]
async fn device_poll(
    query: web::Query<std::collections::HashMap<String, String>>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let device_code = query
        .get("device_code")
        .ok_or(AuthError::DeviceCodeNotFound)?;

    let device = state
        .storage
        .get_device_code(device_code)
        .await?
        .ok_or(AuthError::DeviceCodeNotFound)?;

    // Check status - this will return error if still pending
    device.check_status()?;

    // If we reach here, device is approved - generate tokens with token family
    let user_id = device
        .user_id
        .clone()
        .ok_or(AuthError::Internal("User ID not found".to_string()))?;

    // Generate tokens with token family
    let access_token = state.jwt_manager.create_access_token(
        user_id.clone(),
        Some(format!("user{}@example.com", user_id)),
        vec!["free_user".to_string()],
        device.scopes.clone(),
    )?;

    // Create new token family for this device authorization
    let user_uuid = uuid::Uuid::parse_str(&user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user_id format: {}", e)))?;
    let family_id = state.token_family_manager.create_family(user_uuid).await?;

    let refresh_token = state.jwt_manager.create_refresh_token_with_family(
        user_id.clone(),
        Some(format!("user{}@example.com", user_id)),
        vec!["free_user".to_string()],
        device.scopes.clone(),
        family_id,
    )?;

    // Create session and add token to family
    let refresh_claims = state.jwt_manager.verify_refresh_token(&refresh_token)?;
    state
        .token_family_manager
        .add_token_to_family(family_id, &refresh_claims.jti)
        .await?;
    state
        .session_manager
        .create_session(user_id.clone(), refresh_claims.jti, None)
        .await?;

    // Delete device code after successful token issuance
    state.storage.delete_device_code(device_code).await?;

    Ok(HttpResponse::Ok().json(TokenResponse {
        access_token,
        refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        scope: device.scopes.join(" "),
    }))
}

// ============================================================================
// API Key Management Endpoints
// ============================================================================

#[post("/api/v1/auth/api-keys")]
async fn create_api_key(
    req: actix_web::HttpRequest,
    body: web::Json<CreateApiKeyRequest>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let api_key_manager = state.api_key_manager.as_ref().ok_or(AuthError::Internal(
        "API key manager not configured".to_string(),
    ))?;
    let user_context = extract_user_context(&req)?;

    let api_key = api_key_manager
        .create_api_key(
            Uuid::parse_str(&user_context.user_id).unwrap(),
            body.into_inner(),
        )
        .await?;

    Ok(HttpResponse::Created().json(api_key))
}

#[get("/api/v1/auth/api-keys")]
async fn list_api_keys(
    req: actix_web::HttpRequest,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let api_key_manager = state.api_key_manager.as_ref().ok_or(AuthError::Internal(
        "API key manager not configured".to_string(),
    ))?;
    let user_context = extract_user_context(&req)?;

    let keys = api_key_manager
        .list_user_keys(Uuid::parse_str(&user_context.user_id).unwrap())
        .await?;

    Ok(HttpResponse::Ok().json(keys))
}

#[delete("/api/v1/auth/api-keys/{key_id}")]
async fn revoke_api_key(
    req: actix_web::HttpRequest,
    path: web::Path<Uuid>,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let api_key_manager = state.api_key_manager.as_ref().ok_or(AuthError::Internal(
        "API key manager not configured".to_string(),
    ))?;
    let user_context = extract_user_context(&req)?;
    let key_id = path.into_inner();

    api_key_manager
        .revoke_key(Uuid::parse_str(&user_context.user_id).unwrap(), key_id)
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "API key revoked successfully"
    })))
}

// ============================================================================
// MFA Endpoints
// ============================================================================

#[derive(Debug, Deserialize)]
struct MfaEnrollRequest {
    // Empty - user_id extracted from JWT
}

#[derive(Debug, Serialize)]
struct MfaEnrollResponse {
    qr_code: String,
    backup_codes: Vec<String>,
}

#[post("/api/v1/auth/mfa/enroll")]
async fn mfa_enroll(
    auth_header: web::Header<
        actix_web_httpauth::headers::authorization::Authorization<
            actix_web_httpauth::headers::authorization::Bearer,
        >,
    >,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let mfa_manager = state
        .mfa_manager
        .as_ref()
        .ok_or(AuthError::Internal("MFA not configured".to_string()))?;

    // Extract and verify JWT token
    let token = auth_header.as_ref().token();
    let claims = state.jwt_manager.verify_access_token(token)?;

    // Check if token is revoked
    if state.session_manager.is_token_revoked(&claims.jti).await? {
        return Err(AuthError::Unauthorized);
    }

    let user_id = claims.sub;

    // Initiate MFA enrollment
    let (_secret, qr_code, backup_codes) = mfa_manager.initiate_enrollment(user_id).await?;

    Ok(HttpResponse::Ok().json(MfaEnrollResponse {
        qr_code,
        backup_codes,
    }))
}

#[derive(Debug, Deserialize)]
struct MfaVerifyRequest {
    code: String,
}

#[post("/api/v1/auth/mfa/verify")]
async fn mfa_verify(
    req: web::Json<MfaVerifyRequest>,
    auth_header: web::Header<
        actix_web_httpauth::headers::authorization::Authorization<
            actix_web_httpauth::headers::authorization::Bearer,
        >,
    >,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let mfa_manager = state
        .mfa_manager
        .as_ref()
        .ok_or(AuthError::Internal("MFA not configured".to_string()))?;

    // Extract and verify JWT token
    let token = auth_header.as_ref().token();
    let claims = state.jwt_manager.verify_access_token(token)?;

    // Check if token is revoked
    if state.session_manager.is_token_revoked(&claims.jti).await? {
        return Err(AuthError::Unauthorized);
    }

    let user_id = claims.sub;

    // Check rate limit
    let remaining = state.storage.check_mfa_rate_limit(&user_id).await?;
    if remaining == 0 {
        return Err(AuthError::RateLimitExceeded);
    }

    // Verify enrollment code
    mfa_manager.verify_enrollment(&user_id, &req.code).await?;

    // Reset rate limit on success
    state.storage.reset_mfa_rate_limit(&user_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "MFA enrollment verified successfully"
    })))
}

#[derive(Debug, Deserialize)]
struct MfaChallengeRequest {
    code: String,
}

#[post("/api/v1/auth/mfa/challenge")]
async fn mfa_challenge(
    req: web::Json<MfaChallengeRequest>,
    auth_header: web::Header<
        actix_web_httpauth::headers::authorization::Authorization<
            actix_web_httpauth::headers::authorization::Bearer,
        >,
    >,
    state: Data<AppState>,
) -> Result<impl Responder> {
    let mfa_manager = state
        .mfa_manager
        .as_ref()
        .ok_or(AuthError::Internal("MFA not configured".to_string()))?;

    // Extract and verify JWT token
    let token = auth_header.as_ref().token();
    let claims = state.jwt_manager.verify_access_token(token)?;

    // Check if token is revoked
    if state.session_manager.is_token_revoked(&claims.jti).await? {
        return Err(AuthError::Unauthorized);
    }

    let user_id = claims.sub;

    // Check rate limit
    let remaining = state.storage.check_mfa_rate_limit(&user_id).await?;
    if remaining == 0 {
        return Err(AuthError::RateLimitExceeded);
    }

    // Verify MFA code
    let valid = mfa_manager.verify_challenge(&user_id, &req.code).await?;

    if !valid {
        return Err(AuthError::InvalidMfaCode);
    }

    // Reset rate limit on success
    state.storage.reset_mfa_rate_limit(&user_id).await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "message": "MFA challenge passed",
        "authenticated": true
    })))
}

// ============================================================================
// Password Reset Endpoints
// ============================================================================

#[post("/api/v1/auth/password/forgot")]
async fn forgot_password(
    req: web::Json<ForgotPasswordRequest>,
    state: Data<AppState>,
    db_pool: Data<sqlx::PgPool>,
) -> Result<impl Responder> {
    use crate::user::{PostgresUserRepository, UserRepository};

    let user_repo = PostgresUserRepository::new(db_pool.get_ref().clone());

    // Check rate limit
    let remaining = state
        .storage
        .check_password_reset_rate_limit(&req.email)
        .await?;
    if remaining == 0 {
        // Return success even when rate limited to prevent enumeration
        return Ok(HttpResponse::Ok().json(ForgotPasswordResponse {
            message: "If an account exists with this email, a password reset link has been sent."
                .to_string(),
        }));
    }

    // Find user by email
    let user = user_repo.find_by_email(&req.email).await?;

    // Always return success to prevent email enumeration
    if let Some(user) = user {
        // Generate reset token
        let reset_token = PasswordResetToken::new(user.id.to_string(), user.email.clone());

        // Store token in Redis
        state
            .storage
            .store_password_reset_token(&reset_token.token, &reset_token)
            .await?;

        // Send password reset email
        if let Some(email_manager) = &state.email_manager {
            if let Err(e) = email_manager
                .send_password_reset_email(user.email.clone(), reset_token.token.clone())
                .await
            {
                tracing::error!("Failed to send password reset email: {}", e);
                // Continue anyway - don't expose email sending failures to prevent enumeration
            }
        } else {
            tracing::warn!("Email manager not configured, password reset email not sent");
            tracing::debug!("Reset token for {}: {}", user.email, reset_token.token);
        }

        tracing::info!("Password reset requested for user: {}", user.email);
    }

    Ok(HttpResponse::Ok().json(ForgotPasswordResponse {
        message: "If an account exists with this email, a password reset link has been sent."
            .to_string(),
    }))
}

#[post("/api/v1/auth/password/reset")]
async fn reset_password(
    req: web::Json<ResetPasswordRequest>,
    state: Data<AppState>,
    db_pool: Data<sqlx::PgPool>,
) -> Result<impl Responder> {
    use crate::user::{PasswordHasher, PostgresUserRepository, UserRepository};

    // Validate new password
    PasswordValidator::validate(&req.new_password)?;

    // Get reset token from Redis
    let reset_token = state
        .storage
        .get_password_reset_token(&req.token)
        .await?
        .ok_or(AuthError::InvalidToken(
            "Invalid or expired reset token".to_string(),
        ))?;

    // Check if token is expired
    if reset_token.is_expired() {
        state
            .storage
            .delete_password_reset_token(&req.token)
            .await?;
        return Err(AuthError::InvalidToken("Reset token expired".to_string()));
    }

    let user_repo = PostgresUserRepository::new(db_pool.get_ref().clone());

    // Parse user_id
    let user_id = Uuid::parse_str(&reset_token.user_id)
        .map_err(|e| AuthError::Internal(format!("Invalid user ID: {}", e)))?;

    // Hash new password
    let password_hasher = PasswordHasher::default();
    let new_password_hash = password_hasher.hash_password(&req.new_password)?;

    // Update password in database
    user_repo
        .update_password(user_id, &new_password_hash)
        .await?;

    // Delete reset token (single-use)
    state
        .storage
        .delete_password_reset_token(&req.token)
        .await?;

    // Invalidate all existing sessions for this user (except current if requested)
    let sessions_invalidated = state
        .session_manager
        .invalidate_all_user_sessions(&user_id, None)
        .await
        .unwrap_or(0);

    // Revoke all refresh tokens for this user
    let tokens_revoked = state
        .token_family_manager
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

    // Send password changed notification email
    if let Some(email_manager) = &state.email_manager {
        if let Err(e) = email_manager
            .send_password_changed_notification(reset_token.email.clone())
            .await
        {
            tracing::error!("Failed to send password changed notification: {}", e);
            // Continue anyway - password was already changed successfully
        }
    } else {
        tracing::warn!("Email manager not configured, password changed notification not sent");
    }

    Ok(HttpResponse::Ok().json(ResetPasswordResponse {
        message: "Password has been reset successfully. All sessions have been invalidated."
            .to_string(),
        sessions_invalidated,
        tokens_revoked,
    }))
}

// ============================================================================
// Server Initialization
// ============================================================================

pub async fn start_server(
    bind_address: &str,
    jwt_manager: Arc<JwtManager>,
    session_manager: Arc<SessionManager>,
    token_family_manager: Arc<TokenFamilyManager>,
    oauth_config: OAuthConfig,
    storage: Arc<AuthStorage>,
    redis_client: redis::Client,
    rate_limit_config: RateLimitConfig,
    mfa_manager: Option<Arc<MfaManager>>,
    api_key_manager: Option<Arc<ApiKeyManager>>,
    email_manager: Option<Arc<crate::email::EmailManager>>,
    db_pool: sqlx::PgPool,
) -> std::io::Result<()> {
    let require_email_verification = std::env::var("REQUIRE_EMAIL_VERIFICATION")
        .unwrap_or_else(|_| "false".to_string())
        .parse::<bool>()
        .unwrap_or(false);

    let user_handler_state = Data::new(UserHandlerState {
        user_repository: Arc::new(PostgresUserRepository::new(db_pool.clone())),
        password_hasher: Arc::new(PasswordHasher::new()),
        jwt_manager: jwt_manager.clone(),
        require_email_verification,
    });

    let app_state = Data::new(AppState {
        jwt_manager,
        session_manager,
        oauth_manager: Arc::new(OAuthManager::new(oauth_config)),
        rbac_manager: Arc::new(RbacManager::new()),
        scope_manager: Arc::new(ScopeManager::new()),
        storage,
        token_family_manager,
        mfa_manager,
        api_key_manager,
        email_manager,
    });

    let profile_state = Data::new(ProfileState {
        storage: Arc::new(ProfileStorage::new(db_pool.clone())),
        upload_dir: std::env::var("AVATAR_UPLOAD_DIR")
            .unwrap_or_else(|_| "/tmp/avatars".to_string()),
    });

    let parental_state = Data::new(ParentalControlsState {
        db_pool: db_pool.clone(),
        redis_client: redis_client.clone(),
        jwt_secret: std::env::var("PARENTAL_PIN_JWT_SECRET")
            .unwrap_or_else(|_| "default-parental-pin-secret-change-in-production".to_string()),
    });

    let rate_limit_store = Data::new(Arc::new(RateLimitConfigStore::new(
        redis_client.clone(),
        db_pool.clone(),
    )));

    tracing::info!("Starting auth service on {}", bind_address);

    let db_pool_data = Data::new(db_pool);

    HttpServer::new(move || {
        App::new()
            .wrap(RateLimitMiddleware::new(
                redis_client.clone(),
                rate_limit_config.clone(),
            ))
            .app_data(app_state.clone())
            .app_data(db_pool_data.clone())
            .app_data(user_handler_state.clone())
            .app_data(profile_state.clone())
            .app_data(parental_state.clone())
            .app_data(rate_limit_store.clone())
            .service(health_check)
            .service(authorize)
            .service(token_exchange)
            .service(revoke_token)
            .service(device_authorization)
            .service(approve_device)
            .service(device_poll)
            .service(google_authorize)
            .service(google_callback)
            .service(apple_authorize)
            .service(apple_callback)
            .service(create_api_key)
            .service(list_api_keys)
            .service(revoke_api_key)
            .service(mfa_enroll)
            .service(mfa_verify)
            .service(mfa_challenge)
            .service(list_users)
            .service(get_user_detail)
            .service(update_user)
            .service(delete_user)
            .service(impersonate_user)
            .service(get_audit_logs)
            .service(list_rate_limits)
            .service(get_rate_limit)
            .service(update_rate_limit)
            .service(delete_rate_limit)
            .service(register)
            .service(login)
            .service(get_current_user)
            .service(update_current_user)
            .service(delete_current_user)
            .service(upload_avatar)
            .service(forgot_password)
            .service(reset_password)
            .service(update_parental_controls)
            .service(verify_parental_pin)
    })
    .bind(bind_address)?
    .run()
    .await
}
