use crate::error::{AuthError, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use totp_rs::{Algorithm, Secret, TOTP};

const TOTP_STEP: u64 = 30;
const TOTP_DIGITS: usize = 6;
const TOTP_SKEW: u8 = 1; // Allow ±1 time window (±30 seconds)

#[derive(Debug, Clone)]
pub struct TotpManager {
    encryption_key: [u8; 32],
}

impl TotpManager {
    pub fn new(encryption_key: &[u8; 32]) -> Self {
        Self {
            encryption_key: *encryption_key,
        }
    }

    pub fn generate_secret(&self, user_id: &str) -> Result<(String, String)> {
        // Generate 20 random bytes for TOTP secret (160 bits, standard size)
        let bytes: [u8; 20] = rand::thread_rng().gen();
        let secret = Secret::Raw(bytes.to_vec());
        let secret_base32 = secret.to_encoded().to_string();

        let totp = TOTP::new(
            Algorithm::SHA1,
            TOTP_DIGITS,
            TOTP_SKEW,
            TOTP_STEP,
            secret.to_bytes().unwrap(),
            Some("MediaGateway".to_string()),
            user_id.to_string(),
        )
        .map_err(|e| AuthError::Internal(format!("TOTP generation failed: {}", e)))?;

        let qr_code = totp
            .get_qr_base64()
            .map_err(|e| AuthError::Internal(format!("QR code generation failed: {}", e)))?;

        Ok((secret_base32, qr_code))
    }

    pub fn verify_code(&self, secret: &str, code: &str) -> Result<bool> {
        let secret_bytes = Secret::Encoded(secret.to_string())
            .to_bytes()
            .map_err(|e| AuthError::Internal(format!("Secret decoding failed: {}", e)))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            TOTP_DIGITS,
            TOTP_SKEW,
            TOTP_STEP,
            secret_bytes,
            None,
            String::new(),
        )
        .map_err(|e| AuthError::Internal(format!("TOTP verification failed: {}", e)))?;

        Ok(totp.check_current(code).unwrap_or(false))
    }

    pub fn encrypt_secret(&self, secret: &str) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce_bytes: [u8; 12] = rand::thread_rng().gen();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, secret.as_bytes())
            .map_err(|e| AuthError::Encryption(format!("Encryption failed: {}", e)))?;

        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt_secret(&self, encrypted_data: &[u8]) -> Result<String> {
        if encrypted_data.len() < 12 {
            return Err(AuthError::Encryption("Invalid encrypted data".to_string()));
        }

        let cipher = Aes256Gcm::new(&self.encryption_key.into());
        let nonce = Nonce::from_slice(&encrypted_data[..12]);
        let ciphertext = &encrypted_data[12..];

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AuthError::Encryption(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AuthError::Encryption(format!("UTF-8 conversion failed: {}", e)))
    }

    #[cfg(test)]
    pub fn generate_current_code(&self, secret: &str) -> Result<String> {
        let secret_bytes = Secret::Encoded(secret.to_string())
            .to_bytes()
            .map_err(|e| AuthError::Internal(format!("Secret decoding failed: {}", e)))?;

        let totp = TOTP::new(
            Algorithm::SHA1,
            TOTP_DIGITS,
            TOTP_SKEW,
            TOTP_STEP,
            secret_bytes,
            None,
            String::new(),
        )
        .map_err(|e| AuthError::Internal(format!("TOTP generation failed: {}", e)))?;

        Ok(totp.generate_current().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_encryption_key() -> [u8; 32] {
        [0u8; 32]
    }

    #[test]
    fn test_generate_secret() {
        let manager = TotpManager::new(&test_encryption_key());
        let (secret, qr_code) = manager.generate_secret("test@example.com").unwrap();

        assert!(!secret.is_empty());
        assert!(qr_code.starts_with("data:image/png;base64,"));
    }

    #[test]
    fn test_verify_code_success() {
        let manager = TotpManager::new(&test_encryption_key());
        let (secret, _) = manager.generate_secret("test@example.com").unwrap();

        let code = manager.generate_current_code(&secret).unwrap();
        assert!(manager.verify_code(&secret, &code).unwrap());
    }

    #[test]
    fn test_verify_code_failure() {
        let manager = TotpManager::new(&test_encryption_key());
        let (secret, _) = manager.generate_secret("test@example.com").unwrap();

        assert!(!manager.verify_code(&secret, "000000").unwrap());
    }

    #[test]
    fn test_encrypt_decrypt_secret() {
        let manager = TotpManager::new(&test_encryption_key());
        let original_secret = "JBSWY3DPEHPK3PXP";

        let encrypted = manager.encrypt_secret(original_secret).unwrap();
        let decrypted = manager.decrypt_secret(&encrypted).unwrap();

        assert_eq!(original_secret, decrypted);
    }

    #[test]
    fn test_decrypt_invalid_data() {
        let manager = TotpManager::new(&test_encryption_key());
        let invalid_data = vec![1, 2, 3];

        assert!(manager.decrypt_secret(&invalid_data).is_err());
    }

    #[test]
    fn test_totp_constants() {
        assert_eq!(TOTP_STEP, 30);
        assert_eq!(TOTP_DIGITS, 6);
        assert_eq!(TOTP_SKEW, 1);
    }
}
