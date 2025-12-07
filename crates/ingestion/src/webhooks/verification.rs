//! HMAC signature verification for webhooks

use crate::webhooks::{WebhookError, WebhookResult};
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Verify HMAC-SHA256 signature
///
/// # Arguments
/// * `payload` - Raw webhook payload bytes
/// * `signature` - Signature from webhook header (format: "sha256=...")
/// * `secret` - Platform secret key
///
/// # Returns
/// True if signature is valid, false otherwise
pub fn verify_hmac_signature(payload: &[u8], signature: &str, secret: &str) -> WebhookResult<bool> {
    // Parse signature format: "sha256=hexstring"
    let signature_hex = signature
        .strip_prefix("sha256=")
        .ok_or_else(|| WebhookError::InvalidSignature("Missing sha256= prefix".to_string()))?;

    // Decode hex signature
    let expected_signature = hex::decode(signature_hex)
        .map_err(|e| WebhookError::InvalidSignature(format!("Invalid hex: {}", e)))?;

    // Compute HMAC
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| WebhookError::InvalidSignature(format!("Invalid secret: {}", e)))?;

    mac.update(payload);

    // Verify signature
    mac.verify_slice(&expected_signature)
        .map(|_| true)
        .or(Ok(false))
}

/// Generate HMAC-SHA256 signature
///
/// # Arguments
/// * `payload` - Raw payload bytes
/// * `secret` - Secret key
///
/// # Returns
/// Signature in format "sha256=hexstring"
pub fn generate_hmac_signature(payload: &[u8], secret: &str) -> WebhookResult<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .map_err(|e| WebhookError::InvalidSignature(format!("Invalid secret: {}", e)))?;

    mac.update(payload);
    let result = mac.finalize();
    let signature_bytes = result.into_bytes();

    Ok(format!("sha256={}", hex::encode(signature_bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_verify_signature() {
        let payload = b"test webhook payload";
        let secret = "test-secret-key";

        let signature = generate_hmac_signature(payload, secret).unwrap();
        assert!(signature.starts_with("sha256="));

        let is_valid = verify_hmac_signature(payload, &signature, secret).unwrap();
        assert!(is_valid);
    }

    #[test]
    fn test_verify_invalid_signature() {
        let payload = b"test webhook payload";
        let secret = "test-secret-key";

        let signature = "sha256=0000000000000000000000000000000000000000000000000000000000000000";
        let is_valid = verify_hmac_signature(payload, signature, secret).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_verify_wrong_secret() {
        let payload = b"test webhook payload";
        let secret = "test-secret-key";
        let wrong_secret = "wrong-secret-key";

        let signature = generate_hmac_signature(payload, secret).unwrap();
        let is_valid = verify_hmac_signature(payload, &signature, wrong_secret).unwrap();
        assert!(!is_valid);
    }

    #[test]
    fn test_verify_missing_prefix() {
        let payload = b"test webhook payload";
        let secret = "test-secret-key";

        let result = verify_hmac_signature(payload, "abcd1234", secret);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WebhookError::InvalidSignature(_)
        ));
    }

    #[test]
    fn test_verify_invalid_hex() {
        let payload = b"test webhook payload";
        let secret = "test-secret-key";

        let result = verify_hmac_signature(payload, "sha256=invalid_hex", secret);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WebhookError::InvalidSignature(_)
        ));
    }

    #[test]
    fn test_signature_deterministic() {
        let payload = b"test webhook payload";
        let secret = "test-secret-key";

        let sig1 = generate_hmac_signature(payload, secret).unwrap();
        let sig2 = generate_hmac_signature(payload, secret).unwrap();

        assert_eq!(sig1, sig2);
    }
}
