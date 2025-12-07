use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub avatar_url: Option<String>,
    pub preferences: serde_json::Value,
    pub email_verified: bool,
    pub created_at: DateTime<Utc>,
    pub oauth_providers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateProfileRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub preferences: Option<serde_json::Value>,
}

impl UpdateProfileRequest {
    pub fn validate(&self) -> crate::Result<()> {
        if let Some(ref name) = self.display_name {
            if name.is_empty() || name.len() > 100 {
                return Err(crate::AuthError::Internal(
                    "Display name must be between 1 and 100 characters".to_string(),
                ));
            }
        }

        if let Some(ref url) = self.avatar_url {
            if url.len() > 500 {
                return Err(crate::AuthError::Internal(
                    "Avatar URL must not exceed 500 characters".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AvatarUploadResponse {
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub user_id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}
