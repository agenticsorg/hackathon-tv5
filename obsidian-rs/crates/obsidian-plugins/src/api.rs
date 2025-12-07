//! Plugin API for JavaScript compatibility

use crate::events::{Event, EventEmitter};
use crate::settings::PluginSettings;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command {
    /// Command ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Hotkey
    pub hotkey: Option<String>,
    /// Whether to show in command palette
    pub palette: bool,
    /// Icon
    pub icon: Option<String>,
}

impl Command {
    /// Create a new command
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
            hotkey: None,
            palette: true,
            icon: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set hotkey
    pub fn with_hotkey(mut self, hotkey: impl Into<String>) -> Self {
        self.hotkey = Some(hotkey.into());
        self
    }

    /// Set icon
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Hide from command palette
    pub fn hidden(mut self) -> Self {
        self.palette = false;
        self
    }
}

/// Command handler type
pub type CommandHandler = Box<dyn Fn() + Send + Sync>;

/// Ribbon action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RibbonAction {
    /// Action ID
    pub id: String,
    /// Icon
    pub icon: String,
    /// Tooltip
    pub tooltip: String,
}

/// Status bar item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBarItem {
    /// Item ID
    pub id: String,
    /// Content (HTML)
    pub content: String,
}

/// Plugin context providing access to app features
pub struct PluginContext {
    /// Plugin ID
    pub plugin_id: String,
    /// Event emitter
    emitter: Arc<EventEmitter>,
    /// Registered commands
    commands: RwLock<HashMap<String, Command>>,
    /// Command handlers
    handlers: RwLock<HashMap<String, Arc<CommandHandler>>>,
    /// Ribbon actions
    ribbon_actions: RwLock<Vec<RibbonAction>>,
    /// Status bar items
    status_bar_items: RwLock<Vec<StatusBarItem>>,
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new(plugin_id: impl Into<String>, emitter: Arc<EventEmitter>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            emitter,
            commands: RwLock::new(HashMap::new()),
            handlers: RwLock::new(HashMap::new()),
            ribbon_actions: RwLock::new(Vec::new()),
            status_bar_items: RwLock::new(Vec::new()),
        }
    }

    /// Get the event emitter
    pub fn emitter(&self) -> &Arc<EventEmitter> {
        &self.emitter
    }

    /// Register a command
    pub fn add_command<F>(&self, command: Command, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let full_id = format!("{}:{}", self.plugin_id, command.id);

        self.commands.write().insert(full_id.clone(), command);
        self.handlers
            .write()
            .insert(full_id, Arc::new(Box::new(handler)));
    }

    /// Remove a command
    pub fn remove_command(&self, id: &str) {
        let full_id = format!("{}:{}", self.plugin_id, id);
        self.commands.write().remove(&full_id);
        self.handlers.write().remove(&full_id);
    }

    /// Execute a command
    pub fn execute_command(&self, id: &str) -> bool {
        let full_id = if id.contains(':') {
            id.to_string()
        } else {
            format!("{}:{}", self.plugin_id, id)
        };

        if let Some(handler) = self.handlers.read().get(&full_id) {
            handler();
            self.emitter.emit(&Event::CommandExecuted {
                command_id: full_id,
            });
            true
        } else {
            false
        }
    }

    /// Get all registered commands
    pub fn commands(&self) -> Vec<Command> {
        self.commands.read().values().cloned().collect()
    }

    /// Add a ribbon action
    pub fn add_ribbon_action<F>(&self, icon: &str, tooltip: &str, handler: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        let id = format!("{}-ribbon-{}", self.plugin_id, self.ribbon_actions.read().len());

        self.ribbon_actions.write().push(RibbonAction {
            id: id.clone(),
            icon: icon.to_string(),
            tooltip: tooltip.to_string(),
        });

        self.handlers.write().insert(id, Arc::new(Box::new(handler)));
    }

    /// Get ribbon actions
    pub fn ribbon_actions(&self) -> Vec<RibbonAction> {
        self.ribbon_actions.read().clone()
    }

    /// Add a status bar item
    pub fn add_status_bar_item(&self, content: &str) -> String {
        let id = format!("{}-status-{}", self.plugin_id, self.status_bar_items.read().len());

        self.status_bar_items.write().push(StatusBarItem {
            id: id.clone(),
            content: content.to_string(),
        });

        id
    }

    /// Update a status bar item
    pub fn update_status_bar_item(&self, id: &str, content: &str) {
        let mut items = self.status_bar_items.write();
        if let Some(item) = items.iter_mut().find(|i| i.id == id) {
            item.content = content.to_string();
        }
    }

    /// Remove a status bar item
    pub fn remove_status_bar_item(&self, id: &str) {
        self.status_bar_items.write().retain(|i| i.id != id);
    }

    /// Get status bar items
    pub fn status_bar_items(&self) -> Vec<StatusBarItem> {
        self.status_bar_items.read().clone()
    }
}

/// Plugin API trait
pub trait PluginApi: Send + Sync {
    /// Called when plugin is loaded
    fn on_load(&mut self, ctx: &PluginContext) -> crate::PluginResult<()>;

    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> crate::PluginResult<()>;

    /// Called when settings change
    fn on_settings_changed(&mut self, _settings: &PluginSettings) {}

    /// Called on layout change
    fn on_layout_change(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_creation() {
        let cmd = Command::new("test", "Test Command")
            .with_description("A test command")
            .with_hotkey("Ctrl+T")
            .with_icon("check");

        assert_eq!(cmd.id, "test");
        assert_eq!(cmd.name, "Test Command");
        assert!(cmd.palette);
    }

    #[test]
    fn test_plugin_context() {
        let emitter = Arc::new(EventEmitter::new());
        let ctx = PluginContext::new("test-plugin", emitter);

        // Add command
        let executed = Arc::new(RwLock::new(false));
        let executed_clone = executed.clone();

        ctx.add_command(Command::new("test", "Test"), move || {
            *executed_clone.write() = true;
        });

        // Execute
        assert!(ctx.execute_command("test"));
        assert!(*executed.read());

        // Remove
        ctx.remove_command("test");
        assert!(!ctx.execute_command("test"));
    }

    #[test]
    fn test_ribbon_and_status_bar() {
        let emitter = Arc::new(EventEmitter::new());
        let ctx = PluginContext::new("test-plugin", emitter);

        ctx.add_ribbon_action("icon", "Tooltip", || {});
        assert_eq!(ctx.ribbon_actions().len(), 1);

        let status_id = ctx.add_status_bar_item("Status");
        assert_eq!(ctx.status_bar_items().len(), 1);

        ctx.update_status_bar_item(&status_id, "Updated");
        assert_eq!(ctx.status_bar_items()[0].content, "Updated");

        ctx.remove_status_bar_item(&status_id);
        assert!(ctx.status_bar_items().is_empty());
    }
}
