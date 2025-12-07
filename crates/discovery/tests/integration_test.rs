//! Integration tests for discovery service JWT auth

use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    iat: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_token_creation() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret";

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 3600, // 1 hour from now
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        assert!(!token.is_empty());
    }

    #[test]
    fn test_jwt_token_validation() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret";

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 3600,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // Decode and validate
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .unwrap();

        assert_eq!(decoded.claims.sub, user_id.to_string());
    }

    #[test]
    fn test_jwt_expired_token() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret";

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now - 3600, // 1 hour ago (expired)
            iat: now - 7200,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // Try to decode expired token
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_jwt_invalid_secret() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret";
        let wrong_secret = "wrong-secret";

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 3600,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // Try to decode with wrong secret
        let validation = Validation::new(Algorithm::HS256);

        let result = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(wrong_secret.as_bytes()),
            &validation,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_user_id_extraction_from_token() {
        let user_id = Uuid::new_v4();
        let secret = "test-secret";

        let now = chrono::Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            exp: now + 3600,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        // Decode and extract user_id
        let validation = Validation::new(Algorithm::HS256);
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &validation,
        )
        .unwrap();

        let extracted_user_id = Uuid::parse_str(&decoded.claims.sub).unwrap();

        assert_eq!(extracted_user_id, user_id);
    }
}
