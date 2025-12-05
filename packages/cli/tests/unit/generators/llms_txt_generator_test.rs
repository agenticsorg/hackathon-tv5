/// Comprehensive test suite for llms_txt generator
/// Tests manifest generation with various configurations
use arw_lib::generators::llms_txt::{generate, PolicyInfo, SiteInfo};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

fn create_basic_site_info() -> SiteInfo {
    SiteInfo {
        name: "Test Site".to_string(),
        description: "A test website".to_string(),
        homepage: "https://example.com".to_string(),
        contact: "test@example.com".to_string(),
    }
}

fn create_basic_policy_info() -> PolicyInfo {
    PolicyInfo {
        training_allowed: false,
        inference_allowed: true,
        attribution_required: true,
    }
}

// ============================================================================
// BASIC GENERATION TESTS
// ============================================================================

#[test]
fn test_generate_basic_manifest() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    let result = generate(temp_dir.path(), &site_info, &policy_info);
    assert!(result.is_ok(), "Generation should succeed");

    let manifest_path = temp_dir.path().join("llms.txt");
    assert!(manifest_path.exists(), "llms.txt should be created");
}

#[test]
fn test_generated_manifest_is_valid_yaml() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();
    assert!(parsed.is_mapping(), "Generated content should be valid YAML");
}

#[test]
fn test_generated_manifest_has_required_fields() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    assert!(parsed.get("version").is_some(), "Should have version");
    assert!(parsed.get("profile").is_some(), "Should have profile");
    assert!(parsed.get("site").is_some(), "Should have site");
    assert!(parsed.get("policies").is_some(), "Should have policies");
}

#[test]
fn test_generated_version_is_correct() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("version: 1.0") || content.contains("version: \"1.0\""),
        "Should have version 1.0"
    );
}

#[test]
fn test_generated_profile_is_arw1() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("profile: ARW-1"),
        "Should have profile ARW-1"
    );
}

// ============================================================================
// SITE INFORMATION TESTS
// ============================================================================

#[test]
fn test_site_name_is_included() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        name: "My Test Site".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("My Test Site"),
        "Should include site name"
    );
}

#[test]
fn test_site_description_is_included() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        description: "This is a test website for testing".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("This is a test website for testing"),
        "Should include site description"
    );
}

#[test]
fn test_site_homepage_is_included() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        homepage: "https://mysite.example.com".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("https://mysite.example.com"),
        "Should include homepage URL"
    );
}

#[test]
fn test_site_contact_is_included() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        contact: "admin@mysite.com".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("admin@mysite.com"),
        "Should include contact email"
    );
}

// ============================================================================
// POLICY TESTS
// ============================================================================

#[test]
fn test_training_allowed_true() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        training_allowed: true,
        ..create_basic_policy_info()
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    let training_allowed = parsed["policies"]["training"]["allowed"]
        .as_bool()
        .unwrap();
    assert!(training_allowed, "Training should be allowed");
}

#[test]
fn test_training_allowed_false() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        training_allowed: false,
        ..create_basic_policy_info()
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    let training_allowed = parsed["policies"]["training"]["allowed"]
        .as_bool()
        .unwrap();
    assert!(!training_allowed, "Training should be disallowed");
}

#[test]
fn test_inference_allowed_true() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        inference_allowed: true,
        ..create_basic_policy_info()
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    let inference_allowed = parsed["policies"]["inference"]["allowed"]
        .as_bool()
        .unwrap();
    assert!(inference_allowed, "Inference should be allowed");
}

#[test]
fn test_inference_allowed_false() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        inference_allowed: false,
        ..create_basic_policy_info()
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    let inference_allowed = parsed["policies"]["inference"]["allowed"]
        .as_bool()
        .unwrap();
    assert!(!inference_allowed, "Inference should be disallowed");
}

#[test]
fn test_attribution_required_true() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        attribution_required: true,
        ..create_basic_policy_info()
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    let attribution_required = parsed["policies"]["attribution"]["required"]
        .as_bool()
        .unwrap();
    assert!(attribution_required, "Attribution should be required");
}

#[test]
fn test_attribution_required_false() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        attribution_required: false,
        ..create_basic_policy_info()
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    let attribution_required = parsed["policies"]["attribution"]["required"]
        .as_bool()
        .unwrap();
    assert!(!attribution_required, "Attribution should not be required");
}

// ============================================================================
// CONTENT SECTION TESTS
// ============================================================================

#[test]
fn test_generated_manifest_has_content_example() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("content:"),
        "Should have content section"
    );
    assert!(
        content.contains("machine_view"),
        "Should have machine_view example"
    );
}

#[test]
fn test_content_example_has_homepage() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    assert!(
        parsed.get("content").is_some(),
        "Should have content array"
    );
    let content_array = parsed["content"].as_sequence().unwrap();
    assert!(!content_array.is_empty(), "Content should have example");

    let first_item = &content_array[0];
    assert_eq!(
        first_item["url"].as_str().unwrap(),
        "/",
        "First item should be homepage"
    );
}

