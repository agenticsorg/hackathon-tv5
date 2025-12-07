use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::parental::controls::get_parental_controls;

const VERIFICATION_DURATION_MINUTES: i64 = 5;
const REDIS_KEY_PREFIX: &str = "parental_pin_verified";

/// Request to verify PIN
#[derive(Debug, Deserialize)]
pub struct VerifyPinRequest {
    pub pin: String,
}

/// Response after PIN verification
#[derive(Debug, Serialize)]
pub struct VerifyPinResponse {
    pub verified: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub expires_at: Option<i64>, // Unix timestamp
}

/// Claims for PIN verification JWT
#[derive(Debug, Serialize, Deserialize)]
struct VerificationClaims {
    sub: String, // user_id
    exp: i64,    // expiration timestamp
    iat: i64,    // issued at
    purpose: String,
}

/// Verify PIN and generate verification token
pub async fn verify_pin(
    pool: &PgPool,
    redis_client: &redis::Client,
    user_id: Uuid,
    request: VerifyPinRequest,
    jwt_secret: &str,
) -> Result<VerifyPinResponse, AuthError> {
    // Get parental controls
    let controls = get_parental_controls(pool, user_id).await?.ok_or_else(|| {
        AuthError::ValidationError("Parental controls not configured".to_string())
    })?;

    if !controls.enabled {
        return Err(AuthError::ValidationError(
            "Parental controls not enabled".to_string(),
        ));
    }

    // Verify PIN
    let is_valid = controls.verify_pin(&request.pin)?;

    if !is_valid {
        return Ok(VerifyPinResponse {
            verified: false,
            token: None,
            expires_at: None,
        });
    }

    // Generate verification token
    let now = Utc::now();
    let expiry = now + Duration::minutes(VERIFICATION_DURATION_MINUTES);

    let claims = VerificationClaims {
        sub: user_id.to_string(),
        exp: expiry.timestamp(),
        iat: now.timestamp(),
        purpose: "parental_pin_verification".to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_bytes()),
    )
    .map_err(|e| AuthError::InternalError(format!("Failed to generate token: {}", e)))?;

    // Store verification in Redis
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AuthError::InternalError(format!("Redis connection failed: {}", e)))?;

    let redis_key = format!("{}:{}", REDIS_KEY_PREFIX, user_id);
    let ttl = VERIFICATION_DURATION_MINUTES * 60;

    conn.set_ex::<_, _, ()>(&redis_key, "1", ttl as u64)
        .await
        .map_err(|e| {
            AuthError::InternalError(format!("Failed to store verification in Redis: {}", e))
        })?;

    Ok(VerifyPinResponse {
        verified: true,
        token: Some(token),
        expires_at: Some(expiry.timestamp()),
    })
}

/// Check if PIN is currently verified (from Redis cache)
pub async fn is_pin_verified(
    redis_client: &redis::Client,
    user_id: Uuid,
) -> Result<bool, AuthError> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AuthError::InternalError(format!("Redis connection failed: {}", e)))?;

    let redis_key = format!("{}:{}", REDIS_KEY_PREFIX, user_id);

    let exists: bool = conn.exists(&redis_key).await.map_err(|e| {
        AuthError::InternalError(format!("Failed to check verification in Redis: {}", e))
    })?;

    Ok(exists)
}

/// Verify PIN verification token
pub async fn verify_token(token: &str, jwt_secret: &str) -> Result<Uuid, AuthError> {
    let token_data = decode::<VerificationClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AuthError::InvalidToken(format!("Invalid verification token: {}", e)))?;

    if token_data.claims.purpose != "parental_pin_verification" {
        return Err(AuthError::InvalidToken(
            "Token not for parental PIN verification".to_string(),
        ));
    }

    Uuid::parse_str(&token_data.claims.sub)
        .map_err(|e| AuthError::InvalidToken(format!("Invalid user ID in token: {}", e)))
}

/// Clear PIN verification (e.g., on logout)
pub async fn clear_verification(
    redis_client: &redis::Client,
    user_id: Uuid,
) -> Result<(), AuthError> {
    let mut conn = redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AuthError::InternalError(format!("Redis connection failed: {}", e)))?;

    let redis_key = format!("{}:{}", REDIS_KEY_PREFIX, user_id);

    conn.del::<_, ()>(&redis_key).await.map_err(|e| {
        AuthError::InternalError(format!("Failed to clear verification in Redis: {}", e))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verification_claims_serialization() {
        let claims = VerificationClaims {
            sub: "user123".to_string(),
            exp: 1234567890,
            iat: 1234567800,
            purpose: "parental_pin_verification".to_string(),
        };

        let json = serde_json::to_string(&claims).unwrap();
        let deserialized: VerificationClaims = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.sub, "user123");
        assert_eq!(deserialized.purpose, "parental_pin_verification");
    }

    #[test]
    fn test_redis_key_format() {
        let user_id = Uuid::new_v4();
        let key = format!("{}:{}", REDIS_KEY_PREFIX, user_id);
        assert!(key.starts_with(REDIS_KEY_PREFIX));
        assert!(key.contains(&user_id.to_string()));
    }
}
