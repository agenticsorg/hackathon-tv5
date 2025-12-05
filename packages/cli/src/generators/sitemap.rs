use anyhow::Result;
use serde_json::{json, Value};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SitemapEntry {
    pub loc: String,
    pub lastmod: String,
    pub changefreq: String,
    pub priority: f32,
}

/// Generate sitemap.llm.json structure
pub fn generate_sitemap(
    _site_path: &Path,
    base_url: &str,
    _pages: Vec<&str>,
) -> Result<Value> {
    let sitemap = json!({
        "version": "0.1",
        "site": {
            "title": "Website",
            "base_url": base_url,
            "description": "Generated sitemap",
            "updated": chrono::Utc::now().to_rfc3339()
        },
        "content": {
            "main": {
                "title": "Main Content",
                "priority": 1.0,
                "items": []
            }
        }
    });

    Ok(sitemap)
}

/// Generate sitemap.xml (standard XML format)
pub fn generate_sitemap_xml(entries: Vec<SitemapEntry>) -> Result<String> {
    let mut xml = String::new();

    // XML declaration
    xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
    xml.push('\n');

    // URL set with namespace
    xml.push_str(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9""#);
    xml.push_str(r#" xmlns:xhtml="http://www.w3.org/1999/xhtml""#);
    xml.push_str(r#" xmlns:arw="https://arw.dev/schema/">"#);
    xml.push('\n');

    // Add each URL entry
    for entry in entries {
        xml.push_str("  <url>\n");
        xml.push_str(&format!("    <loc>{}</loc>\n", escape_xml(&entry.loc)));
        xml.push_str(&format!("    <lastmod>{}</lastmod>\n", entry.lastmod));
        xml.push_str(&format!("    <changefreq>{}</changefreq>\n", entry.changefreq));
        xml.push_str(&format!("    <priority>{:.1}</priority>\n", entry.priority));
        xml.push_str("  </url>\n");
    }

    // Close URL set
    xml.push_str("</urlset>\n");

    Ok(xml)
}

/// Escape special XML characters
fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_sitemap_xml() {
        let entries = vec![
            SitemapEntry {
                loc: "https://example.com/page1".to_string(),
                lastmod: "2025-01-27".to_string(),
                changefreq: "weekly".to_string(),
                priority: 0.8,
            },
            SitemapEntry {
                loc: "https://example.com/page2".to_string(),
                lastmod: "2025-01-26".to_string(),
                changefreq: "daily".to_string(),
                priority: 0.9,
            },
        ];

        let xml = generate_sitemap_xml(entries).unwrap();

        assert!(xml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(xml.contains(r#"<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9""#));
        assert!(xml.contains("<loc>https://example.com/page1</loc>"));
        assert!(xml.contains("<lastmod>2025-01-27</lastmod>"));
        assert!(xml.contains("<changefreq>weekly</changefreq>"));
        assert!(xml.contains("<priority>0.8</priority>"));
        assert!(xml.contains("</urlset>"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("test & test"), "test &amp; test");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("a\"b'c"), "a&quot;b&apos;c");
    }
}
