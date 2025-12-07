//! Plugin settings management

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Setting types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SettingType {
    /// Text input
    Text {
        default: String,
        placeholder: Option<String>,
    },
    /// Number input
    Number {
        default: f64,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
    },
    /// Boolean toggle
    Toggle { default: bool },
    /// Dropdown select
    Dropdown {
        default: String,
        options: Vec<DropdownOption>,
    },
    /// Color picker
    Color { default: String },
    /// Text area
    TextArea {
        default: String,
        placeholder: Option<String>,
        rows: Option<u32>,
    },
    /// Hotkey
    Hotkey { default: Option<String> },
    /// File/folder picker
    FilePicker {
        default: Option<String>,
        folder_only: bool,
    },
}

/// Dropdown option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropdownOption {
    /// Option value
    pub value: String,
    /// Display text
    pub display: String,
}

/// A setting definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingDefinition {
    /// Setting ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Setting type and config
    #[serde(flatten)]
    pub setting_type: SettingType,
}

impl SettingDefinition {
    /// Create a text setting
    pub fn text(id: &str, name: &str, description: &str, default: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            setting_type: SettingType::Text {
                default: default.to_string(),
                placeholder: None,
            },
        }
    }

    /// Create a number setting
    pub fn number(id: &str, name: &str, description: &str, default: f64) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            setting_type: SettingType::Number {
                default,
                min: None,
                max: None,
                step: None,
            },
        }
    }

    /// Create a toggle setting
    pub fn toggle(id: &str, name: &str, description: &str, default: bool) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            setting_type: SettingType::Toggle { default },
        }
    }

    /// Create a dropdown setting
    pub fn dropdown(
        id: &str,
        name: &str,
        description: &str,
        options: Vec<(&str, &str)>,
        default: &str,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            setting_type: SettingType::Dropdown {
                default: default.to_string(),
                options: options
                    .into_iter()
                    .map(|(value, display)| DropdownOption {
                        value: value.to_string(),
                        display: display.to_string(),
                    })
                    .collect(),
            },
        }
    }

    /// Get the default value
    pub fn default_value(&self) -> serde_json::Value {
        match &self.setting_type {
            SettingType::Text { default, .. } => serde_json::Value::String(default.clone()),
            SettingType::Number { default, .. } => {
                serde_json::Value::Number(serde_json::Number::from_f64(*default).unwrap())
            }
            SettingType::Toggle { default } => serde_json::Value::Bool(*default),
            SettingType::Dropdown { default, .. } => serde_json::Value::String(default.clone()),
            SettingType::Color { default } => serde_json::Value::String(default.clone()),
            SettingType::TextArea { default, .. } => serde_json::Value::String(default.clone()),
            SettingType::Hotkey { default } => default
                .clone()
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
            SettingType::FilePicker { default, .. } => default
                .clone()
                .map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        }
    }
}

/// Plugin settings container
pub struct PluginSettings {
    /// Plugin ID
    plugin_id: String,
    /// Settings path
    path: PathBuf,
    /// Setting definitions
    definitions: RwLock<Vec<SettingDefinition>>,
    /// Current values
    values: RwLock<HashMap<String, serde_json::Value>>,
    /// Whether settings have changed
    dirty: RwLock<bool>,
}

impl PluginSettings {
    /// Create new settings for a plugin
    pub fn new(plugin_id: &str, data_dir: impl AsRef<Path>) -> Self {
        let path = data_dir.as_ref().join(format!("{}.json", plugin_id));

        Self {
            plugin_id: plugin_id.to_string(),
            path,
            definitions: RwLock::new(Vec::new()),
            values: RwLock::new(HashMap::new()),
            dirty: RwLock::new(false),
        }
    }

    /// Load settings from disk
    pub fn load(&self) -> crate::PluginResult<()> {
        if self.path.exists() {
            let content = std::fs::read_to_string(&self.path)?;
            let values: HashMap<String, serde_json::Value> = serde_json::from_str(&content)?;
            *self.values.write() = values;
        }
        *self.dirty.write() = false;
        Ok(())
    }

    /// Save settings to disk
    pub fn save(&self) -> crate::PluginResult<()> {
        let values = self.values.read();
        let content = serde_json::to_string_pretty(&*values)?;

        // Ensure parent directory exists
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&self.path, content)?;
        *self.dirty.write() = false;
        Ok(())
    }

    /// Register a setting definition
    pub fn register(&self, definition: SettingDefinition) {
        let default = definition.default_value();
        let id = definition.id.clone();

        self.definitions.write().push(definition);

        // Set default if not already set
        let mut values = self.values.write();
        if !values.contains_key(&id) {
            values.insert(id, default);
        }
    }

    /// Get a setting value
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.values
            .read()
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get a setting value or default
    pub fn get_or<T: serde::de::DeserializeOwned>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }

    /// Set a setting value
    pub fn set<T: Serialize>(&self, key: &str, value: T) -> crate::PluginResult<()> {
        let json_value = serde_json::to_value(value)?;
        self.values.write().insert(key.to_string(), json_value);
        *self.dirty.write() = true;
        Ok(())
    }

    /// Reset a setting to default
    pub fn reset(&self, key: &str) {
        let definitions = self.definitions.read();
        if let Some(def) = definitions.iter().find(|d| d.id == key) {
            let default = def.default_value();
            self.values.write().insert(key.to_string(), default);
            *self.dirty.write() = true;
        }
    }

    /// Reset all settings to defaults
    pub fn reset_all(&self) {
        let definitions = self.definitions.read();
        let mut values = self.values.write();
        values.clear();

        for def in definitions.iter() {
            values.insert(def.id.clone(), def.default_value());
        }
        *self.dirty.write() = true;
    }

    /// Check if settings have unsaved changes
    pub fn is_dirty(&self) -> bool {
        *self.dirty.read()
    }

    /// Get all setting definitions
    pub fn definitions(&self) -> Vec<SettingDefinition> {
        self.definitions.read().clone()
    }

    /// Get all current values
    pub fn all_values(&self) -> HashMap<String, serde_json::Value> {
        self.values.read().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_setting_definition() {
        let setting = SettingDefinition::text("key", "Name", "Description", "default");
        let default = setting.default_value();
        assert_eq!(default, serde_json::Value::String("default".to_string()));
    }

    #[test]
    fn test_plugin_settings() {
        let dir = tempdir().unwrap();
        let settings = PluginSettings::new("test-plugin", dir.path());

        settings.register(SettingDefinition::text("name", "Name", "Your name", ""));
        settings.register(SettingDefinition::toggle("enabled", "Enabled", "Enable feature", true));

        // Get default
        let enabled: bool = settings.get_or("enabled", false);
        assert!(enabled);

        // Set value
        settings.set("name", "John").unwrap();
        let name: String = settings.get("name").unwrap();
        assert_eq!(name, "John");

        // Save and reload
        settings.save().unwrap();

        let settings2 = PluginSettings::new("test-plugin", dir.path());
        settings2.load().unwrap();
        let name: String = settings2.get("name").unwrap();
        assert_eq!(name, "John");
    }
}
