//! Plugin system for Obsidian-rs
//!
//! This crate provides:
//! - Plugin loading and lifecycle management
//! - Plugin API for JavaScript compatibility
//! - Event system for plugin communication
//! - Settings and commands registration

pub mod api;
pub mod error;
pub mod events;
pub mod loader;
pub mod manifest;
pub mod registry;
pub mod settings;

pub use api::{PluginApi, PluginContext};
pub use error::{PluginError, PluginResult};
pub use events::{Event, EventEmitter};
pub use loader::PluginLoader;
pub use manifest::{PluginManifest, PluginVersion};
pub use registry::{Plugin, PluginRegistry, PluginState};
pub use settings::PluginSettings;

/// Prelude for common imports
pub mod prelude {
    pub use crate::api::{PluginApi, PluginContext};
    pub use crate::error::{PluginError, PluginResult};
    pub use crate::events::{Event, EventEmitter};
    pub use crate::loader::PluginLoader;
    pub use crate::manifest::{PluginManifest, PluginVersion};
    pub use crate::registry::{Plugin, PluginRegistry, PluginState};
    pub use crate::settings::PluginSettings;
}
