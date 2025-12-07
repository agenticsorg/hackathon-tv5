pub mod backup_codes;
pub mod totp;

use crate::error::{AuthError, Result};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

pub use backup_codes::BackupCodeManager;
pub use totp::TotpManager;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MfaEnrollment {
    pub id: Uuid,
    pub user_id: String,
    pub encrypted_secret: Vec<u8>,
    pub backup_codes_hash: Vec<String>,
    pub is_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub verified_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct MfaManager {
    totp_manager: TotpManager,
    backup_code_manager: BackupCodeManager,
    db_pool: PgPool,
}

impl MfaManager {
    pub fn new(db_pool: PgPool, encryption_key: &[u8; 32]) -> Self {
        Self {
            totp_manager: TotpManager::new(encryption_key),
            backup_code_manager: BackupCodeManager::new(),
            db_pool,
        }
    }

    pub async fn initiate_enrollment(
        &self,
        user_id: String,
    ) -> Result<(String, String, Vec<String>)> {
        // Check if already enrolled
        if self.is_enrolled(&user_id).await? {
            return Err(AuthError::MfaAlreadyEnrolled);
        }

        // Generate TOTP secret and QR code
        let (secret, qr_code) = self.totp_manager.generate_secret(&user_id)?;

        // Generate backup codes
        let backup_codes = self.backup_code_manager.generate_codes(10);
        let backup_codes_hash = self.backup_code_manager.hash_codes(&backup_codes)?;

        // Encrypt secret
        let encrypted_secret = self.totp_manager.encrypt_secret(&secret)?;

        // Store in database (unverified)
        sqlx::query(
            r#"
            INSERT INTO mfa_enrollments (id, user_id, encrypted_secret, backup_codes_hash, is_verified)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id) DO UPDATE
            SET encrypted_secret = $3, backup_codes_hash = $4, is_verified = $5, created_at = NOW()
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(encrypted_secret)
        .bind(&backup_codes_hash)
        .bind(false)
        .execute(&self.db_pool)
        .await?;

        Ok((secret, qr_code, backup_codes))
    }

    pub async fn verify_enrollment(&self, user_id: &str, code: &str) -> Result<()> {
        // Get enrollment
        let enrollment = self.get_enrollment(user_id).await?;

        if enrollment.is_verified {
            return Ok(());
        }

        // Decrypt secret
        let secret = self
            .totp_manager
            .decrypt_secret(&enrollment.encrypted_secret)?;

        // Verify TOTP code
        if !self.totp_manager.verify_code(&secret, code)? {
            return Err(AuthError::InvalidMfaCode);
        }

        // Mark as verified
        sqlx::query(
            r#"
            UPDATE mfa_enrollments
            SET is_verified = true, verified_at = NOW()
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn verify_challenge(&self, user_id: &str, code: &str) -> Result<bool> {
        let enrollment = self.get_enrollment(user_id).await?;

        if !enrollment.is_verified {
            return Err(AuthError::MfaNotEnrolled);
        }

        // Try TOTP code first
        let secret = self
            .totp_manager
            .decrypt_secret(&enrollment.encrypted_secret)?;

        if self.totp_manager.verify_code(&secret, code)? {
            return Ok(true);
        }

        // Try backup code
        if self
            .backup_code_manager
            .verify_code(code, &enrollment.backup_codes_hash)?
        {
            // Remove used backup code
            self.remove_backup_code(user_id, code).await?;
            return Ok(true);
        }

        Err(AuthError::InvalidMfaCode)
    }

    pub async fn is_enrolled(&self, user_id: &str) -> Result<bool> {
        let result: Option<(bool,)> = sqlx::query_as(
            r#"
            SELECT is_verified FROM mfa_enrollments WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?;

        Ok(result.map(|r| r.0).unwrap_or(false))
    }

    pub async fn get_enrollment(&self, user_id: &str) -> Result<MfaEnrollment> {
        sqlx::query_as::<_, MfaEnrollment>(
            r#"
            SELECT id, user_id, encrypted_secret, backup_codes_hash, is_verified, created_at, verified_at
            FROM mfa_enrollments
            WHERE user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_optional(&self.db_pool)
        .await?
        .ok_or(AuthError::MfaEnrollmentNotFound)
    }

    async fn remove_backup_code(&self, user_id: &str, used_code: &str) -> Result<()> {
        let enrollment = self.get_enrollment(user_id).await?;

        let remaining_codes: Vec<String> = enrollment
            .backup_codes_hash
            .into_iter()
            .filter(|hash| {
                !self
                    .backup_code_manager
                    .verify_code(used_code, &[hash.clone()])
                    .unwrap_or(false)
            })
            .collect();

        sqlx::query(
            r#"
            UPDATE mfa_enrollments
            SET backup_codes_hash = $1
            WHERE user_id = $2
            "#,
        )
        .bind(&remaining_codes)
        .bind(user_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn disable_mfa(&self, user_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            DELETE FROM mfa_enrollments WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfa_enrollment_struct() {
        let enrollment = MfaEnrollment {
            id: Uuid::new_v4(),
            user_id: "user123".to_string(),
            encrypted_secret: vec![1, 2, 3],
            backup_codes_hash: vec!["hash1".to_string(), "hash2".to_string()],
            is_verified: false,
            created_at: chrono::Utc::now(),
            verified_at: None,
        };

        assert_eq!(enrollment.user_id, "user123");
        assert!(!enrollment.is_verified);
        assert_eq!(enrollment.backup_codes_hash.len(), 2);
    }
}
