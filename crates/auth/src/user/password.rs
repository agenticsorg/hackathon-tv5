use crate::error::{AuthError, Result};
use argon2::{
    password_hash::{
        rand_core::OsRng, PasswordHash, PasswordHasher as Argon2PasswordHasher, PasswordVerifier,
        SaltString,
    },
    Algorithm, Argon2, ParamsBuilder, Version,
};

/// Password strength validation result
#[derive(Debug)]
pub struct PasswordStrength {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Password hasher using Argon2id
pub struct PasswordHasher {
    argon2: Argon2<'static>,
}

impl Default for PasswordHasher {
    fn default() -> Self {
        Self::new()
    }
}

impl PasswordHasher {
    /// Create a new password hasher with recommended parameters
    /// Memory: 19456 KiB (19 MiB)
    /// Iterations: 2
    /// Parallelism: 1
    pub fn new() -> Self {
        let params = ParamsBuilder::new()
            .m_cost(19456) // 19 MiB
            .t_cost(2) // 2 iterations
            .p_cost(1) // 1 thread
            .build()
            .expect("Failed to build Argon2 parameters");

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        Self { argon2 }
    }

    /// Hash a password using Argon2id
    pub fn hash_password(&self, password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);

        let password_hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AuthError::Internal(format!("Password hashing failed: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AuthError::Internal(format!("Invalid password hash: {}", e)))?;

        match self
            .argon2
            .verify_password(password.as_bytes(), &parsed_hash)
        {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(AuthError::Internal(format!(
                "Password verification failed: {}",
                e
            ))),
        }
    }

    /// Validate password strength
    /// Requirements:
    /// - Minimum 8 characters
    /// - At least one uppercase letter
    /// - At least one lowercase letter
    /// - At least one number
    pub fn validate_password_strength(password: &str) -> PasswordStrength {
        let mut errors = Vec::new();

        if password.len() < 8 {
            errors.push("Password must be at least 8 characters long".to_string());
        }

        if !password.chars().any(|c| c.is_uppercase()) {
            errors.push("Password must contain at least one uppercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_lowercase()) {
            errors.push("Password must contain at least one lowercase letter".to_string());
        }

        if !password.chars().any(|c| c.is_numeric()) {
            errors.push("Password must contain at least one number".to_string());
        }

        PasswordStrength {
            is_valid: errors.is_empty(),
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing() {
        let hasher = PasswordHasher::new();
        let password = "TestPassword123";

        let hash = hasher.hash_password(password).unwrap();
        assert!(hash.starts_with("$argon2id$"));

        let is_valid = hasher.verify_password(password, &hash).unwrap();
        assert!(is_valid);

        let is_invalid = hasher.verify_password("WrongPassword", &hash).unwrap();
        assert!(!is_invalid);
    }

    #[test]
    fn test_password_strength_valid() {
        let strength = PasswordHasher::validate_password_strength("Test1234");
        assert!(strength.is_valid);
        assert!(strength.errors.is_empty());
    }

    #[test]
    fn test_password_strength_too_short() {
        let strength = PasswordHasher::validate_password_strength("Test1");
        assert!(!strength.is_valid);
        assert!(strength
            .errors
            .contains(&"Password must be at least 8 characters long".to_string()));
    }

    #[test]
    fn test_password_strength_no_uppercase() {
        let strength = PasswordHasher::validate_password_strength("test1234");
        assert!(!strength.is_valid);
        assert!(strength
            .errors
            .contains(&"Password must contain at least one uppercase letter".to_string()));
    }

    #[test]
    fn test_password_strength_no_lowercase() {
        let strength = PasswordHasher::validate_password_strength("TEST1234");
        assert!(!strength.is_valid);
        assert!(strength
            .errors
            .contains(&"Password must contain at least one lowercase letter".to_string()));
    }

    #[test]
    fn test_password_strength_no_number() {
        let strength = PasswordHasher::validate_password_strength("TestTest");
        assert!(!strength.is_valid);
        assert!(strength
            .errors
            .contains(&"Password must contain at least one number".to_string()));
    }

    #[test]
    fn test_password_strength_multiple_errors() {
        let strength = PasswordHasher::validate_password_strength("test");
        assert!(!strength.is_valid);
        assert_eq!(strength.errors.len(), 3); // too short, no uppercase, no number
    }

    #[test]
    fn test_hash_uniqueness() {
        let hasher = PasswordHasher::new();
        let password = "TestPassword123";

        let hash1 = hasher.hash_password(password).unwrap();
        let hash2 = hasher.hash_password(password).unwrap();

        // Same password should produce different hashes due to different salts
        assert_ne!(hash1, hash2);

        // Both should verify correctly
        assert!(hasher.verify_password(password, &hash1).unwrap());
        assert!(hasher.verify_password(password, &hash2).unwrap());
    }
}
