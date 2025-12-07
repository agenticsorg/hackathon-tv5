//! PKCE (Proof Key for Code Exchange) tests

use crate::oauth::pkce::*;

#[test]
fn test_pkce_challenge_generation() {
    let pkce = PkceChallenge::generate();

    assert!(!pkce.code_verifier.is_empty());
    assert!(!pkce.code_challenge.is_empty());
    assert_eq!(pkce.code_challenge_method, "S256");
    assert!(!pkce.state.is_empty());
}

#[test]
fn test_code_verifier_length_within_bounds() {
    let pkce = PkceChallenge::generate();

    assert!(pkce.code_verifier.len() >= 43);
    assert!(pkce.code_verifier.len() <= 128);
}

#[test]
fn test_code_verifier_is_alphanumeric() {
    let pkce = PkceChallenge::generate();

    assert!(pkce.code_verifier.chars().all(|c| c.is_alphanumeric()));
}

#[test]
fn test_s256_challenge_computation() {
    let pkce = PkceChallenge::generate();
    let verifier = pkce.code_verifier.clone();

    // Recompute challenge to verify it matches
    let computed_challenge = PkceChallenge::create_s256_challenge(&verifier);
    assert_eq!(computed_challenge, pkce.code_challenge);
}

#[test]
fn test_pkce_verification_success() {
    let pkce = PkceChallenge::generate();
    let verifier = pkce.code_verifier.clone();

    let result = pkce.verify(&verifier);
    assert!(result.is_ok());
}

#[test]
fn test_pkce_verification_failure_wrong_verifier() {
    let pkce = PkceChallenge::generate();

    let result = pkce.verify("wrong_verifier_12345");
    assert!(result.is_err());
}

#[test]
fn test_pkce_verification_failure_empty_verifier() {
    let pkce = PkceChallenge::generate();

    let result = pkce.verify("");
    assert!(result.is_err());
}

#[test]
fn test_state_parameter_length() {
    let pkce = PkceChallenge::generate();

    assert_eq!(pkce.state.len(), 32);
}

#[test]
fn test_state_is_alphanumeric() {
    let pkce = PkceChallenge::generate();

    assert!(pkce.state.chars().all(|c| c.is_alphanumeric()));
}

#[test]
fn test_multiple_pkce_generations_are_unique() {
    let pkce1 = PkceChallenge::generate();
    let pkce2 = PkceChallenge::generate();

    assert_ne!(pkce1.code_verifier, pkce2.code_verifier);
    assert_ne!(pkce1.code_challenge, pkce2.code_challenge);
    assert_ne!(pkce1.state, pkce2.state);
}

#[test]
fn test_authorization_code_creation() {
    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec!["read:content".to_string()],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    assert!(!code.code.is_empty());
    assert_eq!(code.client_id, "client123");
    assert_eq!(code.redirect_uri, "https://example.com/callback");
    assert_eq!(code.scopes.len(), 1);
    assert_eq!(code.code_challenge, "challenge123");
    assert_eq!(code.user_id, "user123");
    assert!(!code.used);
}

#[test]
fn test_authorization_code_not_expired_initially() {
    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    assert!(!code.is_expired());
}

#[test]
fn test_authorization_code_expiration_time() {
    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    let expected_expiration = code.created_at + chrono::Duration::minutes(10);
    assert_eq!(code.expires_at, expected_expiration);
}

#[test]
fn test_authorization_code_mark_as_used() {
    let mut code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    assert!(!code.used);
    code.mark_as_used();
    assert!(code.used);
}

#[test]
fn test_authorization_code_verify_pkce_success() {
    let pkce = PkceChallenge::generate();
    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        pkce.code_challenge.clone(),
        "user123".to_string(),
    );

    let result = code.verify_pkce(&pkce.code_verifier);
    assert!(result.is_ok());
}

#[test]
fn test_authorization_code_verify_pkce_failure() {
    let pkce = PkceChallenge::generate();
    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        pkce.code_challenge.clone(),
        "user123".to_string(),
    );

    let result = code.verify_pkce("wrong_verifier");
    assert!(result.is_err());
}

#[test]
fn test_authorization_code_length() {
    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    // Code should be 32 characters
    assert_eq!(code.code.len(), 32);
}

#[test]
fn test_authorization_code_is_alphanumeric() {
    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    assert!(code.code.chars().all(|c| c.is_alphanumeric()));
}

#[test]
fn test_authorization_code_uniqueness() {
    let code1 = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    let code2 = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        vec![],
        "challenge123".to_string(),
        "user123".to_string(),
    );

    assert_ne!(code1.code, code2.code);
}

#[test]
fn test_pkce_challenge_method_is_s256() {
    let pkce = PkceChallenge::generate();
    assert_eq!(pkce.code_challenge_method, "S256");
}

#[test]
fn test_authorization_code_includes_scopes() {
    let scopes = vec![
        "read:content".to_string(),
        "write:content".to_string(),
        "admin:users".to_string(),
    ];

    let code = AuthorizationCode::new(
        "client123".to_string(),
        "https://example.com/callback".to_string(),
        scopes.clone(),
        "challenge123".to_string(),
        "user123".to_string(),
    );

    assert_eq!(code.scopes, scopes);
    assert_eq!(code.scopes.len(), 3);
}
