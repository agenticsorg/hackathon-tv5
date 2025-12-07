use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::Rng;
use sha2::{Digest, Sha256};

pub enum TokenType {
    User,
    Service,
    Mcp,
}

impl TokenType {
    fn prefix(&self) -> &str {
        match self {
            TokenType::User => "mg_user_",
            TokenType::Service => "mg_svc_",
            TokenType::Mcp => "mg_mcp_",
        }
    }
}

pub struct TokenManager;

impl TokenManager {
    /// Generate a secure API token with format: mg_{type}_{base62(128bit)}
    pub fn generate_token(token_type: TokenType) -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 16] = rng.gen(); // 128 bits

        let base62_encoded = Self::base62_encode(&random_bytes);
        format!("{}{}", token_type.prefix(), base62_encoded)
    }

    /// Hash token for storage (SHA-256)
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify token hash
    pub fn verify_token(token: &str, hash: &str) -> bool {
        let computed_hash = Self::hash_token(token);
        computed_hash == hash
    }

    /// Base62 encoding (A-Za-z0-9)
    fn base62_encode(data: &[u8]) -> String {
        const BASE62_CHARS: &[u8] =
            b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";

        let mut num = data
            .iter()
            .fold(0u128, |acc, &byte| (acc << 8) | byte as u128);
        let mut result = Vec::new();

        while num > 0 {
            let remainder = (num % 62) as usize;
            result.push(BASE62_CHARS[remainder]);
            num /= 62;
        }

        if result.is_empty() {
            result.push(b'0');
        }

        result.reverse();
        String::from_utf8(result).unwrap()
    }

    /// Generate refresh token (opaque, cryptographically secure)
    pub fn generate_refresh_token() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen(); // 256 bits
        URL_SAFE_NO_PAD.encode(random_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_generation() {
        let token = TokenManager::generate_token(TokenType::User);
        assert!(token.starts_with("mg_user_"));
        assert!(token.len() > 8);
    }

    #[test]
    fn test_token_hashing() {
        let token = "mg_user_test123";
        let hash1 = TokenManager::hash_token(token);
        let hash2 = TokenManager::hash_token(token);

        // Same token should produce same hash
        assert_eq!(hash1, hash2);

        // Verification should work
        assert!(TokenManager::verify_token(token, &hash1));
        assert!(!TokenManager::verify_token("wrong_token", &hash1));
    }

    #[test]
    fn test_refresh_token_generation() {
        let token1 = TokenManager::generate_refresh_token();
        let token2 = TokenManager::generate_refresh_token();

        // Tokens should be unique
        assert_ne!(token1, token2);

        // Should be base64url encoded
        assert!(token1
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_base62_encoding() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let encoded = TokenManager::base62_encode(&data);

        // Should only contain base62 characters
        assert!(encoded.chars().all(|c| c.is_alphanumeric()));
    }
}
