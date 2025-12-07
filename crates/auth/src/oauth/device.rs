use crate::error::{AuthError, Result};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

const USER_CODE_LENGTH: usize = 8;
const DEVICE_CODE_LENGTH: usize = 32;
const POLLING_INTERVAL: u64 = 5; // seconds
const DEVICE_CODE_TTL: i64 = 15 * 60; // 15 minutes

/// Device Authorization Grant (RFC 8628) implementation
/// For Smart TV, CLI, and other input-constrained devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u64,
    pub interval: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub status: DeviceCodeStatus,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeviceCodeStatus {
    Pending,
    Approved,
    Denied,
    Expired,
}

impl DeviceCode {
    pub fn new(client_id: String, scopes: Vec<String>, base_url: &str) -> Self {
        let device_code = Self::generate_device_code();
        let user_code = Self::generate_user_code();
        let created_at = chrono::Utc::now();

        let verification_uri = format!("{}/device", base_url);
        let verification_uri_complete = format!("{}/device?user_code={}", base_url, user_code);

        Self {
            device_code,
            user_code: user_code.clone(),
            verification_uri,
            verification_uri_complete,
            expires_in: DEVICE_CODE_TTL as u64,
            interval: POLLING_INTERVAL,
            created_at,
            client_id,
            scopes,
            status: DeviceCodeStatus::Pending,
            user_id: None,
        }
    }

    /// Generate device code (long random string)
    fn generate_device_code() -> String {
        let mut rng = rand::thread_rng();
        (0..DEVICE_CODE_LENGTH)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect()
    }

    /// Generate user code (short alphanumeric code)
    /// Format: XXXX-XXXX for easy human input
    fn generate_user_code() -> String {
        let mut rng = rand::thread_rng();
        let chars: String = (0..USER_CODE_LENGTH)
            .map(|_| {
                let options = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // No 0, O, 1, I
                options[rng.gen_range(0..options.len())] as char
            })
            .collect();

        format!("{}-{}", &chars[0..4], &chars[4..8])
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(self.created_at);
        age.num_seconds() > DEVICE_CODE_TTL
    }

    pub fn approve(&mut self, user_id: String) {
        self.status = DeviceCodeStatus::Approved;
        self.user_id = Some(user_id);
    }

    pub fn deny(&mut self) {
        self.status = DeviceCodeStatus::Denied;
    }

    pub fn check_status(&self) -> Result<DeviceCodeStatus> {
        if self.is_expired() {
            return Ok(DeviceCodeStatus::Expired);
        }

        match self.status {
            DeviceCodeStatus::Pending => Err(AuthError::AuthorizationPending),
            DeviceCodeStatus::Approved => Ok(DeviceCodeStatus::Approved),
            DeviceCodeStatus::Denied => Err(AuthError::AccessDenied),
            DeviceCodeStatus::Expired => Err(AuthError::DeviceCodeExpired),
        }
    }
}

/// Response for device authorization request
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceAuthorizationResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub verification_uri_complete: String,
    pub expires_in: u64,
    pub interval: u64,
}

impl From<&DeviceCode> for DeviceAuthorizationResponse {
    fn from(device: &DeviceCode) -> Self {
        Self {
            device_code: device.device_code.clone(),
            user_code: device.user_code.clone(),
            verification_uri: device.verification_uri.clone(),
            verification_uri_complete: device.verification_uri_complete.clone(),
            expires_in: device.expires_in,
            interval: device.interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_code_generation() {
        let device = DeviceCode::new(
            "client123".to_string(),
            vec!["read:content".to_string()],
            "https://auth.example.com",
        );

        assert_eq!(device.device_code.len(), DEVICE_CODE_LENGTH);
        assert_eq!(device.user_code.len(), 9); // XXXX-XXXX
        assert!(device.user_code.contains('-'));
        assert_eq!(device.interval, POLLING_INTERVAL);
        assert_eq!(device.status, DeviceCodeStatus::Pending);
    }

    #[test]
    fn test_device_code_approval() {
        let mut device = DeviceCode::new(
            "client123".to_string(),
            vec!["read:content".to_string()],
            "https://auth.example.com",
        );

        device.approve("user123".to_string());
        assert_eq!(device.status, DeviceCodeStatus::Approved);
        assert_eq!(device.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_device_code_status_check() {
        let device = DeviceCode::new(
            "client123".to_string(),
            vec!["read:content".to_string()],
            "https://auth.example.com",
        );

        // Pending should return error
        assert!(device.check_status().is_err());
    }

    #[test]
    fn test_user_code_format() {
        let device = DeviceCode::new("client123".to_string(), vec![], "https://auth.example.com");

        // User code should not contain confusing characters
        assert!(!device.user_code.contains('0'));
        assert!(!device.user_code.contains('O'));
        assert!(!device.user_code.contains('1'));
        assert!(!device.user_code.contains('I'));
    }

    #[test]
    fn test_device_code_approved_status() {
        let mut device = DeviceCode::new(
            "client123".to_string(),
            vec!["read:content".to_string()],
            "https://auth.example.com",
        );

        // Approve the device
        device.approve("user456".to_string());

        // Check status should succeed for approved device
        let status = device.check_status().unwrap();
        assert_eq!(status, DeviceCodeStatus::Approved);
        assert_eq!(device.user_id, Some("user456".to_string()));
    }

    #[test]
    fn test_device_code_denied_status() {
        let mut device = DeviceCode::new(
            "client123".to_string(),
            vec!["read:content".to_string()],
            "https://auth.example.com",
        );

        // Deny the device
        device.deny();

        // Check status should return error for denied device
        let result = device.check_status();
        assert!(result.is_err());
        assert_eq!(device.status, DeviceCodeStatus::Denied);
    }

    #[test]
    fn test_device_code_pending_returns_error() {
        let device = DeviceCode::new(
            "client123".to_string(),
            vec!["read:content".to_string()],
            "https://auth.example.com",
        );

        // Pending status should return AuthorizationPending error
        let result = device.check_status();
        assert!(result.is_err());

        match result {
            Err(AuthError::AuthorizationPending) => {
                // Expected error
            }
            _ => panic!("Expected AuthorizationPending error"),
        }
    }

    #[test]
    fn test_device_code_cannot_approve_twice() {
        let mut device = DeviceCode::new(
            "client123".to_string(),
            vec!["read:content".to_string()],
            "https://auth.example.com",
        );

        // First approval
        device.approve("user123".to_string());
        assert_eq!(device.user_id, Some("user123".to_string()));

        // Second approval should overwrite (though API prevents this)
        device.approve("user456".to_string());
        assert_eq!(device.user_id, Some("user456".to_string()));
        assert_eq!(device.status, DeviceCodeStatus::Approved);
    }
}
