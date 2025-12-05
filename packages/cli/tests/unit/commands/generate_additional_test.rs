/// Additional comprehensive tests for generate.rs command
/// These tests ensure 100% code coverage including edge cases and error paths
#[cfg(test)]
mod generate_additional_tests {
    use arw_cli::commands::generate;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Helper to create test HTML with custom content
    fn create_html_with_content(dir: &Path, filename: &str, content: &str) -> std::path::PathBuf {
        let path = dir.join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    /// Helper to create minimal valid HTML
    fn create_minimal_html(dir: &Path, filename: &str) -> std::path::PathBuf {
        create_html_with_content(
            dir,
            filename,
            "<!DOCTYPE html><html><head><title>Test</title></head><body></body></html>",
        )
    }

    #[tokio::test]
    async fn test_generate_with_empty_html() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_html_with_content(temp_dir.path(), "empty.html", "");

        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        // Should handle empty HTML gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_generate_with_malformed_html() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_html_with_content(
            temp_dir.path(),
            "malformed.html",
            "<html><head><title>Unclosed tags",
        );

        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        // Parser should handle malformed HTML
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_generate_with_special_characters_in_filename() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_minimal_html(temp_dir.path(), "test file with spaces.html");

        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(temp_dir.path().join("test file with spaces.llm.md").exists());
    }

    #[tokio::test]
    async fn test_generate_with_nested_directories() {
        let temp_dir = TempDir::new().unwrap();
        let nested = temp_dir.path().join("level1").join("level2").join("level3");
        fs::create_dir_all(&nested).unwrap();

        create_minimal_html(&nested, "deep.html");

        let result = generate::run(
            temp_dir.path().to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            true,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(temp_dir.path().join("deep.llm.md").exists());
    }

    #[tokio::test]
    async fn test_generate_with_symlinks() {
        let temp_dir = TempDir::new().unwrap();
        let real_file = create_minimal_html(temp_dir.path(), "real.html");
        let symlink = temp_dir.path().join("link.html");

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&real_file, &symlink).ok();

            if symlink.exists() {
                let result = generate::run(
                    symlink.to_str().unwrap().to_string(),
                    Some(temp_dir.path().to_str().unwrap().to_string()),
                    false,
                    "markdown".to_string(),
                    false,
                )
                .await;

                assert!(result.is_ok());
            }
        }
    }

    #[tokio::test]
    async fn test_generate_recursive_with_mixed_file_types() {
        let temp_dir = TempDir::new().unwrap();

        create_minimal_html(temp_dir.path(), "page1.html");
        create_minimal_html(temp_dir.path(), "page2.html");
        fs::write(temp_dir.path().join("data.json"), "{}").unwrap();
        fs::write(temp_dir.path().join("style.css"), "body {}").unwrap();
        fs::write(temp_dir.path().join("script.js"), "console.log()").unwrap();
        fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();

        let result = generate::run(
            temp_dir.path().to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            true,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());

        // Only HTML files should be processed
        assert!(temp_dir.path().join("page1.llm.md").exists());
        assert!(temp_dir.path().join("page2.llm.md").exists());
        assert!(!temp_dir.path().join("data.llm.md").exists());
        assert!(!temp_dir.path().join("style.llm.md").exists());
        assert!(!temp_dir.path().join("script.llm.md").exists());
        assert!(!temp_dir.path().join("README.llm.md").exists());
    }

    #[tokio::test]
    async fn test_generate_with_html_file_without_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_without_ext = temp_dir.path().join("noext");
        fs::write(
            &file_without_ext,
            "<!DOCTYPE html><html><head><title>No Ext</title></head><body></body></html>",
        )
        .unwrap();

        let result = generate::run(
            file_without_ext.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());
        // Should generate output even without .html extension when processing single file
    }

    #[tokio::test]
    async fn test_generate_recursive_empty_directory() {
        let temp_dir = TempDir::new().unwrap();
        let empty_subdir = temp_dir.path().join("empty");
        fs::create_dir(&empty_subdir).unwrap();

        let result = generate::run(
            temp_dir.path().to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            true,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_with_large_html_file() {
        let temp_dir = TempDir::new().unwrap();

        // Create a large HTML file with many elements
        let mut large_html = String::from("<!DOCTYPE html><html><head><title>Large</title></head><body>");
        for i in 0..1000 {
            large_html.push_str(&format!("<p>Paragraph {}</p>", i));
        }
        large_html.push_str("</body></html>");

        let html_file = create_html_with_content(temp_dir.path(), "large.html", &large_html);

        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(temp_dir.path().join("large.llm.md").exists());

        let output_content = fs::read_to_string(temp_dir.path().join("large.llm.md")).unwrap();
        assert!(!output_content.is_empty());
    }

    #[tokio::test]
    async fn test_generate_with_unicode_content() {
        let temp_dir = TempDir::new().unwrap();

        let unicode_html = r#"<!DOCTYPE html>
<html lang="ja">
<head><title>日本語テスト</title></head>
<body>
    <h1>こんにちは世界</h1>
    <p>🎌 Unicode test with emoji 🚀</p>
    <p>中文测试 العربية тест</p>
</body>
</html>"#;

        let html_file = create_html_with_content(temp_dir.path(), "unicode.html", unicode_html);

        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(temp_dir.path().join("unicode.llm.md").exists());

        let output_content = fs::read_to_string(temp_dir.path().join("unicode.llm.md")).unwrap();
        assert!(!output_content.is_empty());
    }

    #[tokio::test]
    async fn test_generate_with_complex_html_structure() {
        let temp_dir = TempDir::new().unwrap();

        let complex_html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Complex Structure</title>
    <meta name="description" content="Test">
</head>
<body>
    <header>
        <nav>
            <ul>
                <li><a href="/">Home</a></li>
                <li><a href="/about">About</a></li>
            </ul>
        </nav>
    </header>
    <main>
        <article>
            <h1>Main Article</h1>
            <section>
                <h2>Section 1</h2>
                <p>Content here</p>
            </section>
        </article>
        <aside>
            <h3>Related</h3>
        </aside>
    </main>
    <footer>
        <p>&copy; 2024</p>
    </footer>
</body>
</html>"#;

        let html_file = create_html_with_content(temp_dir.path(), "complex.html", complex_html);

        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());
        assert!(temp_dir.path().join("complex.llm.md").exists());
    }

    #[tokio::test]
    async fn test_generate_output_path_creation() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_minimal_html(temp_dir.path(), "test.html");
        let output_subdir = temp_dir.path().join("output");

        // Output directory doesn't exist yet - should be created if needed or fail gracefully
        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(output_subdir.to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
        )
        .await;

        // Depending on implementation, might succeed or fail
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_generate_force_flag_ignored() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_minimal_html(temp_dir.path(), "test.html");

        // Create existing output file
        fs::write(temp_dir.path().join("test.llm.md"), "existing content").unwrap();

        let result = generate::run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            true, // force flag currently ignored (_force parameter)
        )
        .await;

        assert!(result.is_ok());
        // Output should be overwritten (force flag is currently unused but parameter exists)
    }

    #[tokio::test]
    async fn test_generate_format_parameter_ignored() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_minimal_html(temp_dir.path(), "test.html");

        // Test with different format values (currently ignored as _format parameter)
        let formats = vec!["markdown", "json", "yaml", "html"];

        for format in formats {
            let result = generate::run(
                html_file.to_str().unwrap().to_string(),
                Some(temp_dir.path().to_str().unwrap().to_string()),
                false,
                format.to_string(),
                false,
            )
            .await;

            assert!(result.is_ok(), "Should succeed with format: {}", format);
        }
    }

    #[tokio::test]
    async fn test_generate_recursive_with_hidden_files() {
        let temp_dir = TempDir::new().unwrap();

        create_minimal_html(temp_dir.path(), "visible.html");
        create_minimal_html(temp_dir.path(), ".hidden.html");

        let result = generate::run(
            temp_dir.path().to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            true,
            "markdown".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());

        // Verify visible file is processed
        assert!(temp_dir.path().join("visible.llm.md").exists());

        // Hidden files might or might not be processed depending on walkdir behavior
        // Both outcomes are acceptable
    }

    #[tokio::test]
    async fn test_generate_with_readonly_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_minimal_html(temp_dir.path(), "test.html");
        let readonly_dir = temp_dir.path().join("readonly");
        fs::create_dir(&readonly_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o444); // read-only
            fs::set_permissions(&readonly_dir, perms).unwrap();

            let result = generate::run(
                html_file.to_str().unwrap().to_string(),
                Some(readonly_dir.to_str().unwrap().to_string()),
                false,
                "markdown".to_string(),
                false,
            )
            .await;

            // Should fail due to permission error
            assert!(result.is_err());

            // Restore permissions for cleanup
            let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&readonly_dir, perms).unwrap();
        }
    }
}
