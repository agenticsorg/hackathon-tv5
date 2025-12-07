use crate::error::{AuthError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// OAuth 2.0 Scopes for Media Gateway
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Scope {
    // Read scopes
    ReadContent,
    ReadWatchlist,
    ReadPreferences,
    ReadRecommendations,
    ReadDevices,

    // Write scopes
    WriteWatchlist,
    WritePreferences,
    WriteRatings,
    WriteDevices,

    // Special scopes
    PlaybackControl,
    AdminFull,
}

impl Scope {
    pub fn as_str(&self) -> &str {
        match self {
            Scope::ReadContent => "read:content",
            Scope::ReadWatchlist => "read:watchlist",
            Scope::ReadPreferences => "read:preferences",
            Scope::ReadRecommendations => "read:recommendations",
            Scope::ReadDevices => "read:devices",
            Scope::WriteWatchlist => "write:watchlist",
            Scope::WritePreferences => "write:preferences",
            Scope::WriteRatings => "write:ratings",
            Scope::WriteDevices => "write:devices",
            Scope::PlaybackControl => "playback:control",
            Scope::AdminFull => "admin:full",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "read:content" => Some(Scope::ReadContent),
            "read:watchlist" => Some(Scope::ReadWatchlist),
            "read:preferences" => Some(Scope::ReadPreferences),
            "read:recommendations" => Some(Scope::ReadRecommendations),
            "read:devices" => Some(Scope::ReadDevices),
            "write:watchlist" => Some(Scope::WriteWatchlist),
            "write:preferences" => Some(Scope::WritePreferences),
            "write:ratings" => Some(Scope::WriteRatings),
            "write:devices" => Some(Scope::WriteDevices),
            "playback:control" => Some(Scope::PlaybackControl),
            "admin:full" => Some(Scope::AdminFull),
            _ => None,
        }
    }

    pub fn requires_consent(&self) -> bool {
        matches!(self, Scope::PlaybackControl | Scope::AdminFull)
    }

    /// Get implied scopes (e.g., write implies read)
    pub fn implied_scopes(&self) -> Vec<Scope> {
        match self {
            Scope::WriteWatchlist => vec![Scope::ReadWatchlist],
            Scope::WritePreferences => vec![Scope::ReadPreferences],
            Scope::WriteDevices => vec![Scope::ReadDevices],
            _ => vec![],
        }
    }
}

pub struct ScopeManager {
    available_scopes: HashSet<Scope>,
}

impl ScopeManager {
    pub fn new() -> Self {
        let mut available_scopes = HashSet::new();
        available_scopes.insert(Scope::ReadContent);
        available_scopes.insert(Scope::ReadWatchlist);
        available_scopes.insert(Scope::ReadPreferences);
        available_scopes.insert(Scope::ReadRecommendations);
        available_scopes.insert(Scope::ReadDevices);
        available_scopes.insert(Scope::WriteWatchlist);
        available_scopes.insert(Scope::WritePreferences);
        available_scopes.insert(Scope::WriteRatings);
        available_scopes.insert(Scope::WriteDevices);
        available_scopes.insert(Scope::PlaybackControl);

        Self { available_scopes }
    }

    /// Parse space-separated scope string
    pub fn parse_scopes(&self, scope_string: &str) -> Result<Vec<Scope>> {
        let mut scopes = Vec::new();

        for scope_str in scope_string.split_whitespace() {
            if let Some(scope) = Scope::from_str(scope_str) {
                scopes.push(scope);
            } else {
                return Err(AuthError::InvalidScope(scope_str.to_string()));
            }
        }

        Ok(scopes)
    }

    /// Validate requested scopes
    pub fn validate_scopes(&self, scopes: &[Scope]) -> Result<()> {
        for scope in scopes {
            if !self.available_scopes.contains(scope) {
                return Err(AuthError::InvalidScope(scope.as_str().to_string()));
            }
        }
        Ok(())
    }

    /// Expand scopes to include implied scopes
    pub fn expand_scopes(&self, scopes: Vec<Scope>) -> Vec<Scope> {
        let mut expanded = HashSet::new();

        for scope in scopes {
            expanded.insert(scope.clone());

            // Add implied scopes
            for implied in scope.implied_scopes() {
                expanded.insert(implied);
            }
        }

        expanded.into_iter().collect()
    }

    /// Check if scopes contain a specific scope
    pub fn has_scope(&self, scopes: &[Scope], required: &Scope) -> bool {
        scopes.contains(required)
    }

    /// Convert scopes to space-separated string
    pub fn scopes_to_string(scopes: &[Scope]) -> String {
        scopes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Parse string scopes from JWT claims
    pub fn parse_from_strings(&self, scope_strings: &[String]) -> Result<Vec<Scope>> {
        let mut scopes = Vec::new();

        for scope_str in scope_strings {
            if let Some(scope) = Scope::from_str(scope_str) {
                scopes.push(scope);
            } else {
                return Err(AuthError::InvalidScope(scope_str.clone()));
            }
        }

        Ok(scopes)
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_parsing() {
        let manager = ScopeManager::new();

        let scopes = manager
            .parse_scopes("read:content write:watchlist")
            .unwrap();
        assert_eq!(scopes.len(), 2);
        assert!(scopes.contains(&Scope::ReadContent));
        assert!(scopes.contains(&Scope::WriteWatchlist));
    }

    #[test]
    fn test_invalid_scope() {
        let manager = ScopeManager::new();

        let result = manager.parse_scopes("invalid:scope");
        assert!(result.is_err());
    }

    #[test]
    fn test_scope_expansion() {
        let manager = ScopeManager::new();

        let scopes = vec![Scope::WriteWatchlist];
        let expanded = manager.expand_scopes(scopes);

        assert!(expanded.contains(&Scope::WriteWatchlist));
        assert!(expanded.contains(&Scope::ReadWatchlist));
    }

    #[test]
    fn test_scope_to_string() {
        let scopes = vec![Scope::ReadContent, Scope::WriteWatchlist];
        let scope_string = ScopeManager::scopes_to_string(&scopes);

        assert!(scope_string.contains("read:content"));
        assert!(scope_string.contains("write:watchlist"));
    }

    #[test]
    fn test_scope_consent_required() {
        assert!(Scope::PlaybackControl.requires_consent());
        assert!(!Scope::ReadContent.requires_consent());
    }
}
