use crate::error::{AuthError, Result};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

const ACCESS_TOKEN_TTL: i64 = 3600; // 1 hour
const REFRESH_TOKEN_TTL: i64 = 7 * 24 * 3600; // 7 days

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID
    pub email: Option<String>,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
    pub iat: i64,           // Issued at
    pub exp: i64,           // Expiration
    pub jti: String,        // JWT ID (unique identifier)
    pub token_type: String, // "access" or "refresh"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_family_id: Option<Uuid>, // Token family for refresh token rotation tracking
}

impl Claims {
    pub fn new_access_token(
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        scopes: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            email,
            roles,
            scopes,
            iat: now,
            exp: now + ACCESS_TOKEN_TTL,
            jti: Uuid::new_v4().to_string(),
            token_type: "access".to_string(),
            token_family_id: None, // Access tokens don't need family tracking
        }
    }

    pub fn new_refresh_token(
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        scopes: Vec<String>,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            email,
            roles,
            scopes,
            iat: now,
            exp: now + REFRESH_TOKEN_TTL,
            jti: Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
            token_family_id: None, // Set externally after family creation
        }
    }

    /// Create a new refresh token with a specific token family ID
    pub fn new_refresh_token_with_family(
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        scopes: Vec<String>,
        family_id: Uuid,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            sub: user_id,
            email,
            roles,
            scopes,
            iat: now,
            exp: now + REFRESH_TOKEN_TTL,
            jti: Uuid::new_v4().to_string(),
            token_type: "refresh".to_string(),
            token_family_id: Some(family_id),
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        self.exp < now
    }

    pub fn validate_type(&self, expected_type: &str) -> Result<()> {
        if self.token_type != expected_type {
            return Err(AuthError::InvalidToken(format!(
                "Expected {} token, got {}",
                expected_type, self.token_type
            )));
        }
        Ok(())
    }
}

/// JWT Manager using RS256 (asymmetric signing)
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    issuer: String,
    audience: String,
}

impl JwtManager {
    /// Create new JWT manager with RS256 keys
    /// In production, load keys from Google Secret Manager
    pub fn new(
        private_key_pem: &[u8],
        public_key_pem: &[u8],
        issuer: String,
        audience: String,
    ) -> Result<Self> {
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem)
            .map_err(|e| AuthError::Config(format!("Invalid private key: {}", e)))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem)
            .map_err(|e| AuthError::Config(format!("Invalid public key: {}", e)))?;

        Ok(Self {
            encoding_key,
            decoding_key,
            issuer,
            audience,
        })
    }

    /// Generate access token
    pub fn create_access_token(
        &self,
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        scopes: Vec<String>,
    ) -> Result<String> {
        let claims = Claims::new_access_token(user_id, email, roles, scopes);
        self.encode_token(&claims)
    }

    /// Generate refresh token
    pub fn create_refresh_token(
        &self,
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        scopes: Vec<String>,
    ) -> Result<String> {
        let claims = Claims::new_refresh_token(user_id, email, roles, scopes);
        self.encode_token(&claims)
    }

    /// Generate refresh token with a specific token family ID
    pub fn create_refresh_token_with_family(
        &self,
        user_id: String,
        email: Option<String>,
        roles: Vec<String>,
        scopes: Vec<String>,
        family_id: Uuid,
    ) -> Result<String> {
        let claims =
            Claims::new_refresh_token_with_family(user_id, email, roles, scopes, family_id);
        self.encode_token(&claims)
    }

    /// Encode JWT with RS256 algorithm
    fn encode_token(&self, claims: &Claims) -> Result<String> {
        let header = Header::new(Algorithm::RS256);

        encode(&header, claims, &self.encoding_key)
            .map_err(|e| AuthError::Internal(format!("Failed to encode JWT: {}", e)))
    }

    /// Verify and decode JWT
    pub fn verify_token(&self, token: &str) -> Result<Claims> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        validation.validate_exp = true;

        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)?;

        // Additional expiration check
        if token_data.claims.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        Ok(token_data.claims)
    }

    /// Verify access token
    pub fn verify_access_token(&self, token: &str) -> Result<Claims> {
        let claims = self.verify_token(token)?;
        claims.validate_type("access")?;
        Ok(claims)
    }

    /// Verify refresh token
    pub fn verify_refresh_token(&self, token: &str) -> Result<Claims> {
        let claims = self.verify_token(token)?;
        claims.validate_type("refresh")?;
        Ok(claims)
    }

    /// Extract token from Authorization header
    pub fn extract_bearer_token(auth_header: &str) -> Result<&str> {
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::InvalidToken("Missing Bearer prefix".to_string()));
        }

        Ok(&auth_header[7..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_keys() -> (Vec<u8>, Vec<u8>) {
        // Generate test RSA keys (in production, use proper key generation)
        let private_key = include_bytes!("../../../tests/fixtures/test_private_key.pem");
        let public_key = include_bytes!("../../../tests/fixtures/test_public_key.pem");
        (private_key.to_vec(), public_key.to_vec())
    }

    #[test]
    #[ignore] // Requires test keys
    fn test_jwt_creation_and_verification() {
        let (private_key, public_key) = create_test_keys();
        let manager = JwtManager::new(
            &private_key,
            &public_key,
            "https://api.mediagateway.io".to_string(),
            "mediagateway-users".to_string(),
        )
        .unwrap();

        let token = manager
            .create_access_token(
                "user123".to_string(),
                Some("user@example.com".to_string()),
                vec!["user".to_string()],
                vec!["read:content".to_string()],
            )
            .unwrap();

        let claims = manager.verify_access_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.token_type, "access");
    }

    #[test]
    fn test_bearer_token_extraction() {
        let header = "Bearer abc123";
        let token = JwtManager::extract_bearer_token(header).unwrap();
        assert_eq!(token, "abc123");

        let invalid_header = "abc123";
        assert!(JwtManager::extract_bearer_token(invalid_header).is_err());
    }
}
