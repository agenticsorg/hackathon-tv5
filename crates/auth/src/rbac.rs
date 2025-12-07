use crate::error::{AuthError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Role-Based Access Control (RBAC) implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Anonymous,
    FreeUser,
    PremiumUser,
    Admin,
    ServiceAccount,
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::Anonymous => "anonymous",
            Role::FreeUser => "free_user",
            Role::PremiumUser => "premium_user",
            Role::Admin => "admin",
            Role::ServiceAccount => "service_account",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "anonymous" => Some(Role::Anonymous),
            "free_user" => Some(Role::FreeUser),
            "premium_user" => Some(Role::PremiumUser),
            "admin" => Some(Role::Admin),
            "service_account" => Some(Role::ServiceAccount),
            _ => None,
        }
    }
}

/// Permission format: resource:action:scope
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
    pub scope: String,
}

impl Permission {
    pub fn new(
        resource: impl Into<String>,
        action: impl Into<String>,
        scope: impl Into<String>,
    ) -> Self {
        Self {
            resource: resource.into(),
            action: action.into(),
            scope: scope.into(),
        }
    }

    pub fn from_string(perm: &str) -> Result<Self> {
        let parts: Vec<&str> = perm.split(':').collect();
        if parts.len() != 3 {
            return Err(AuthError::Internal(format!(
                "Invalid permission format: {}",
                perm
            )));
        }

        Ok(Self {
            resource: parts[0].to_string(),
            action: parts[1].to_string(),
            scope: parts[2].to_string(),
        })
    }

    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.resource, self.action, self.scope)
    }

    pub fn matches(&self, required: &Permission) -> bool {
        // Wildcard matching
        (self.resource == "*" || self.resource == required.resource)
            && (self.action == "*" || self.action == required.action)
            && (self.scope == "*" || self.scope == required.scope)
    }
}

pub struct RbacManager {
    role_permissions: HashMap<Role, Vec<Permission>>,
}

impl RbacManager {
    pub fn new() -> Self {
        let mut role_permissions = HashMap::new();

        // Anonymous - minimal access
        role_permissions.insert(
            Role::Anonymous,
            vec![
                Permission::new("content", "read", "public"),
                Permission::new("content", "search", "limited"),
            ],
        );

        // Free User
        role_permissions.insert(
            Role::FreeUser,
            vec![
                Permission::new("content", "read", "*"),
                Permission::new("content", "search", "basic"),
                Permission::new("recommendation", "get", "limited"),
                Permission::new("watchlist", "read", "self"),
                Permission::new("watchlist", "write", "self"),
                Permission::new("preferences", "read", "self"),
                Permission::new("preferences", "write", "self"),
                Permission::new("device", "register", "5"),
            ],
        );

        // Premium User - inherits free user + additional permissions
        let mut premium_perms = role_permissions.get(&Role::FreeUser).unwrap().clone();
        premium_perms.extend(vec![
            Permission::new("content", "search", "advanced"),
            Permission::new("recommendation", "get", "unlimited"),
            Permission::new("device", "register", "unlimited"),
            Permission::new("export", "data", "self"),
        ]);
        role_permissions.insert(Role::PremiumUser, premium_perms);

        // Admin - full access
        role_permissions.insert(Role::Admin, vec![Permission::new("*", "*", "*")]);

        // Service Account - API access
        role_permissions.insert(
            Role::ServiceAccount,
            vec![
                Permission::new("content", "read", "*"),
                Permission::new("content", "write", "metadata"),
                Permission::new("recommendation", "compute", "*"),
            ],
        );

        Self { role_permissions }
    }

    /// Check if roles have a specific permission
    pub fn has_permission(&self, roles: &[Role], required: &Permission) -> bool {
        for role in roles {
            if let Some(permissions) = self.role_permissions.get(role) {
                for perm in permissions {
                    if perm.matches(required) {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Check permission and return error if denied
    pub fn require_permission(&self, roles: &[Role], required: &Permission) -> Result<()> {
        if self.has_permission(roles, required) {
            Ok(())
        } else {
            tracing::warn!(
                "Permission denied: {:?} does not have {}",
                roles,
                required.to_string()
            );
            Err(AuthError::InsufficientPermissions)
        }
    }

    /// Get all permissions for a role
    pub fn get_role_permissions(&self, role: &Role) -> Vec<Permission> {
        self.role_permissions.get(role).cloned().unwrap_or_default()
    }
}

impl Default for RbacManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_parsing() {
        let perm = Permission::from_string("content:read:public").unwrap();
        assert_eq!(perm.resource, "content");
        assert_eq!(perm.action, "read");
        assert_eq!(perm.scope, "public");
    }

    #[test]
    fn test_permission_matching() {
        let wildcard = Permission::new("*", "*", "*");
        let specific = Permission::new("content", "read", "public");

        assert!(wildcard.matches(&specific));

        let resource_wildcard = Permission::new("content", "*", "*");
        assert!(resource_wildcard.matches(&specific));

        let no_match = Permission::new("user", "write", "self");
        assert!(!no_match.matches(&specific));
    }

    #[test]
    fn test_rbac_free_user_permissions() {
        let rbac = RbacManager::new();

        let free_user_roles = vec![Role::FreeUser];

        // Should have permission
        assert!(rbac.has_permission(
            &free_user_roles,
            &Permission::new("content", "read", "public")
        ));

        assert!(rbac.has_permission(
            &free_user_roles,
            &Permission::new("watchlist", "write", "self")
        ));

        // Should not have permission
        assert!(!rbac.has_permission(
            &free_user_roles,
            &Permission::new("content", "search", "advanced")
        ));
    }

    #[test]
    fn test_rbac_admin_permissions() {
        let rbac = RbacManager::new();

        let admin_roles = vec![Role::Admin];

        // Admin should have all permissions
        assert!(rbac.has_permission(&admin_roles, &Permission::new("content", "delete", "all")));

        assert!(rbac.has_permission(&admin_roles, &Permission::new("user", "suspend", "any")));
    }

    #[test]
    fn test_rbac_require_permission() {
        let rbac = RbacManager::new();

        let free_user_roles = vec![Role::FreeUser];

        // Should succeed
        assert!(rbac
            .require_permission(
                &free_user_roles,
                &Permission::new("content", "read", "public")
            )
            .is_ok());

        // Should fail
        assert!(rbac
            .require_permission(&free_user_roles, &Permission::new("user", "delete", "any"))
            .is_err());
    }
}
