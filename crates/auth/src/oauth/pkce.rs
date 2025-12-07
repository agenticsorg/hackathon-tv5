use crate::error::{AuthError, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CODE_VERIFIER_MIN_LENGTH: usize = 43;
const CODE_VERIFIER_MAX_LENGTH: usize = 128;
const STATE_LENGTH: usize = 32;

/// PKCE (Proof Key for Code Exchange) implementation following RFC 7636
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub state: String,
}

impl PkceChallenge {
    /// Generate a new PKCE challenge with S256 method
    pub fn generate() -> Self {
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::create_s256_challenge(&code_verifier);
        let state = Self::generate_state();

        Self {
            code_verifier,
            code_challenge,
            code_challenge_method: "S256".to_string(),
            state,
        }
    }

    /// Generate cryptographically random code verifier (43-128 chars)
    fn generate_code_verifier() -> String {
        let mut rng = rand::thread_rng();
        let length = rng.gen_range(CODE_VERIFIER_MIN_LENGTH..=CODE_VERIFIER_MAX_LENGTH);

        (0..length)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect()
    }

    /// Create S256 challenge: BASE64URL(SHA256(code_verifier))
    pub fn create_s256_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        let result = hasher.finalize();
        URL_SAFE_NO_PAD.encode(result)
    }

    /// Generate cryptographically random state parameter
    fn generate_state() -> String {
        let mut rng = rand::thread_rng();
        (0..STATE_LENGTH)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect()
    }

    /// Verify that a code verifier matches the stored challenge
    pub fn verify(&self, verifier: &str) -> Result<()> {
        let computed_challenge = Self::create_s256_challenge(verifier);

        if computed_challenge != self.code_challenge {
            tracing::warn!(
                "PKCE verification failed: computed={}, expected={}",
                computed_challenge,
                self.code_challenge
            );
            return Err(AuthError::InvalidPkceVerifier);
        }

        Ok(())
    }
}

/// Authorization code with PKCE binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub code: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub code_challenge: String,
    pub user_id: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub used: bool,
}

impl AuthorizationCode {
    pub fn new(
        client_id: String,
        redirect_uri: String,
        scopes: Vec<String>,
        code_challenge: String,
        user_id: String,
    ) -> Self {
        let code = Self::generate_code();
        let created_at = chrono::Utc::now();
        let expires_at = created_at + chrono::Duration::minutes(10);

        Self {
            code,
            client_id,
            redirect_uri,
            scopes,
            code_challenge,
            user_id,
            created_at,
            expires_at,
            used: false,
        }
    }

    fn generate_code() -> String {
        let mut rng = rand::thread_rng();
        (0..32).map(|_| rng.sample(Alphanumeric) as char).collect()
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    pub fn mark_as_used(&mut self) {
        self.used = true;
    }

    pub fn verify_pkce(&self, verifier: &str) -> Result<()> {
        let computed_challenge = PkceChallenge::create_s256_challenge(verifier);

        if computed_challenge != self.code_challenge {
            return Err(AuthError::InvalidPkceVerifier);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pkce_generation() {
        let pkce = PkceChallenge::generate();

        assert!(pkce.code_verifier.len() >= CODE_VERIFIER_MIN_LENGTH);
        assert!(pkce.code_verifier.len() <= CODE_VERIFIER_MAX_LENGTH);
        assert_eq!(pkce.code_challenge_method, "S256");
        assert_eq!(pkce.state.len(), STATE_LENGTH);
    }

    #[test]
    fn test_pkce_verification() {
        let pkce = PkceChallenge::generate();
        let verifier = pkce.code_verifier.clone();

        assert!(pkce.verify(&verifier).is_ok());
        assert!(pkce.verify("invalid_verifier").is_err());
    }

    #[test]
    fn test_auth_code_expiration() {
        let code = AuthorizationCode::new(
            "client123".to_string(),
            "https://example.com/callback".to_string(),
            vec!["read:content".to_string()],
            "challenge".to_string(),
            "user123".to_string(),
        );

        assert!(!code.is_expired());
    }
}
