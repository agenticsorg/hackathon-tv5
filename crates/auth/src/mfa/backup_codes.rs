use crate::error::{AuthError, Result};
use rand::Rng;

const BACKUP_CODE_LENGTH: usize = 8;
const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Excludes ambiguous chars

#[derive(Debug, Clone)]
pub struct BackupCodeManager;

impl BackupCodeManager {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_codes(&self, count: usize) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let mut codes = Vec::with_capacity(count);

        for _ in 0..count {
            let code: String = (0..BACKUP_CODE_LENGTH)
                .map(|_| {
                    let idx = rng.gen_range(0..CHARSET.len());
                    CHARSET[idx] as char
                })
                .collect();
            codes.push(code);
        }

        codes
    }

    pub fn hash_codes(&self, codes: &[String]) -> Result<Vec<String>> {
        codes
            .iter()
            .map(|code| {
                bcrypt::hash(code, bcrypt::DEFAULT_COST)
                    .map_err(|e| AuthError::Internal(format!("Bcrypt hash failed: {}", e)))
            })
            .collect()
    }

    pub fn verify_code(&self, code: &str, hashes: &[String]) -> Result<bool> {
        for hash in hashes {
            match bcrypt::verify(code, hash) {
                Ok(true) => return Ok(true),
                Ok(false) => continue,
                Err(e) => {
                    tracing::warn!("Bcrypt verify error: {}", e);
                    continue;
                }
            }
        }
        Ok(false)
    }
}

impl Default for BackupCodeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_codes() {
        let manager = BackupCodeManager::new();
        let codes = manager.generate_codes(10);

        assert_eq!(codes.len(), 10);
        for code in &codes {
            assert_eq!(code.len(), BACKUP_CODE_LENGTH);
            assert!(code.chars().all(|c| CHARSET.contains(&(c as u8))));
        }
    }

    #[test]
    fn test_codes_are_unique() {
        let manager = BackupCodeManager::new();
        let codes = manager.generate_codes(100);

        let unique_codes: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique_codes.len(), 100);
    }

    #[test]
    fn test_hash_codes() {
        let manager = BackupCodeManager::new();
        let codes = vec!["ABCD1234".to_string(), "EFGH5678".to_string()];

        let hashes = manager.hash_codes(&codes).unwrap();

        assert_eq!(hashes.len(), 2);
        assert_ne!(hashes[0], hashes[1]);
        assert!(hashes[0].starts_with("$2"));
    }

    #[test]
    fn test_verify_code_success() {
        let manager = BackupCodeManager::new();
        let codes = vec!["TESTCODE".to_string()];
        let hashes = manager.hash_codes(&codes).unwrap();

        assert!(manager.verify_code("TESTCODE", &hashes).unwrap());
    }

    #[test]
    fn test_verify_code_failure() {
        let manager = BackupCodeManager::new();
        let codes = vec!["TESTCODE".to_string()];
        let hashes = manager.hash_codes(&codes).unwrap();

        assert!(!manager.verify_code("WRONGCODE", &hashes).unwrap());
    }

    #[test]
    fn test_verify_code_empty_hashes() {
        let manager = BackupCodeManager::new();
        let hashes: Vec<String> = vec![];

        assert!(!manager.verify_code("ANYCODE", &hashes).unwrap());
    }

    #[test]
    fn test_backup_code_charset() {
        assert_eq!(CHARSET.len(), 32);
        assert!(!CHARSET.contains(&b'0'));
        assert!(!CHARSET.contains(&b'O'));
        assert!(!CHARSET.contains(&b'I'));
        assert!(!CHARSET.contains(&b'1'));
    }
}
