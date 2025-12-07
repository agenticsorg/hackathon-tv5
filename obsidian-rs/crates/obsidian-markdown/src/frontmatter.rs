//! Frontmatter parser for YAML metadata

use obsidian_core::note::Frontmatter;
use obsidian_core::position::{Location, Position};

/// Result of parsing frontmatter
#[derive(Debug, Clone)]
pub struct FrontmatterResult {
    /// Parsed frontmatter (if present)
    pub frontmatter: Option<Frontmatter>,
    /// Position of the frontmatter block
    pub position: Option<Position>,
    /// Content after frontmatter
    pub content: String,
    /// Byte offset where content starts
    pub content_start: usize,
}

/// Parser for YAML frontmatter
#[derive(Debug, Default)]
pub struct FrontmatterParser;

impl FrontmatterParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self
    }

    /// Parse frontmatter from markdown content
    pub fn parse(&self, content: &str) -> FrontmatterResult {
        // Check if content starts with ---
        if !content.starts_with("---") {
            return FrontmatterResult {
                frontmatter: None,
                position: None,
                content: content.to_string(),
                content_start: 0,
            };
        }

        // Find the end of frontmatter
        let after_start = &content[3..];

        // Must have a newline after opening ---
        if !after_start.starts_with('\n') && !after_start.starts_with("\r\n") {
            return FrontmatterResult {
                frontmatter: None,
                position: None,
                content: content.to_string(),
                content_start: 0,
            };
        }

        // Find closing ---
        let search_start = if after_start.starts_with("\r\n") { 2 } else { 1 };
        let search_content = &after_start[search_start..];

        let end_pos = search_content.find("\n---");
        let end_offset = match end_pos {
            Some(pos) => 3 + search_start + pos + 4, // 3 for opening ---, search_start for newline, pos, 4 for \n---
            None => {
                // Try \r\n---
                if let Some(pos) = search_content.find("\r\n---") {
                    3 + search_start + pos + 5
                } else {
                    return FrontmatterResult {
                        frontmatter: None,
                        position: None,
                        content: content.to_string(),
                        content_start: 0,
                    };
                }
            }
        };

        // Extract YAML content
        let yaml_start = 3 + search_start;
        let yaml_end = end_offset - 4;
        let yaml_content = &content[yaml_start..yaml_end];

        // Parse YAML
        let frontmatter = match serde_yaml::from_str::<Frontmatter>(yaml_content) {
            Ok(fm) => Some(fm),
            Err(_) => {
                // Try to parse as generic YAML and extract known fields
                match serde_yaml::from_str::<serde_yaml::Value>(yaml_content) {
                    Ok(value) => Some(parse_yaml_value(&value)),
                    Err(_) => None,
                }
            }
        };

        // Calculate position
        let (end_line, end_col) = offset_to_line_col(content, end_offset);
        let position = Position::new(
            Location::new(0, 0, 0),
            Location::new(end_line, end_col, end_offset),
        );

        // Find content start (skip newline after closing ---)
        let content_start = if content.len() > end_offset && content[end_offset..].starts_with('\n')
        {
            end_offset + 1
        } else if content.len() > end_offset + 1 && content[end_offset..].starts_with("\r\n") {
            end_offset + 2
        } else {
            end_offset
        };

        let remaining_content = if content_start < content.len() {
            content[content_start..].to_string()
        } else {
            String::new()
        };

        FrontmatterResult {
            frontmatter,
            position: Some(position),
            content: remaining_content,
            content_start,
        }
    }

    /// Serialize frontmatter to YAML
    pub fn serialize(&self, frontmatter: &Frontmatter) -> String {
        if frontmatter.is_empty() {
            return String::new();
        }

        match serde_yaml::to_string(frontmatter) {
            Ok(yaml) => format!("---\n{}---\n", yaml),
            Err(_) => String::new(),
        }
    }

    /// Update frontmatter in content
    pub fn update(&self, content: &str, frontmatter: &Frontmatter) -> String {
        let result = self.parse(content);
        let new_fm = self.serialize(frontmatter);

        if new_fm.is_empty() && result.frontmatter.is_none() {
            return content.to_string();
        }

        format!("{}{}", new_fm, result.content)
    }
}

