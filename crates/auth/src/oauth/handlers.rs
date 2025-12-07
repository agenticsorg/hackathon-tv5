use crate::error::{AuthError, Result};
use crate::oauth::pkce::PkceChallenge;
use crate::oauth::providers::{AppleOAuthProvider, GitHubOAuthProvider, GoogleOAuthProvider};
use crate::storage::AuthStorage;
use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use std::sync::Arc;

/// Google OAuth authorization endpoint
#[get("/auth/oauth/google/authorize")]
pub async fn google_authorize(storage: web::Data<Arc<AuthStorage>>) -> Result<impl Responder> {
    // Load Google provider
    let provider = GoogleOAuthProvider::from_env()?;

    // Generate PKCE challenge
    let pkce = PkceChallenge::generate();

    // Store PKCE session in Redis
    storage.store_pkce(&pkce.state, &pkce).await?;

    // Generate authorization URL
    let auth_url = provider.generate_authorization_url(&pkce);

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

/// Google OAuth callback query parameters
#[derive(Debug, Deserialize)]
pub struct GoogleCallbackQuery {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// Google OAuth callback endpoint
#[get("/auth/oauth/google/callback")]
pub async fn google_callback(
    query: web::Query<GoogleCallbackQuery>,
    storage: web::Data<Arc<AuthStorage>>,
) -> Result<impl Responder> {
    // Check for OAuth errors
    if let Some(error) = &query.error {
        tracing::error!(
            "Google OAuth error: {} - {:?}",
            error,
            query.error_description
        );
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": error,
            "error_description": query.error_description.as_deref().unwrap_or("Unknown error")
        })));
    }

    // Retrieve PKCE session
    let pkce = storage
        .get_pkce(&query.state)
        .await?
        .ok_or_else(|| AuthError::Internal("PKCE session not found or expired".to_string()))?;

    // Verify state parameter matches
    if pkce.state != query.state {
        tracing::error!(
            "State mismatch: expected={}, received={}",
            pkce.state,
            query.state
        );
        return Err(AuthError::Internal("State mismatch".to_string()));
    }

    // Load Google provider
    let provider = GoogleOAuthProvider::from_env()?;

    // Exchange authorization code for access token
    let token_response = provider
        .exchange_code(&query.code, &pkce.code_verifier)
        .await?;

    // Get user profile
    let user_profile = provider
        .get_user_profile(&token_response.access_token)
        .await?;

    // Clean up PKCE session
    storage.delete_pkce(&query.state).await?;

    // Return user profile and tokens
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_profile": {
            "id": user_profile.id,
            "email": user_profile.email,
            "verified_email": user_profile.verified_email,
            "name": user_profile.name,
            "given_name": user_profile.given_name,
            "family_name": user_profile.family_name,
            "picture": user_profile.picture,
            "locale": user_profile.locale,
        },
        "access_token": token_response.access_token,
        "expires_in": token_response.expires_in,
        "refresh_token": token_response.refresh_token,
        "scope": token_response.scope,
        "token_type": token_response.token_type,
    })))
}

