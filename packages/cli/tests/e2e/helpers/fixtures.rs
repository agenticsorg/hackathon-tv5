/// Test fixture generators for creating test data on-the-fly
use std::fs;
use std::path::Path;
use tempfile::TempDir;

/// Create a minimal valid llms.txt manifest
pub fn create_minimal_llms_txt() -> String {
    r#"version: "1.0"
profile: ARW-1

site:
  name: "Test Site"
  homepage: "https://example.com"

policies:
  training:
    allowed: false
  inference:
    allowed: true
  attribution:
    required: true
"#
    .to_string()
}

/// Create a complete llms.txt manifest with all features
pub fn create_complete_llms_txt() -> String {
    r#"version: "1.0"
profile: ARW-3

site:
  name: "Complete Test Site"
  description: "A comprehensive test site"
  homepage: "https://example.com"
  contact: "ai@example.com"

content:
  - url: "/"
    machine_view: "/index.llm.md"
    purpose: "homepage"
    priority: "high"
    chunks:
      - id: "intro"
        heading: "Introduction"
        description: "Welcome section"
      - id: "features"
        heading: "Features"

  - url: "/about"
    machine_view: "/about.llm.md"
    purpose: "about"
    priority: "medium"

actions:
  - id: "search"
    name: "Search"
    description: "Search the site"
    endpoint: "/api/search"
    method: "POST"
    auth: "none"
    parameters:
      - name: "query"
        type: "string"
        required: true

policies:
  training:
    allowed: false
    conditions: "Attribution required"
  inference:
    allowed: true
    rate_limits: "100 requests per hour"
  attribution:
    required: true
    format: "Site Name - URL"
"#
    .to_string()
}

/// Create a test directory with llms.txt
pub fn create_test_site(llms_txt_content: &str) -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    fs::write(temp_dir.path().join("llms.txt"), llms_txt_content).unwrap();
    temp_dir
}

/// Create a test directory with full ARW structure
pub fn create_complete_test_site() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create llms.txt
    fs::write(
        base_path.join("llms.txt"),
        create_complete_llms_txt(),
    )
    .unwrap();

    // Create machine views
    fs::write(
        base_path.join("index.llm.md"),
        "# Homepage\n\nWelcome to our site.",
    )
    .unwrap();

    fs::write(
        base_path.join("about.llm.md"),
        "# About Us\n\nLearn more about us.",
    )
    .unwrap();

    // Create .well-known directory
    let well_known = base_path.join(".well-known");
    fs::create_dir(&well_known).unwrap();

    temp_dir
}

/// Create invalid llms.txt with missing required fields
pub fn create_invalid_llms_txt_missing_version() -> String {
    r#"profile: ARW-1

site:
  name: "Test Site"
  homepage: "https://example.com"

policies:
  training:
    allowed: false
"#
    .to_string()
}

/// Create invalid llms.txt with wrong profile
pub fn create_invalid_llms_txt_wrong_profile() -> String {
    r#"version: "1.0"
profile: INVALID-PROFILE

site:
  name: "Test Site"
  homepage: "https://example.com"

policies:
  training:
    allowed: false
"#
    .to_string()
}

/// Create HTML page for testing generation
pub fn create_test_html_page() -> String {
    r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="description" content="Test page description">
    <title>Test Page</title>
</head>
<body>
    <header>
        <h1>Welcome to Test Site</h1>
        <nav>
            <a href="/">Home</a>
            <a href="/about">About</a>
        </nav>
    </header>

    <main>
        <article>
            <h2>Main Content</h2>
            <p>This is a test page for ARW generation.</p>

            <h3>Section 1</h3>
            <p>First section content.</p>

            <h3>Section 2</h3>
            <p>Second section content.</p>
        </article>
    </main>

    <footer>
        <p>&copy; 2024 Test Site</p>
    </footer>
</body>
</html>
"#
    .to_string()
}

/// Create malformed HTML for error testing
pub fn create_malformed_html() -> String {
    r#"<!DOCTYPE html>
<html>
<head>
    <title>Malformed</title>
<body>
    <h1>Missing closing head tag</h1>
    <p>Unclosed paragraph
    <div>
        Nested content
</html>
"#
    .to_string()
}

/// Create robots.txt content
pub fn create_robots_txt() -> String {
    r#"User-agent: *
Allow: /

# Agent-Ready Web Discovery
# See llms.txt for machine-readable content
Sitemap: https://example.com/sitemap.xml
"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_llms_txt_is_valid_yaml() {
        let content = create_minimal_llms_txt();
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_complete_llms_txt_is_valid_yaml() {
        let content = create_complete_llms_txt();
        let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
        assert!(parsed.is_ok());
    }

    #[test]
    fn test_create_test_site_creates_directory() {
        let temp_dir = create_test_site(&create_minimal_llms_txt());
        assert!(temp_dir.path().join("llms.txt").exists());
    }

    #[test]
    fn test_create_complete_test_site_has_all_files() {
        let temp_dir = create_complete_test_site();
        assert!(temp_dir.path().join("llms.txt").exists());
        assert!(temp_dir.path().join("index.llm.md").exists());
        assert!(temp_dir.path().join("about.llm.md").exists());
        assert!(temp_dir.path().join(".well-known").exists());
    }
}
