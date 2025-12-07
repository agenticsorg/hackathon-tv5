//! JWT token generation and validation tests

use crate::jwt::*;

#[test]
fn test_claims_new_access_token() {
    let user_id = "user123".to_string();
    let email = Some("user@example.com".to_string());
    let roles = vec!["user".to_string()];
    let scopes = vec!["read:content".to_string()];

    let claims = Claims::new_access_token(
        user_id.clone(),
        email.clone(),
        roles.clone(),
        scopes.clone(),
    );

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.email, email);
    assert_eq!(claims.token_type, "access");
    assert!(!claims.jti.is_empty());
}

#[test]
fn test_claims_new_refresh_token() {
    let user_id = "user123".to_string();
    let claims = Claims::new_refresh_token(
        user_id.clone(),
        None,
        vec!["user".to_string()],
        vec!["read:content".to_string()],
    );

    assert_eq!(claims.sub, user_id);
    assert_eq!(claims.token_type, "refresh");
}

#[test]
fn test_claims_expiration_check() {
    let mut claims = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    // Not expired initially
    assert!(!claims.is_expired());

    // Set expiration to past
    claims.exp = chrono::Utc::now().timestamp() - 3600;
    assert!(claims.is_expired());
}

#[test]
fn test_claims_validate_type_access() {
    let claims = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    assert!(claims.validate_type("access").is_ok());
    assert!(claims.validate_type("refresh").is_err());
}

#[test]
fn test_claims_validate_type_refresh() {
    let claims = Claims::new_refresh_token("user123".to_string(), None, vec![], vec![]);

    assert!(claims.validate_type("refresh").is_ok());
    assert!(claims.validate_type("access").is_err());
}

#[test]
fn test_access_token_ttl() {
    let claims1 = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    let expected_exp = claims1.iat + 3600; // 1 hour
    assert_eq!(claims1.exp, expected_exp);
}

#[test]
fn test_refresh_token_ttl() {
    let claims = Claims::new_refresh_token("user123".to_string(), None, vec![], vec![]);

    let expected_exp = claims.iat + (7 * 24 * 3600); // 7 days
    assert_eq!(claims.exp, expected_exp);
}

#[test]
fn test_jwt_extract_bearer_token_success() {
    let header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let token = JwtManager::extract_bearer_token(header).unwrap();

    assert_eq!(token, "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");
}

#[test]
fn test_jwt_extract_bearer_token_missing_prefix() {
    let header = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
    let result = JwtManager::extract_bearer_token(header);

    assert!(result.is_err());
}

#[test]
fn test_jwt_extract_bearer_token_wrong_scheme() {
    let header = "Basic dXNlcjpwYXNz";
    let result = JwtManager::extract_bearer_token(header);

    assert!(result.is_err());
}

#[test]
fn test_claims_jti_is_unique() {
    let claims1 = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    let claims2 = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    assert_ne!(claims1.jti, claims2.jti);
}

#[test]
fn test_claims_includes_roles() {
    let roles = vec!["admin".to_string(), "user".to_string()];
    let claims = Claims::new_access_token("user123".to_string(), None, roles.clone(), vec![]);

    assert_eq!(claims.roles, roles);
    assert_eq!(claims.roles.len(), 2);
}

#[test]
fn test_claims_includes_scopes() {
    let scopes = vec!["read:content".to_string(), "write:content".to_string()];
    let claims = Claims::new_access_token("user123".to_string(), None, vec![], scopes.clone());

    assert_eq!(claims.scopes, scopes);
    assert_eq!(claims.scopes.len(), 2);
}

#[test]
fn test_claims_with_email() {
    let email = "test@example.com".to_string();
    let claims =
        Claims::new_access_token("user123".to_string(), Some(email.clone()), vec![], vec![]);

    assert_eq!(claims.email, Some(email));
}

#[test]
fn test_claims_without_email() {
    let claims = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    assert!(claims.email.is_none());
}

#[test]
fn test_claims_iat_is_current_time() {
    let now = chrono::Utc::now().timestamp();
    let claims = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    // iat should be within 1 second of current time
    assert!((claims.iat - now).abs() <= 1);
}

#[test]
fn test_token_type_values() {
    let access = Claims::new_access_token("user123".to_string(), None, vec![], vec![]);

    let refresh = Claims::new_refresh_token("user123".to_string(), None, vec![], vec![]);

    assert_eq!(access.token_type, "access");
    assert_eq!(refresh.token_type, "refresh");
}