/// GitHub OAuth authorization endpoint
#[get("/auth/oauth/github/authorize")]
pub async fn github_authorize(storage: web::Data<Arc<AuthStorage>>) -> Result<impl Responder> {
    // Load GitHub provider
    let provider = GitHubOAuthProvider::from_env()?;

    // Generate PKCE challenge
    let pkce = PkceChallenge::generate();

    // Store PKCE session in Redis
    storage.store_pkce(&pkce.state, &pkce).await?;

    // Generate authorization URL
    let auth_url = provider.generate_authorization_url(&pkce);

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

/// GitHub OAuth callback query parameters
#[derive(Debug, Deserialize)]
pub struct GitHubCallbackQuery {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// GitHub OAuth callback endpoint
#[get("/auth/oauth/github/callback")]
pub async fn github_callback(
    query: web::Query<GitHubCallbackQuery>,
    storage: web::Data<Arc<AuthStorage>>,
) -> Result<impl Responder> {
    // Check for OAuth errors
    if let Some(error) = &query.error {
        tracing::error!(
            "GitHub OAuth error: {} - {:?}",
            error,
            query.error_description
        );
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": error,
            "error_description": query.error_description.as_deref().unwrap_or("Unknown error")
        })));
    }

    // Retrieve PKCE session
    let pkce = storage
        .get_pkce(&query.state)
        .await?
        .ok_or_else(|| AuthError::Internal("PKCE session not found or expired".to_string()))?;

    // Verify state parameter matches
    if pkce.state != query.state {
        tracing::error!(
            "State mismatch: expected={}, received={}",
            pkce.state,
            query.state
        );
        return Err(AuthError::Internal("State mismatch".to_string()));
    }

    // Load GitHub provider
    let provider = GitHubOAuthProvider::from_env()?;

    // Exchange authorization code for access token
    let token_response = provider
        .exchange_code(&query.code, &pkce.code_verifier)
        .await?;

    // Get user profile
    let user_profile = provider
        .get_user_profile(&token_response.access_token)
        .await?;

    // Get verified email
    let verified_email = provider
        .get_verified_email(&token_response.access_token)
        .await
        .unwrap_or_else(|_| {
            user_profile
                .email
                .clone()
                .unwrap_or_else(|| format!("{}@github.local", user_profile.login))
        });

    // Clean up PKCE session
    storage.delete_pkce(&query.state).await?;

    // Return user profile and tokens
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_profile": {
            "id": user_profile.id,
            "login": user_profile.login,
            "email": verified_email,
            "name": user_profile.name,
            "avatar_url": user_profile.avatar_url,
            "bio": user_profile.bio,
            "location": user_profile.location,
            "company": user_profile.company,
            "blog": user_profile.blog,
        },
        "access_token": token_response.access_token,
        "scope": token_response.scope,
        "token_type": token_response.token_type,
    })))
}

/// Apple OAuth authorization endpoint
#[get("/auth/oauth/apple/authorize")]
pub async fn apple_authorize(storage: web::Data<Arc<AuthStorage>>) -> Result<impl Responder> {
    // Load Apple provider
    let provider = AppleOAuthProvider::from_env()?;

    // Generate PKCE challenge
    let pkce = PkceChallenge::generate();

    // Store PKCE session in Redis
    storage.store_pkce(&pkce.state, &pkce).await?;

    // Generate authorization URL
    let auth_url = provider.generate_authorization_url(&pkce);

    Ok(HttpResponse::Found()
        .append_header(("Location", auth_url))
        .finish())
}

/// Apple OAuth callback query parameters (form_post)
#[derive(Debug, Deserialize)]
pub struct AppleCallbackForm {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub id_token: Option<String>,
    pub user: Option<String>,
}

/// Apple OAuth callback endpoint (POST due to response_mode=form_post)
#[post("/auth/oauth/apple/callback")]
pub async fn apple_callback(
    form: web::Form<AppleCallbackForm>,
    storage: web::Data<Arc<AuthStorage>>,
) -> Result<impl Responder> {
    // Check for OAuth errors
    if let Some(error) = &form.error {
        tracing::error!("Apple OAuth error: {}", error);
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": error,
        })));
    }

    // Retrieve PKCE session
    let pkce = storage
        .get_pkce(&form.state)
        .await?
        .ok_or_else(|| AuthError::Internal("PKCE session not found or expired".to_string()))?;

    // Verify state parameter matches
    if pkce.state != form.state {
        tracing::error!(
            "State mismatch: expected={}, received={}",
            pkce.state,
            form.state
        );
        return Err(AuthError::Internal("State mismatch".to_string()));
    }

    // Load Apple provider
    let provider = AppleOAuthProvider::from_env()?;

    // Exchange authorization code for access token
    let token_response = provider
        .exchange_code(&form.code, &pkce.code_verifier)
        .await?;

    // Extract user info from ID token
    let user_info = provider.get_user_info(&token_response)?;

    // Clean up PKCE session
    storage.delete_pkce(&form.state).await?;

    // Return user profile and tokens
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "user_profile": {
            "id": user_info.id,
            "email": user_info.email,
            "email_verified": user_info.email_verified,
            "is_private_email": user_info.is_private_email,
        },
        "access_token": token_response.access_token,
        "expires_in": token_response.expires_in,
        "refresh_token": token_response.refresh_token,
        "token_type": token_response.token_type,
    })))
}
