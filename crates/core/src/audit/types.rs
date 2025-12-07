use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub action: AuditAction,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl AuditEvent {
    pub fn new(action: AuditAction, resource_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id: None,
            action,
            resource_type,
            resource_id: None,
            details: serde_json::json!({}),
            ip_address: None,
            user_agent: None,
        }
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_resource_id(mut self, resource_id: String) -> Self {
        self.resource_id = Some(resource_id);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = details;
        self
    }

    pub fn with_ip_address(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }

    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditAction {
    AuthLogin,
    AuthLogout,
    AuthFailed,
    AuthRegister,
    AuthPasswordReset,
    EmailVerified,
    UserCreated,
    UserUpdated,
    UserDeleted,
    AdminAction,
    AdminImpersonate,
    ApiKeyCreated,
    ApiKeyRevoked,
    ContentCreated,
    ContentUpdated,
    ContentDeleted,
    Create,
    Update,
    Delete,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::AuthLogin => "AUTH_LOGIN",
            AuditAction::AuthLogout => "AUTH_LOGOUT",
            AuditAction::AuthFailed => "AUTH_FAILED",
            AuditAction::AuthRegister => "AUTH_REGISTER",
            AuditAction::AuthPasswordReset => "AUTH_PASSWORD_RESET",
            AuditAction::EmailVerified => "EMAIL_VERIFIED",
            AuditAction::UserCreated => "USER_CREATED",
            AuditAction::UserUpdated => "USER_UPDATED",
            AuditAction::UserDeleted => "USER_DELETED",
            AuditAction::AdminAction => "ADMIN_ACTION",
            AuditAction::AdminImpersonate => "ADMIN_IMPERSONATE",
            AuditAction::ApiKeyCreated => "API_KEY_CREATED",
            AuditAction::ApiKeyRevoked => "API_KEY_REVOKED",
            AuditAction::ContentCreated => "CONTENT_CREATED",
            AuditAction::ContentUpdated => "CONTENT_UPDATED",
            AuditAction::ContentDeleted => "CONTENT_DELETED",
            AuditAction::Create => "CREATE",
            AuditAction::Update => "UPDATE",
            AuditAction::Delete => "DELETE",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "AUTH_LOGIN" => Some(AuditAction::AuthLogin),
            "AUTH_LOGOUT" => Some(AuditAction::AuthLogout),
            "AUTH_FAILED" => Some(AuditAction::AuthFailed),
            "AUTH_REGISTER" => Some(AuditAction::AuthRegister),
            "AUTH_PASSWORD_RESET" => Some(AuditAction::AuthPasswordReset),
            "EMAIL_VERIFIED" => Some(AuditAction::EmailVerified),
            "USER_CREATED" => Some(AuditAction::UserCreated),
            "USER_UPDATED" => Some(AuditAction::UserUpdated),
            "USER_DELETED" => Some(AuditAction::UserDeleted),
            "ADMIN_ACTION" => Some(AuditAction::AdminAction),
            "ADMIN_IMPERSONATE" => Some(AuditAction::AdminImpersonate),
            "API_KEY_CREATED" => Some(AuditAction::ApiKeyCreated),
            "API_KEY_REVOKED" => Some(AuditAction::ApiKeyRevoked),
            "CONTENT_CREATED" => Some(AuditAction::ContentCreated),
            "CONTENT_UPDATED" => Some(AuditAction::ContentUpdated),
            "CONTENT_DELETED" => Some(AuditAction::ContentDeleted),
            "CREATE" => Some(AuditAction::Create),
            "UPDATE" => Some(AuditAction::Update),
            "DELETE" => Some(AuditAction::Delete),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFilter {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub user_id: Option<Uuid>,
    pub action: Option<AuditAction>,
    pub resource_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for AuditFilter {
    fn default() -> Self {
        Self {
            start_date: None,
            end_date: None,
            user_id: None,
            action: None,
            resource_type: None,
            limit: Some(100),
            offset: Some(0),
        }
    }
}

impl AuditFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self.end_date = Some(end);
        self
    }

    pub fn with_user_id(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_action(mut self, action: AuditAction) -> Self {
        self.action = Some(action);
        self
    }

    pub fn with_resource_type(mut self, resource_type: String) -> Self {
        self.resource_type = Some(resource_type);
        self
    }

    pub fn with_limit(mut self, limit: i64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: i64) -> Self {
        self.offset = Some(offset);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new(AuditAction::AuthLogin, "user".to_string());

        assert_eq!(event.action, AuditAction::AuthLogin);
        assert_eq!(event.resource_type, "user");
        assert!(event.user_id.is_none());
        assert!(event.resource_id.is_none());
        assert_eq!(event.details, serde_json::json!({}));
    }

    #[test]
    fn test_audit_event_builder() {
        let user_id = Uuid::new_v4();
        let event = AuditEvent::new(AuditAction::UserCreated, "user".to_string())
            .with_user_id(user_id)
            .with_resource_id("user-123".to_string())
            .with_details(serde_json::json!({"email": "test@example.com"}))
            .with_ip_address("192.168.1.1".to_string())
            .with_user_agent("Mozilla/5.0".to_string());

        assert_eq!(event.user_id, Some(user_id));
        assert_eq!(event.resource_id, Some("user-123".to_string()));
        assert_eq!(event.details["email"], "test@example.com");
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(event.user_agent, Some("Mozilla/5.0".to_string()));
    }

    #[test]
    fn test_audit_action_as_str() {
        assert_eq!(AuditAction::AuthLogin.as_str(), "AUTH_LOGIN");
        assert_eq!(AuditAction::UserCreated.as_str(), "USER_CREATED");
        assert_eq!(AuditAction::ContentDeleted.as_str(), "CONTENT_DELETED");
    }

    #[test]
    fn test_audit_action_from_str() {
        assert_eq!(
            AuditAction::from_str("AUTH_LOGIN"),
            Some(AuditAction::AuthLogin)
        );
        assert_eq!(
            AuditAction::from_str("USER_CREATED"),
            Some(AuditAction::UserCreated)
        );
        assert_eq!(AuditAction::from_str("INVALID"), None);
    }

    #[test]
    fn test_audit_action_roundtrip() {
        let actions = vec![
            AuditAction::AuthLogin,
            AuditAction::UserCreated,
            AuditAction::AdminAction,
            AuditAction::ContentDeleted,
        ];

        for action in actions {
            let str_repr = action.as_str();
            let parsed = AuditAction::from_str(str_repr);
            assert_eq!(parsed, Some(action));
        }
    }

    #[test]
    fn test_audit_filter_default() {
        let filter = AuditFilter::default();

        assert!(filter.start_date.is_none());
        assert!(filter.end_date.is_none());
        assert!(filter.user_id.is_none());
        assert!(filter.action.is_none());
        assert!(filter.resource_type.is_none());
        assert_eq!(filter.limit, Some(100));
        assert_eq!(filter.offset, Some(0));
    }

    #[test]
    fn test_audit_filter_builder() {
        let user_id = Uuid::new_v4();
        let start = Utc::now();
        let end = Utc::now();

        let filter = AuditFilter::new()
            .with_date_range(start, end)
            .with_user_id(user_id)
            .with_action(AuditAction::AuthLogin)
            .with_resource_type("user".to_string())
            .with_limit(50)
            .with_offset(10);

        assert_eq!(filter.start_date, Some(start));
        assert_eq!(filter.end_date, Some(end));
        assert_eq!(filter.user_id, Some(user_id));
        assert_eq!(filter.action, Some(AuditAction::AuthLogin));
        assert_eq!(filter.resource_type, Some("user".to_string()));
        assert_eq!(filter.limit, Some(50));
        assert_eq!(filter.offset, Some(10));
    }

    #[test]
    fn test_audit_event_serialization() {
        let event = AuditEvent::new(AuditAction::AuthLogin, "user".to_string())
            .with_user_id(Uuid::new_v4())
            .with_details(serde_json::json!({"success": true}));

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: AuditEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event.id, deserialized.id);
        assert_eq!(event.action, deserialized.action);
        assert_eq!(event.resource_type, deserialized.resource_type);
    }
}
