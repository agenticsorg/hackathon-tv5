use media_gateway_auth::{
    api_keys, email, jwt::JwtManager, mfa, middleware::RateLimitConfig, oauth::OAuthConfig,
    server::start_server, session::SessionManager, storage::AuthStorage,
    token_family::TokenFamilyManager,
};
use sqlx::postgres::PgPoolOptions;
use std::{env, fs, sync::Arc};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .json()
        .init();

    tracing::info!("Starting Media Gateway Auth Service");

    // Load configuration
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8084".to_string());
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    // Load JWT keys
    let private_key_path = env::var("JWT_PRIVATE_KEY_PATH")
        .unwrap_or_else(|_| "/secrets/jwt_private_key.pem".to_string());
    let public_key_path = env::var("JWT_PUBLIC_KEY_PATH")
        .unwrap_or_else(|_| "/secrets/jwt_public_key.pem".to_string());

    let private_key = fs::read(&private_key_path).expect("Failed to read JWT private key");
    let public_key = fs::read(&public_key_path).expect("Failed to read JWT public key");

    let jwt_issuer =
        env::var("JWT_ISSUER").unwrap_or_else(|_| "https://api.mediagateway.io".to_string());
    let jwt_audience =
        env::var("JWT_AUDIENCE").unwrap_or_else(|_| "mediagateway-users".to_string());

    // Initialize JWT manager
    let jwt_manager = Arc::new(
        JwtManager::new(&private_key, &public_key, jwt_issuer, jwt_audience)
            .expect("Failed to initialize JWT manager"),
    );

    // Initialize session manager
    let session_manager =
        Arc::new(SessionManager::new(&redis_url).expect("Failed to initialize session manager"));

    // Initialize token family manager
    let token_family_manager = Arc::new(
        TokenFamilyManager::new(&redis_url).expect("Failed to initialize token family manager"),
    );

    // Initialize OAuth config (load from environment)
    let mut providers = std::collections::HashMap::new();

    // Add Google OAuth provider if configured
    if let (Ok(client_id), Ok(client_secret)) = (
        env::var("GOOGLE_CLIENT_ID"),
        env::var("GOOGLE_CLIENT_SECRET"),
    ) {
        let redirect_uri = env::var("GOOGLE_REDIRECT_URI").unwrap_or_else(|_| {
            "https://api.mediagateway.io/auth/oauth/google/callback".to_string()
        });

        providers.insert(
            "google".to_string(),
            media_gateway_auth::oauth::OAuthProvider {
                client_id,
                client_secret,
                authorization_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                redirect_uri,
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
            },
        );
        tracing::info!("Google OAuth provider configured");
    }

    let oauth_config = OAuthConfig { providers };

    // Initialize auth storage (Redis-backed)
    let auth_storage =
        Arc::new(AuthStorage::new(&redis_url).expect("Failed to initialize auth storage"));

    // Initialize PostgreSQL connection pool
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/media_gateway".to_string()
    });
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database connection pool");

    // Initialize Redis client for rate limiting
    let redis_client = redis::Client::open(redis_url.as_str())
        .expect("Failed to create Redis client for rate limiting");

    // Configure rate limits
    let rate_limit_config = RateLimitConfig::new(
        env::var("RATE_LIMIT_TOKEN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
        env::var("RATE_LIMIT_DEVICE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5),
        env::var("RATE_LIMIT_AUTHORIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(20),
        env::var("RATE_LIMIT_REVOKE")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
        env::var("RATE_LIMIT_REGISTER")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5),
        env::var("RATE_LIMIT_LOGIN")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10),
    );

    let rate_limit_config = if let Ok(secret) = env::var("INTERNAL_SERVICE_SECRET") {
        rate_limit_config.with_internal_secret(secret)
    } else {
        rate_limit_config
    };

    tracing::info!("Rate limiting configured: token={}, device={}, authorize={}, revoke={}, register={}, login={}",
        rate_limit_config.token_endpoint_limit,
        rate_limit_config.device_endpoint_limit,
        rate_limit_config.authorize_endpoint_limit,
        rate_limit_config.revoke_endpoint_limit,
        rate_limit_config.register_endpoint_limit,
        rate_limit_config.login_endpoint_limit
    );

    // Initialize optional managers (None for now, can be configured via env vars)
    let mfa_manager: Option<Arc<mfa::MfaManager>> = None;
    let api_key_manager: Option<Arc<api_keys::ApiKeyManager>> = None;
    let email_manager: Option<Arc<email::EmailManager>> = None;

    // Start server with rate limiting
    start_server(
        &bind_address,
        jwt_manager,
        session_manager,
        token_family_manager,
        oauth_config,
        auth_storage,
        redis_client,
        rate_limit_config,
        mfa_manager,
        api_key_manager,
        email_manager,
        db_pool,
    )
    .await
}
