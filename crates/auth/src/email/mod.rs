pub mod providers;
pub mod service;
pub mod templates;

pub use providers::{ConsoleProvider, SendGridProvider};
pub use service::{EmailError, EmailManager, EmailService, VerificationToken};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub provider: EmailProviderConfig,
    pub from_email: String,
    pub from_name: String,
    pub base_url: String,
    pub verification_ttl_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EmailProviderConfig {
    SendGrid {
        api_key: String,
    },
    AwsSes {
        region: String,
        access_key_id: String,
        secret_access_key: String,
    },
    Console,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            provider: EmailProviderConfig::Console,
            from_email: "noreply@mediagateway.local".to_string(),
            from_name: "Media Gateway".to_string(),
            base_url: "http://localhost:8080".to_string(),
            verification_ttl_hours: 24,
        }
    }
}
