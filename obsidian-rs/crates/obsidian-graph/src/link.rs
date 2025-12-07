//! Link types and resolution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use parking_lot::RwLock;

/// Type of link
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkType {
    /// Wikilink: [[note]]
    Wiki,
    /// Markdown link: [text](url)
    Markdown,
    /// Embed: ![[note]]
    Embed,
    /// Tag: #tag
    Tag,
}

/// A link between notes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    /// Source file path
    pub source: String,
    /// Target (note name or path)
    pub target: String,
    /// Display text
    pub display: Option<String>,
    /// Link type
    pub link_type: LinkType,
    /// Target heading (if any)
    pub heading: Option<String>,
    /// Target block ID (if any)
    pub block_id: Option<String>,
    /// Line number in source
    pub line: Option<usize>,
    /// Whether the link is resolved
    pub resolved: bool,
}

impl Link {
    /// Create a new link
    pub fn new(source: impl Into<String>, target: impl Into<String>, link_type: LinkType) -> Self {
        Self {
            source: source.into(),
            target: target.into(),
            display: None,
            link_type,
            heading: None,
            block_id: None,
            line: None,
            resolved: false,
        }
    }

    /// Set display text
    pub fn with_display(mut self, display: impl Into<String>) -> Self {
        self.display = Some(display.into());
        self
    }

    /// Set heading
    pub fn with_heading(mut self, heading: impl Into<String>) -> Self {
        self.heading = Some(heading.into());
        self
    }

    /// Set block ID
    pub fn with_block_id(mut self, block_id: impl Into<String>) -> Self {
        self.block_id = Some(block_id.into());
        self
    }

    /// Set line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Mark as resolved
    pub fn with_resolved(mut self, resolved: bool) -> Self {
        self.resolved = resolved;
        self
    }

    /// Get the full target path including subpath
    pub fn full_target(&self) -> String {
        let mut full = self.target.clone();
        if let Some(ref heading) = self.heading {
            full.push('#');
            full.push_str(heading);
        }
        if let Some(ref block_id) = self.block_id {
            full.push_str("#^");
            full.push_str(block_id);
        }
        full
    }
}

/// Link resolver for finding note files
pub struct LinkResolver {
    /// Note name to path mapping
    notes: RwLock<HashMap<String, PathBuf>>,
    /// Aliases to path mapping
    aliases: RwLock<HashMap<String, PathBuf>>,
    /// Vault root path
    vault_path: PathBuf,
}

impl LinkResolver {
    /// Create a new link resolver
    pub fn new(vault_path: impl Into<PathBuf>) -> Self {
        Self {
            notes: RwLock::new(HashMap::new()),
            aliases: RwLock::new(HashMap::new()),
            vault_path: vault_path.into(),
        }
    }

    /// Register a note
    pub fn register_note(&self, path: impl Into<PathBuf>, aliases: &[String]) {
        let path = path.into();

        // Extract note name from path
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();

        let name_lower = name.to_lowercase();

        // Register by name
        {
            let mut notes = self.notes.write();
            notes.insert(name_lower.clone(), path.clone());
            // Also register with extension
            notes.insert(format!("{}.md", name_lower), path.clone());
        }

        // Register aliases
        {
            let mut alias_map = self.aliases.write();
            for alias in aliases {
                alias_map.insert(alias.to_lowercase(), path.clone());
            }
        }
    }

    /// Unregister a note
    pub fn unregister_note(&self, path: &Path) {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Remove from notes
        {
            let mut notes = self.notes.write();
            notes.remove(&name);
            notes.remove(&format!("{}.md", name));
        }

        // Remove aliases pointing to this path
        {
            let mut aliases = self.aliases.write();
            aliases.retain(|_, p| p != path);
        }
    }

    /// Resolve a link target to a file path
    pub fn resolve(&self, target: &str) -> Option<PathBuf> {
        // Remove any subpath (heading or block)
        let target = target.split('#').next().unwrap_or(target);
        let target_lower = target.to_lowercase();

        // Try exact match
        {
            let notes = self.notes.read();
            if let Some(path) = notes.get(&target_lower) {
                return Some(path.clone());
            }
        }

        // Try aliases
        {
            let aliases = self.aliases.read();
            if let Some(path) = aliases.get(&target_lower) {
                return Some(path.clone());
            }
        }

        // Try as relative path
        let relative = self.vault_path.join(target);
        if relative.exists() {
            return Some(relative);
        }

        // Try with .md extension
        let with_ext = self.vault_path.join(format!("{}.md", target));
        if with_ext.exists() {
            return Some(with_ext);
        }

        None
    }

