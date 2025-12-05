// Frontmatter parsing utilities

use serde_yaml::Value;

#[allow(dead_code)]
pub fn extract_frontmatter(content: &str) -> Option<Value> {
    if !content.starts_with("---\n") {
        return None;
    }

    let parts: Vec<&str> = content.splitn(3, "---\n").collect();
    if parts.len() < 3 {
        return None;
    }

    serde_yaml::from_str(parts[1]).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_valid_frontmatter() {
        let content = r#"---
title: Test Page
description: A test page
author: Test Author
tags:
  - test
  - example
---
# Main Content
This is the main content."#;

        let result = extract_frontmatter(content);
        assert!(result.is_some(), "Should extract valid frontmatter");

        let frontmatter = result.unwrap();
        assert!(frontmatter.is_mapping(), "Frontmatter should be a mapping");
        assert_eq!(
            frontmatter.get("title").and_then(|v| v.as_str()),
            Some("Test Page"),
            "Should extract title field"
        );
        assert_eq!(
            frontmatter.get("description").and_then(|v| v.as_str()),
            Some("A test page"),
            "Should extract description field"
        );
        assert_eq!(
            frontmatter.get("author").and_then(|v| v.as_str()),
            Some("Test Author"),
            "Should extract author field"
        );
    }

    #[test]
    fn test_extract_frontmatter_with_arrays() {
        let content = r#"---
tags:
  - rust
  - testing
  - yaml
numbers:
  - 1
  - 2
  - 3
---
Content here"#;

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        let tags = frontmatter.get("tags").and_then(|v| v.as_sequence());
        assert!(tags.is_some(), "Should extract tags array");
        assert_eq!(tags.unwrap().len(), 3, "Should have 3 tags");
    }

    #[test]
    fn test_extract_frontmatter_with_nested_objects() {
        let content = r#"---
meta:
  author:
    name: John Doe
    email: john@example.com
  date: 2025-01-17
---
Content"#;

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        let meta = frontmatter.get("meta");
        assert!(meta.is_some(), "Should extract nested meta object");
        let author = meta
            .and_then(|m| m.get("author"))
            .and_then(|a| a.get("name"))
            .and_then(|n| n.as_str());
        assert_eq!(author, Some("John Doe"), "Should extract nested author name");
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# Just a heading\nNo frontmatter here";
        let result = extract_frontmatter(content);
        assert!(result.is_none(), "Should return None for content without frontmatter");
    }

    #[test]
    fn test_empty_frontmatter() {
        let content = "---\n---\n# Content";
        let result = extract_frontmatter(content);
        // Empty YAML should parse as null
        assert!(result.is_some(), "Should handle empty frontmatter");
    }

    #[test]
    fn test_incomplete_frontmatter_delimiter() {
        let content = "---\ntitle: Test\n# Missing closing delimiter";
        let result = extract_frontmatter(content);
        assert!(result.is_none(), "Should return None for incomplete frontmatter");
    }

    #[test]
    fn test_frontmatter_not_at_start() {
        let content = "Some text before\n---\ntitle: Test\n---\nContent";
        let result = extract_frontmatter(content);
        assert!(result.is_none(), "Should return None if frontmatter not at start");
    }

    #[test]
    fn test_invalid_yaml_frontmatter() {
        let content = "---\ntitle: Test\n  invalid: : yaml: syntax\n---\nContent";
        let result = extract_frontmatter(content);
        assert!(result.is_none(), "Should return None for invalid YAML");
    }

    #[test]
    fn test_frontmatter_with_empty_content() {
        let content = "---\ntitle: Test\n---\n";
        let result = extract_frontmatter(content);
        assert!(result.is_some(), "Should extract frontmatter even with empty content");
    }

    #[test]
    fn test_frontmatter_with_special_characters() {
        let content = r#"---
title: "Test: With Special Characters!"
description: 'Single quotes with "double" inside'
code: |
  function test() {
    return true;
  }
---
Content"#;

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        assert_eq!(
            frontmatter.get("title").and_then(|v| v.as_str()),
            Some("Test: With Special Characters!"),
            "Should handle special characters in strings"
        );
    }

    #[test]
    fn test_frontmatter_with_boolean_and_numbers() {
        let content = r#"---
published: true
draft: false
count: 42
rating: 4.5
---
Content"#;

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        assert_eq!(
            frontmatter.get("published").and_then(|v| v.as_bool()),
            Some(true),
            "Should extract boolean true"
        );
        assert_eq!(
            frontmatter.get("draft").and_then(|v| v.as_bool()),
            Some(false),
            "Should extract boolean false"
        );
        assert_eq!(
            frontmatter.get("count").and_then(|v| v.as_i64()),
            Some(42),
            "Should extract integer"
        );
        assert_eq!(
            frontmatter.get("rating").and_then(|v| v.as_f64()),
            Some(4.5),
            "Should extract float"
        );
    }

    #[test]
    fn test_frontmatter_with_null_values() {
        let content = r#"---
title: Test
author: null
tags: ~
---
Content"#;

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        assert!(
            frontmatter.get("author").map(|v| v.is_null()).unwrap_or(false),
            "Should handle null values"
        );
    }

    #[test]
    fn test_multiple_documents_only_first_frontmatter() {
        let content = "---\ntitle: First\n---\nContent\n---\ntitle: Second\n---\nMore content";
        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        assert_eq!(
            frontmatter.get("title").and_then(|v| v.as_str()),
            Some("First"),
            "Should only extract first frontmatter block"
        );
    }

    #[test]
    fn test_frontmatter_with_unicode() {
        let content = r#"---
title: 测试 Test 🚀
author: José García
emoji: ✨🎉
---
Content"#;

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        assert_eq!(
            frontmatter.get("title").and_then(|v| v.as_str()),
            Some("测试 Test 🚀"),
            "Should handle Unicode characters"
        );
    }

    #[test]
    fn test_frontmatter_with_dates() {
        let content = r#"---
created: 2025-01-17
updated: 2025-01-17T10:30:00Z
---
Content"#;

        let result = extract_frontmatter(content);
        assert!(result.is_some());

        let frontmatter = result.unwrap();
        // YAML dates are parsed as strings
        let created = frontmatter.get("created");
        assert!(created.is_some(), "Should extract date field");
    }
}
