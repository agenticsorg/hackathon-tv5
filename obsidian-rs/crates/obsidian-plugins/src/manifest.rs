//! Plugin manifest types

use semver::Version;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Plugin version
pub type PluginVersion = Version;

/// Plugin manifest (manifest.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Unique plugin ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Minimum app version required
    #[serde(rename = "minAppVersion")]
    pub min_app_version: String,
    /// Plugin description
    pub description: String,
    /// Author name
    pub author: String,
    /// Author URL
    #[serde(rename = "authorUrl", default)]
    pub author_url: Option<String>,
    /// Funding URL
    #[serde(rename = "fundingUrl", default)]
    pub funding_url: Option<String>,
    /// Whether this is a desktop-only plugin
    #[serde(rename = "isDesktopOnly", default)]
    pub is_desktop_only: bool,
    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

impl PluginManifest {
    /// Load manifest from file
    pub fn load(path: impl AsRef<Path>) -> crate::PluginResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&content)?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Validate the manifest
    pub fn validate(&self) -> crate::PluginResult<()> {
        if self.id.is_empty() {
            return Err(crate::PluginError::InvalidManifest(
                "Plugin ID is required".to_string(),
            ));
        }

        if self.name.is_empty() {
            return Err(crate::PluginError::InvalidManifest(
                "Plugin name is required".to_string(),
            ));
        }

        // Validate version string
        if Version::parse(&self.version).is_err() {
            return Err(crate::PluginError::InvalidManifest(format!(
                "Invalid version: {}",
                self.version
            )));
        }

        Ok(())
    }

    /// Parse the version
    pub fn parsed_version(&self) -> Option<Version> {
        Version::parse(&self.version).ok()
    }

    /// Check if a dependency is satisfied
    pub fn check_dependency(&self, id: &str, version: &str) -> bool {
        if let Some(required) = self.dependencies.get(id) {
            // Simple version check - in production, use semver range
            version >= required.as_str()
        } else {
            true
        }
    }
}

/// Plugin data.json (settings persistence)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginData {
    /// Plugin-specific data
    #[serde(flatten)]
    pub data: HashMap<String, serde_json::Value>,
}

impl PluginData {
    /// Load data from file
    pub fn load(path: impl AsRef<Path>) -> crate::PluginResult<Self> {
        let path = path.as_ref();
        if path.exists() {
            let content = std::fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Save data to file
    pub fn save(&self, path: impl AsRef<Path>) -> crate::PluginResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get a value
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a value
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> crate::PluginResult<()> {
        let json_value = serde_json::to_value(value)?;
        self.data.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Remove a value
    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.data.remove(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let manifest = PluginManifest {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            min_app_version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            author_url: None,
            funding_url: None,
            is_desktop_only: false,
            dependencies: HashMap::new(),
        };

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_manifest_invalid_version() {
        let manifest = PluginManifest {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "invalid".to_string(),
            min_app_version: "1.0.0".to_string(),
            description: "".to_string(),
            author: "".to_string(),
            author_url: None,
            funding_url: None,
            is_desktop_only: false,
            dependencies: HashMap::new(),
        };

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_plugin_data() {
        let mut data = PluginData::default();

        data.set("key", "value").unwrap();
        let value: Option<String> = data.get("key");
        assert_eq!(value, Some("value".to_string()));

        data.remove("key");
        let value: Option<String> = data.get("key");
        assert!(value.is_none());
    }
}