    /// Check if a target is resolved
    pub fn is_resolved(&self, target: &str) -> bool {
        self.resolve(target).is_some()
    }

    /// Get all unresolved links from a set of links
    pub fn find_unresolved<'a>(&self, links: &'a [Link]) -> Vec<&'a Link> {
        links
            .iter()
            .filter(|link| !self.is_resolved(&link.target))
            .collect()
    }

    /// Get number of registered notes
    pub fn note_count(&self) -> usize {
        self.notes.read().len() / 2 // Divided by 2 because we register both with and without extension
    }

    /// Get all registered note names
    pub fn all_notes(&self) -> Vec<String> {
        self.notes
            .read()
            .keys()
            .filter(|k| !k.ends_with(".md"))
            .cloned()
            .collect()
    }

    /// Suggest completions for a partial link
    pub fn suggest_completions(&self, partial: &str, limit: usize) -> Vec<String> {
        let partial_lower = partial.to_lowercase();
        let notes = self.notes.read();

        let mut suggestions: Vec<_> = notes
            .keys()
            .filter(|k| !k.ends_with(".md"))
            .filter(|k| k.contains(&partial_lower))
            .cloned()
            .collect();

        // Sort by relevance (starts with > contains)
        suggestions.sort_by(|a, b| {
            let a_starts = a.starts_with(&partial_lower);
            let b_starts = b.starts_with(&partial_lower);

            match (a_starts, b_starts) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.len().cmp(&b.len()),
            }
        });

        suggestions.truncate(limit);
        suggestions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_link_creation() {
        let link = Link::new("source.md", "target", LinkType::Wiki)
            .with_display("Target Note")
            .with_heading("Section")
            .with_line(10)
            .with_resolved(true);

        assert_eq!(link.source, "source.md");
        assert_eq!(link.target, "target");
        assert_eq!(link.display, Some("Target Note".to_string()));
        assert_eq!(link.heading, Some("Section".to_string()));
        assert_eq!(link.line, Some(10));
        assert!(link.resolved);
    }

    #[test]
    fn test_full_target() {
        let link = Link::new("source.md", "target", LinkType::Wiki)
            .with_heading("Section");

        assert_eq!(link.full_target(), "target#Section");

        let link2 = Link::new("source.md", "target", LinkType::Wiki)
            .with_block_id("abc123");

        assert_eq!(link2.full_target(), "target#^abc123");
    }

    #[test]
    fn test_link_resolver() {
        let dir = tempdir().unwrap();
        let resolver = LinkResolver::new(dir.path());

        // Create test files
        let note1_path = dir.path().join("note1.md");
        std::fs::write(&note1_path, "# Note 1").unwrap();

        let note2_path = dir.path().join("note2.md");
        std::fs::write(&note2_path, "# Note 2").unwrap();

        // Register notes
        resolver.register_note(&note1_path, &["First Note".to_string()]);
        resolver.register_note(&note2_path, &[]);

        // Test resolution
        assert!(resolver.resolve("note1").is_some());
        assert!(resolver.resolve("Note1").is_some()); // Case insensitive
        assert!(resolver.resolve("First Note").is_some()); // Alias
        assert!(resolver.resolve("nonexistent").is_none());
    }

    #[test]
    fn test_suggest_completions() {
        let dir = tempdir().unwrap();
        let resolver = LinkResolver::new(dir.path());

        resolver.register_note(dir.path().join("hello.md"), &[]);
        resolver.register_note(dir.path().join("hello-world.md"), &[]);
        resolver.register_note(dir.path().join("help.md"), &[]);
        resolver.register_note(dir.path().join("goodbye.md"), &[]);

        let suggestions = resolver.suggest_completions("hel", 10);
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions.contains(&"hello".to_string()));
        assert!(suggestions.contains(&"hello-world".to_string()));
        assert!(suggestions.contains(&"help".to_string()));
    }
}