/// Parse a YAML value into Frontmatter
fn parse_yaml_value(value: &serde_yaml::Value) -> Frontmatter {
    let mut fm = Frontmatter::default();

    if let serde_yaml::Value::Mapping(map) = value {
        for (key, val) in map {
            if let serde_yaml::Value::String(key_str) = key {
                match key_str.as_str() {
                    "title" => {
                        if let serde_yaml::Value::String(s) = val {
                            fm.title = Some(s.clone());
                        }
                    }
                    "tags" => {
                        fm.tags = parse_string_array(val);
                    }
                    "aliases" => {
                        fm.aliases = parse_string_array(val);
                    }
                    "cssclasses" | "cssclass" => {
                        fm.cssclasses = parse_string_array(val);
                    }
                    "publish" => {
                        if let serde_yaml::Value::Bool(b) = val {
                            fm.publish = Some(*b);
                        }
                    }
                    "created" => {
                        if let serde_yaml::Value::String(s) = val {
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                                fm.created = Some(dt.with_timezone(&chrono::Utc));
                            }
                        }
                    }
                    "modified" => {
                        if let serde_yaml::Value::String(s) = val {
                            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                                fm.modified = Some(dt.with_timezone(&chrono::Utc));
                            }
                        }
                    }
                    _ => {
                        // Store in extra
                        if let Ok(json_val) = serde_json::to_value(val) {
                            fm.extra.insert(key_str.clone(), json_val);
                        }
                    }
                }
            }
        }
    }

    fm
}

/// Parse a YAML value as a string array
fn parse_string_array(value: &serde_yaml::Value) -> Vec<String> {
    match value {
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .filter_map(|v| {
                if let serde_yaml::Value::String(s) = v {
                    Some(s.clone())
                } else {
                    None
                }
            })
            .collect(),
        serde_yaml::Value::String(s) => vec![s.clone()],
        _ => Vec::new(),
    }
}

/// Convert byte offset to line and column
fn offset_to_line_col(content: &str, offset: usize) -> (usize, usize) {
    let mut line = 0;
    let mut col = 0;
    let mut current_offset = 0;

    for c in content.chars() {
        if current_offset >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
        current_offset += c.len_utf8();
    }

    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_frontmatter() {
        let parser = FrontmatterParser::new();
        let result = parser.parse("# Hello World\n\nSome content.");

        assert!(result.frontmatter.is_none());
        assert!(result.position.is_none());
        assert_eq!(result.content, "# Hello World\n\nSome content.");
    }

    #[test]
    fn test_simple_frontmatter() {
        let parser = FrontmatterParser::new();
        let content = r#"---
title: My Note
tags:
  - test
  - example
---
# Content starts here
"#;

        let result = parser.parse(content);

        assert!(result.frontmatter.is_some());
        let fm = result.frontmatter.unwrap();
        assert_eq!(fm.title, Some("My Note".to_string()));
        assert_eq!(fm.tags, vec!["test", "example"]);
        assert_eq!(result.content, "# Content starts here\n");
    }

    #[test]
    fn test_frontmatter_with_aliases() {
        let parser = FrontmatterParser::new();
        let content = r#"---
title: Test
aliases:
  - Alias One
  - Alias Two
---
Content
"#;

        let result = parser.parse(content);
        let fm = result.frontmatter.unwrap();

        assert_eq!(fm.aliases, vec!["Alias One", "Alias Two"]);
    }

    #[test]
    fn test_frontmatter_position() {
        let parser = FrontmatterParser::new();
        let content = "---\ntitle: Test\n---\nContent";

        let result = parser.parse(content);

        assert!(result.position.is_some());
        let pos = result.position.unwrap();
        assert_eq!(pos.start.line, 0);
        assert!(pos.end.line >= 2);
    }

    #[test]
    fn test_serialize_frontmatter() {
        let parser = FrontmatterParser::new();
        let mut fm = Frontmatter::default();
        fm.title = Some("Test Title".to_string());
        fm.tags = vec!["tag1".to_string(), "tag2".to_string()];

        let yaml = parser.serialize(&fm);

        assert!(yaml.starts_with("---\n"));
        assert!(yaml.ends_with("---\n"));
        assert!(yaml.contains("title:"));
        assert!(yaml.contains("tags:"));
    }

    #[test]
    fn test_empty_frontmatter_serialize() {
        let parser = FrontmatterParser::new();
        let fm = Frontmatter::default();

        let yaml = parser.serialize(&fm);
        assert!(yaml.is_empty());
    }

    #[test]
    fn test_update_frontmatter() {
        let parser = FrontmatterParser::new();
        let content = "---\ntitle: Old\n---\n# Content";

        let mut new_fm = Frontmatter::default();
        new_fm.title = Some("New Title".to_string());

        let updated = parser.update(content, &new_fm);

        assert!(updated.contains("New Title"));
        assert!(updated.contains("# Content"));
    }
}
