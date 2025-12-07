use crate::{
    error::AuthError,
    mfa::{BackupCodeManager, MfaManager, TotpManager},
};
use sqlx::PgPool;

fn test_encryption_key() -> [u8; 32] {
    [42u8; 32]
}

#[sqlx::test]
async fn test_mfa_enrollment_flow(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_1".to_string();

    // Initiate enrollment
    let (secret, qr_code, backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    assert!(!secret.is_empty());
    assert!(qr_code.starts_with("data:image/png;base64,"));
    assert_eq!(backup_codes.len(), 10);

    // Verify user is not yet enrolled
    let is_enrolled = mfa_manager.is_enrolled(&user_id).await.unwrap();
    assert!(!is_enrolled);

    // Generate valid TOTP code
    let totp_manager = TotpManager::new(&test_encryption_key());
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    // Verify enrollment with valid code
    let result = mfa_manager.verify_enrollment(&user_id, &valid_code).await;
    assert!(result.is_ok());

    // Now user should be enrolled
    let is_enrolled = mfa_manager.is_enrolled(&user_id).await.unwrap();
    assert!(is_enrolled);

    Ok(())
}

#[sqlx::test]
async fn test_mfa_enrollment_invalid_code(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_2".to_string();

    // Initiate enrollment
    let (_secret, _qr_code, _backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    // Try to verify with invalid code
    let result = mfa_manager.verify_enrollment(&user_id, "000000").await;
    assert!(matches!(result, Err(AuthError::InvalidMfaCode)));

    // User should still not be enrolled
    let is_enrolled = mfa_manager.is_enrolled(&user_id).await.unwrap();
    assert!(!is_enrolled);

    Ok(())
}

#[sqlx::test]
async fn test_mfa_challenge_with_totp(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_3".to_string();

    // Setup: Enroll user
    let (secret, _qr_code, _backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    let totp_manager = TotpManager::new(&test_encryption_key());
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    mfa_manager
        .verify_enrollment(&user_id, &valid_code)
        .await
        .unwrap();

    // Test: Verify challenge with TOTP code
    let challenge_code = totp_manager.generate_current_code(&secret).unwrap();
    let result = mfa_manager
        .verify_challenge(&user_id, &challenge_code)
        .await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}

#[sqlx::test]
async fn test_mfa_challenge_with_backup_code(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_4".to_string();

    // Setup: Enroll user
    let (secret, _qr_code, backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    let totp_manager = TotpManager::new(&test_encryption_key());
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    mfa_manager
        .verify_enrollment(&user_id, &valid_code)
        .await
        .unwrap();

    // Test: Verify challenge with backup code
    let backup_code = backup_codes[0].clone();
    let result = mfa_manager.verify_challenge(&user_id, &backup_code).await;

    assert!(result.is_ok());
    assert!(result.unwrap());

    // Test: Backup code should be single-use
    let result = mfa_manager.verify_challenge(&user_id, &backup_code).await;

    assert!(matches!(result, Err(AuthError::InvalidMfaCode)));

    Ok(())
}

#[sqlx::test]
async fn test_mfa_challenge_invalid_code(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_5".to_string();

    // Setup: Enroll user
    let (secret, _qr_code, _backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    let totp_manager = TotpManager::new(&test_encryption_key());
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    mfa_manager
        .verify_enrollment(&user_id, &valid_code)
        .await
        .unwrap();

    // Test: Verify challenge with invalid code
    let result = mfa_manager.verify_challenge(&user_id, "000000").await;

    assert!(matches!(result, Err(AuthError::InvalidMfaCode)));

    Ok(())
}

#[sqlx::test]
async fn test_mfa_already_enrolled(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_6".to_string();

    // First enrollment
    let (secret, _qr_code, _backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    let totp_manager = TotpManager::new(&test_encryption_key());
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    mfa_manager
        .verify_enrollment(&user_id, &valid_code)
        .await
        .unwrap();

    // Try to enroll again
    let result = mfa_manager.initiate_enrollment(user_id.clone()).await;

    assert!(matches!(result, Err(AuthError::MfaAlreadyEnrolled)));

    Ok(())
}

#[sqlx::test]
async fn test_mfa_challenge_not_enrolled(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_7".to_string();

    // Try to verify challenge without enrollment
    let result = mfa_manager.verify_challenge(&user_id, "123456").await;

    assert!(matches!(result, Err(AuthError::MfaEnrollmentNotFound)));

    Ok(())
}

#[sqlx::test]
async fn test_mfa_disable(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_8".to_string();

    // Setup: Enroll user
    let (secret, _qr_code, _backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    let totp_manager = TotpManager::new(&test_encryption_key());
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    mfa_manager
        .verify_enrollment(&user_id, &valid_code)
        .await
        .unwrap();

    assert!(mfa_manager.is_enrolled(&user_id).await.unwrap());

    // Disable MFA
    mfa_manager.disable_mfa(&user_id).await.unwrap();

    // Verify user is no longer enrolled
    assert!(!mfa_manager.is_enrolled(&user_id).await.unwrap());

    Ok(())
}

#[sqlx::test]
async fn test_multiple_backup_codes(pool: PgPool) -> sqlx::Result<()> {
    let mfa_manager = MfaManager::new(pool.clone(), &test_encryption_key());
    let user_id = "test_user_9".to_string();

    // Setup: Enroll user
    let (secret, _qr_code, backup_codes) = mfa_manager
        .initiate_enrollment(user_id.clone())
        .await
        .unwrap();

    let totp_manager = TotpManager::new(&test_encryption_key());
    let valid_code = totp_manager.generate_current_code(&secret).unwrap();

    mfa_manager
        .verify_enrollment(&user_id, &valid_code)
        .await
        .unwrap();

    // Use first 3 backup codes
    for i in 0..3 {
        let result = mfa_manager
            .verify_challenge(&user_id, &backup_codes[i])
            .await;
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    // Verify used codes can't be reused
    for i in 0..3 {
        let result = mfa_manager
            .verify_challenge(&user_id, &backup_codes[i])
            .await;
        assert!(matches!(result, Err(AuthError::InvalidMfaCode)));
    }

    // Verify remaining codes still work
    let result = mfa_manager
        .verify_challenge(&user_id, &backup_codes[3])
        .await;
    assert!(result.is_ok());
    assert!(result.unwrap());

    Ok(())
}

#[test]
fn test_backup_code_manager() {
    let manager = BackupCodeManager::new();

    // Generate codes
    let codes = manager.generate_codes(10);
    assert_eq!(codes.len(), 10);

    // All codes should be 8 characters
    for code in &codes {
        assert_eq!(code.len(), 8);
    }

    // Hash codes
    let hashes = manager.hash_codes(&codes).unwrap();
    assert_eq!(hashes.len(), 10);

    // Verify valid code
    assert!(manager.verify_code(&codes[0], &hashes).unwrap());

    // Verify invalid code
    assert!(!manager.verify_code("INVALID1", &hashes).unwrap());
}

#[test]
fn test_totp_manager() {
    let manager = TotpManager::new(&test_encryption_key());

    // Generate secret
    let (secret, qr_code) = manager.generate_secret("test@example.com").unwrap();
    assert!(!secret.is_empty());
    assert!(qr_code.starts_with("data:image/png;base64,"));

    // Generate and verify code
    let code = manager.generate_current_code(&secret).unwrap();
    assert!(manager.verify_code(&secret, &code).unwrap());

    // Invalid code should fail
    assert!(!manager.verify_code(&secret, "000000").unwrap());

    // Encrypt and decrypt secret
    let encrypted = manager.encrypt_secret(&secret).unwrap();
    let decrypted = manager.decrypt_secret(&encrypted).unwrap();
    assert_eq!(secret, decrypted);
}
