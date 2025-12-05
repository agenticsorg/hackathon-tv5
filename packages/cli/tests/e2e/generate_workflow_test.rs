/// End-to-end tests for the generate workflow
mod common;
mod helpers;

use common::*;
use helpers::*;
use std::fs;

#[test]
fn test_generate_from_html_file() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("index.html");

    fs::write(&html_path, create_test_html_page()).unwrap();

    let output = run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    assert_output_contains(&output, "Success");

    // Verify .llm.md file was created
    let llm_md_path = temp_dir.path().join("index.llm.md");
    assert!(llm_md_path.exists());
    assert_file_contains(&llm_md_path, "# Welcome to Test Site");
}

#[test]
fn test_generate_recursive_directory() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Create nested HTML files
    fs::write(temp_dir.path().join("index.html"), create_test_html_page()).unwrap();

    let subdir = temp_dir.path().join("pages");
    fs::create_dir(&subdir).unwrap();
    fs::write(subdir.join("about.html"), create_test_html_page()).unwrap();

    let output = run_cli_success(
        &[
            "generate",
            temp_dir.path().to_str().unwrap(),
            "--recursive",
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    assert_output_contains(&output, "Success");

    // Verify both files were processed
    assert!(temp_dir.path().join("index.llm.md").exists());
    assert!(subdir.join("about.llm.md").exists());
}

#[test]
fn test_generate_with_force_overwrite() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("index.html");
    let llm_md_path = temp_dir.path().join("index.llm.md");

    fs::write(&html_path, create_test_html_page()).unwrap();
    fs::write(&llm_md_path, "Existing content").unwrap();

    let output = run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
            "--force",
        ],
        None,
    );

    assert_output_contains(&output, "Success");

    // Verify file was overwritten
    let content = fs::read_to_string(&llm_md_path).unwrap();
    assert!(!content.contains("Existing content"));
    assert!(content.contains("Welcome to Test Site"));
}

#[test]
fn test_generate_without_force_preserves_existing() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("index.html");
    let llm_md_path = temp_dir.path().join("index.llm.md");

    fs::write(&html_path, create_test_html_page()).unwrap();
    fs::write(&llm_md_path, "Existing content").unwrap();

    // Without --force, should skip existing file
    let output = run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    // Verify existing file was not modified
    let content = fs::read_to_string(&llm_md_path).unwrap();
    assert!(content.contains("Existing content"));
}

#[test]
fn test_generate_auto_detects_format() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("page.html");

    fs::write(&html_path, create_test_html_page()).unwrap();

    let output = run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--format",
            "auto",
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    assert_output_contains(&output, "Success");
    assert!(temp_dir.path().join("page.llm.md").exists());
}

#[test]
fn test_generate_from_markdown() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let md_path = temp_dir.path().join("content.md");

    fs::write(
        &md_path,
        "# My Article\n\nThis is some content.\n\n## Section 1\n\nMore content.",
    )
    .unwrap();

    let output = run_cli_success(
        &[
            "generate",
            md_path.to_str().unwrap(),
            "--format",
            "markdown",
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    assert_output_contains(&output, "Success");

    let llm_md_path = temp_dir.path().join("content.llm.md");
    assert!(llm_md_path.exists());
    assert_file_contains(&llm_md_path, "# My Article");
}

#[test]
fn test_generate_handles_malformed_html() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("malformed.html");

    fs::write(&html_path, create_malformed_html()).unwrap();

    // Should still succeed, HTML parser is forgiving
    let output = run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    assert_output_contains(&output, "Success");
    assert!(temp_dir.path().join("malformed.llm.md").exists());
}

#[test]
fn test_generate_missing_source_file() {
    setup_test_env();

    let (_stdout, stderr) = run_cli_failure(
        &["generate", "/nonexistent/file.html", "--output", "/tmp"],
        None,
    );

    assert_output_contains(&stderr, "not found");
}

#[test]
fn test_generate_preserves_heading_structure() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("structured.html");

    let html = r#"
<!DOCTYPE html>
<html>
<head><title>Structured</title></head>
<body>
    <h1>Main Title</h1>
    <h2>Section 1</h2>
    <p>Content 1</p>
    <h3>Subsection 1.1</h3>
    <p>Content 1.1</p>
    <h2>Section 2</h2>
    <p>Content 2</p>
</body>
</html>
    "#;

    fs::write(&html_path, html).unwrap();

    run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    let llm_md_path = temp_dir.path().join("structured.llm.md");
    let content = fs::read_to_string(&llm_md_path).unwrap();

    assert!(content.contains("# Main Title"));
    assert!(content.contains("## Section 1"));
    assert!(content.contains("### Subsection 1.1"));
}

#[test]
fn test_generate_extracts_metadata() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("meta.html");

    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>My Page</title>
    <meta name="description" content="Page description">
    <meta name="author" content="John Doe">
</head>
<body>
    <h1>Content</h1>
</body>
</html>
    "#;

    fs::write(&html_path, html).unwrap();

    run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    let llm_md_path = temp_dir.path().join("meta.llm.md");
    assert!(llm_md_path.exists());
}

#[test]
fn test_generate_custom_output_directory() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("page.html");
    let output_dir = temp_dir.path().join("machine-views");

    fs::write(&html_path, create_test_html_page()).unwrap();
    fs::create_dir(&output_dir).unwrap();

    run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
        ],
        None,
    );

    assert!(output_dir.join("page.llm.md").exists());
}