// ============================================================================
// SPECIAL CHARACTER HANDLING TESTS
// ============================================================================

#[test]
fn test_escapes_quotes_in_site_name() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        name: "Test \"Quoted\" Site".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    // Should be properly escaped in YAML
    assert!(
        content.contains(r#"\""#) || content.contains("'Test \"Quoted\" Site'"),
        "Should escape quotes properly"
    );
}

#[test]
fn test_escapes_backslashes_in_description() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        description: "Path: C:\\Users\\Test".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    // Should be readable YAML
    let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
    assert!(parsed.is_ok(), "Should produce valid YAML with backslashes");
}

#[test]
fn test_handles_unicode_in_site_name() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        name: "Test Site 测试 🚀".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("测试"),
        "Should preserve Chinese characters"
    );
    assert!(content.contains("🚀"), "Should preserve emoji");
}

#[test]
fn test_handles_newlines_in_description() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = SiteInfo {
        description: "Line 1\nLine 2\nLine 3".to_string(),
        ..create_basic_site_info()
    };
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    // Should produce valid YAML
    let parsed: Result<serde_yaml::Value, _> = serde_yaml::from_str(&content);
    assert!(parsed.is_ok(), "Should handle newlines in YAML");
}

// ============================================================================
// COMMENTS AND FORMATTING TESTS
// ============================================================================

#[test]
fn test_includes_arw_header_comment() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("Agent-Ready Web"),
        "Should include ARW header"
    );
    assert!(
        content.contains("Generated by ARW CLI"),
        "Should mention ARW CLI"
    );
}

#[test]
fn test_includes_github_link() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("github.com/agent-ready-web/agent-ready-web"),
        "Should include GitHub link"
    );
}

#[test]
fn test_includes_helpful_comments() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();

    assert!(
        content.contains("Machine-Readable Content") || content.contains("Usage Policies"),
        "Should include section comments"
    );
}

// ============================================================================
// FILE OVERWRITE TESTS
// ============================================================================

#[test]
fn test_overwrites_existing_file() {
    let temp_dir = TempDir::new().unwrap();
    let manifest_path = temp_dir.path().join("llms.txt");

    // Create initial file
    fs::write(&manifest_path, "old content").unwrap();

    // Generate new manifest
    let site_info = create_basic_site_info();
    let policy_info = create_basic_policy_info();
    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let content = fs::read_to_string(&manifest_path).unwrap();
    assert!(
        !content.contains("old content"),
        "Should overwrite old content"
    );
    assert!(
        content.contains("version:"),
        "Should have new manifest content"
    );
}

// ============================================================================
// ERROR HANDLING TESTS
// ============================================================================

#[test]
fn test_fails_on_readonly_directory() {
    // This test is platform-specific and may not work on all systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        // Make directory read-only
        let metadata = fs::metadata(&readonly_dir).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o444);
        fs::set_permissions(&readonly_dir, permissions).unwrap();

        let site_info = create_basic_site_info();
        let policy_info = create_basic_policy_info();

        let result = generate(&readonly_dir, &site_info, &policy_info);
        assert!(result.is_err(), "Should fail on read-only directory");

        // Cleanup: restore permissions
        let metadata = fs::metadata(&readonly_dir).unwrap();
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&readonly_dir, permissions).unwrap();
    }
}

// ============================================================================
// POLICY COMBINATION TESTS
// ============================================================================

#[test]
fn test_all_policies_enabled() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        training_allowed: true,
        inference_allowed: true,
        attribution_required: true,
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    assert_eq!(
        parsed["policies"]["training"]["allowed"].as_bool().unwrap(),
        true
    );
    assert_eq!(
        parsed["policies"]["inference"]["allowed"].as_bool().unwrap(),
        true
    );
    assert_eq!(
        parsed["policies"]["attribution"]["required"]
            .as_bool()
            .unwrap(),
        true
    );
}

#[test]
fn test_all_policies_disabled() {
    let temp_dir = TempDir::new().unwrap();
    let site_info = create_basic_site_info();
    let policy_info = PolicyInfo {
        training_allowed: false,
        inference_allowed: false,
        attribution_required: false,
    };

    generate(temp_dir.path(), &site_info, &policy_info).unwrap();

    let manifest_path = temp_dir.path().join("llms.txt");
    let content = fs::read_to_string(&manifest_path).unwrap();
    let parsed: serde_yaml::Value = serde_yaml::from_str(&content).unwrap();

    assert_eq!(
        parsed["policies"]["training"]["allowed"].as_bool().unwrap(),
        false
    );
    assert_eq!(
        parsed["policies"]["inference"]["allowed"].as_bool().unwrap(),
        false
    );
    assert_eq!(
        parsed["policies"]["attribution"]["required"]
            .as_bool()
            .unwrap(),
        false
    );
}
