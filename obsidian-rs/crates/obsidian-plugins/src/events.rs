//! Event system for plugin communication

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

/// Event types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
    /// File created
    FileCreated { path: String },
    /// File modified
    FileModified { path: String },
    /// File deleted
    FileDeleted { path: String },
    /// File renamed
    FileRenamed { old_path: String, new_path: String },
    /// Active file changed
    ActiveFileChanged { path: Option<String> },
    /// Layout changed
    LayoutChanged,
    /// Plugin loaded
    PluginLoaded { id: String },
    /// Plugin unloaded
    PluginUnloaded { id: String },
    /// Settings changed
    SettingsChanged { plugin_id: String },
    /// Command executed
    CommandExecuted { command_id: String },
    /// Custom event
    Custom { name: String, payload: serde_json::Value },
}

impl Event {
    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::FileCreated { .. } => "file-created",
            Self::FileModified { .. } => "file-modified",
            Self::FileDeleted { .. } => "file-deleted",
            Self::FileRenamed { .. } => "file-renamed",
            Self::ActiveFileChanged { .. } => "active-file-changed",
            Self::LayoutChanged => "layout-changed",
            Self::PluginLoaded { .. } => "plugin-loaded",
            Self::PluginUnloaded { .. } => "plugin-unloaded",
            Self::SettingsChanged { .. } => "settings-changed",
            Self::CommandExecuted { .. } => "command-executed",
            Self::Custom { .. } => "custom",
        }
    }
}

/// Event handler type
pub type EventHandler = Box<dyn Fn(&Event) + Send + Sync>;

/// Event handler ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

/// Event emitter for pub/sub communication
pub struct EventEmitter {
    /// Event handlers
    handlers: RwLock<HashMap<String, Vec<(HandlerId, Arc<EventHandler>)>>>,
    /// Global handlers
    global_handlers: RwLock<Vec<(HandlerId, Arc<EventHandler>)>>,
    /// Next handler ID
    next_id: RwLock<u64>,
}

impl Default for EventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            global_handlers: RwLock::new(Vec::new()),
            next_id: RwLock::new(0),
        }
    }

    /// Generate next handler ID
    fn next_handler_id(&self) -> HandlerId {
        let mut id = self.next_id.write();
        *id += 1;
        HandlerId(*id)
    }

    /// Subscribe to a specific event type
    pub fn on<F>(&self, event_type: &str, handler: F) -> HandlerId
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let id = self.next_handler_id();
        let mut handlers = self.handlers.write();

        handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push((id, Arc::new(Box::new(handler))));

        id
    }

    /// Subscribe to all events
    pub fn on_any<F>(&self, handler: F) -> HandlerId
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        let id = self.next_handler_id();
        let mut handlers = self.global_handlers.write();
        handlers.push((id, Arc::new(Box::new(handler))));
        id
    }

    /// Unsubscribe a handler
    pub fn off(&self, id: HandlerId) {
        // Remove from specific handlers
        let mut handlers = self.handlers.write();
        for (_, handlers) in handlers.iter_mut() {
            handlers.retain(|(h_id, _)| *h_id != id);
        }

        // Remove from global handlers
        let mut global = self.global_handlers.write();
        global.retain(|(h_id, _)| *h_id != id);
    }

    /// Emit an event
    pub fn emit(&self, event: &Event) {
        let event_type = event.event_type();

        // Call specific handlers
        let handlers = self.handlers.read();
        if let Some(handlers) = handlers.get(event_type) {
            for (_, handler) in handlers {
                handler(event);
            }
        }

        // Call global handlers
        let global = self.global_handlers.read();
        for (_, handler) in global.iter() {
            handler(event);
        }
    }

    /// Clear all handlers
    pub fn clear(&self) {
        self.handlers.write().clear();
        self.global_handlers.write().clear();
    }

    /// Get number of handlers for an event type
    pub fn handler_count(&self, event_type: &str) -> usize {
        self.handlers
            .read()
            .get(event_type)
            .map(|h| h.len())
            .unwrap_or(0)
    }
}

/// Trait for objects that can emit events
pub trait Emits {
    /// Get the event emitter
    fn emitter(&self) -> &EventEmitter;

    /// Emit an event
    fn emit(&self, event: &Event) {
        self.emitter().emit(event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_event_emit() {
        let emitter = EventEmitter::new();
        let count = Arc::new(AtomicUsize::new(0));

        let count_clone = count.clone();
        emitter.on("file-created", move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        emitter.emit(&Event::FileCreated {
            path: "test.md".to_string(),
        });

        assert_eq!(count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_event_off() {
        let emitter = EventEmitter::new();
        let count = Arc::new(AtomicUsize::new(0));

        let count_clone = count.clone();
        let id = emitter.on("file-created", move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        emitter.emit(&Event::FileCreated {
            path: "test.md".to_string(),
        });
        assert_eq!(count.load(Ordering::SeqCst), 1);

        emitter.off(id);

        emitter.emit(&Event::FileCreated {
            path: "test2.md".to_string(),
        });
        assert_eq!(count.load(Ordering::SeqCst), 1); // Still 1
    }

    #[test]
    fn test_global_handler() {
        let emitter = EventEmitter::new();
        let count = Arc::new(AtomicUsize::new(0));

        let count_clone = count.clone();
        emitter.on_any(move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        emitter.emit(&Event::FileCreated {
            path: "test.md".to_string(),
        });
        emitter.emit(&Event::FileDeleted {
            path: "test.md".to_string(),
        });

        assert_eq!(count.load(Ordering::SeqCst), 2);
    }
}
