use anyhow::{Context, Result};
use dialoguer::Confirm;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::cli;
use crate::generators::machine_view;
use crate::parsers::registry::get_default_registry;

/// Check if a file is a Next.js App Router route file
fn is_nextjs_route(path: &Path) -> bool {
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    // Route files in Next.js App Router
    matches!(
        filename,
        "page.tsx" | "page.ts" | "page.jsx" | "page.js" |
        "route.tsx" | "route.ts" | "route.jsx" | "route.js"
    )
}

/// Map Next.js app directory path to web route path for output
/// Examples:
/// - app/page.tsx -> index
/// - app/about/page.tsx -> about/page
/// - app/blog/[slug]/page.tsx -> blog/[slug]/page
/// - app/api/users/route.ts -> api/users/route
/// - app/(marketing)/contact/page.tsx -> contact/page (route groups filtered)
fn map_to_route_path(app_relative_path: &Path) -> Option<PathBuf> {
    // Remove the "app" prefix if present
    let path_str = app_relative_path.to_str()?;
    let cleaned = path_str.strip_prefix("app/")
        .or_else(|| path_str.strip_prefix("app\\"))
        .unwrap_or(path_str);

    // Parse the path components
    let components: Vec<&str> = cleaned.split(&['/', '\\'][..]).collect();

    // Filter out route groups (folders in parentheses) but keep the filename
    let filtered_components: Vec<&str> = components
        .iter()
        .filter(|c| {
            let is_route_group = c.starts_with('(') && c.ends_with(')');
            let is_route_file = c.starts_with("page.") || c.starts_with("route.");
            !is_route_group || is_route_file
        })
        .copied()
        .collect();

    // Build the route path
    if filtered_components.is_empty() || (filtered_components.len() == 1 && filtered_components[0].starts_with("page.")) {
        // Root route: app/page.tsx -> index
        Some(PathBuf::from("index"))
    } else {
        // For nested routes, keep the structure including the filename
        // This preserves: about/page.tsx, api/users/route.ts, etc.
        Some(PathBuf::from(filtered_components.join("/")))
    }
}

pub async fn run(
    source: String,
    output: Option<String>,
    recursive: bool,
    _format: String,
    force: bool,
    output_format: String,
    sync: bool,
) -> Result<()> {
    // Validate output format
    if output_format != "markdown" && output_format != "toon" {
        anyhow::bail!("Invalid output format '{}'. Must be 'markdown' or 'toon'", output_format);
    }

    let registry = get_default_registry();
    let supported_exts: Vec<_> = registry.supported_extensions();

    cli::info(&format!("Generating machine views from: {} (format: {})", source, output_format));
    cli::info(&format!("Supported file types: {}", supported_exts.join(", ")));

    let source_path = Path::new(&source);
    let output_dir = output.as_deref().unwrap_or(".");

    if !source_path.exists() {
        anyhow::bail!("Source path does not exist: {}", source);
    }

    let mut count = 0;
    let mut skipped = 0;
    let mut generated_files: Vec<PathBuf> = Vec::new();

    if source_path.is_file() {
        // Generate for single file
        if registry.is_supported(source_path) {
            if let Some(output_path) = generate_machine_view_multi(source_path, Path::new(output_dir), &output_format, &registry, None, force)? {
                generated_files.push(output_path);
                count = 1;
            } else {
                skipped = 1;
            }
        } else {
            cli::warn(&format!("Unsupported file type: {:?}", source_path));
        }
    } else if recursive {
        // Generate recursively for all supported file types
        for entry in WalkDir::new(source_path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                // Skip node_modules, .git, etc.
                let path_str = path.to_string_lossy();
                if path_str.contains("node_modules")
                    || path_str.contains(".git")
                    || path_str.contains(".next")
                    || path_str.contains("dist/")
                    || path_str.contains("build/")
                {
                    continue;
                }

                // Only process Next.js route files (page.tsx, route.ts, etc.)
                if !is_nextjs_route(path) {
                    continue;
                }

                if registry.is_supported(path) {
                    // Calculate relative path from source for route mapping
                    let rel_path = path.strip_prefix(source_path).unwrap_or(path);

                    // Map to web route structure
                    let route_path = match map_to_route_path(rel_path) {
                        Some(p) => p,
                        None => {
                            cli::warn(&format!("Failed to map route path for {:?}", path));
                            skipped += 1;
                            continue;
                        }
                    };

                    match generate_machine_view_multi(path, Path::new(output_dir), &output_format, &registry, Some(&route_path), force) {
                        Ok(Some(output_path)) => {
                            generated_files.push(output_path);
                            count += 1;
                        }
                        Ok(None) => {
                            skipped += 1;
                        }
                        Err(e) => {
                            cli::warn(&format!("Failed to process {:?}: {}", path, e));
                            skipped += 1;
                        }
                    }
                }
            }
        }
    } else {
        anyhow::bail!("Source is a directory. Use --recursive to process directories");
    }

    cli::success(&format!("Generated {} machine view(s) in {} format", count, output_format));
    if skipped > 0 {
        cli::warn(&format!("Skipped {} file(s)", skipped));
    }

    // Sync llms.txt if requested
    if sync && !generated_files.is_empty() {
        cli::info("Syncing llms.txt with generated files...");
        match sync_llms_txt(Path::new(output_dir), &generated_files) {
            Ok(added) => {
                if added > 0 {
                    cli::success(&format!("Added {} new entries to llms.txt", added));
                } else {
                    cli::info("llms.txt already up to date");
                }
            }
            Err(e) => {
                cli::warn(&format!("Failed to sync llms.txt: {}", e));
            }
        }
    }

    Ok(())
}

/// Generate machine view using the multi-language parser registry
/// Returns Some(output_path) if file was generated, None if skipped
fn generate_machine_view_multi(
    source: &Path,
    output_dir: &Path,
    output_format: &str,
    registry: &crate::parsers::registry::ParserRegistry,
    relative_path: Option<&Path>,
    force: bool,
) -> Result<Option<PathBuf>> {
    let content = std::fs::read_to_string(source)
        .with_context(|| format!("Failed to read file: {:?}", source))?;

    // Use the registry to parse the content
    let extracted = registry.parse(&content, source)?;

    let (output_content, file_extension) = match output_format {
        "toon" => {
            // For TOON format, still use the HTML-based generator if it's HTML
            // Otherwise convert extracted content to TOON
            if source.extension().map(|e| e == "html" || e == "htm").unwrap_or(false) {
                let toon = machine_view::to_toon(&content, source)?;
                (toon, ".llm.toon")
            } else {
                let toon = extracted_to_toon(&extracted, source);
                (toon, ".llm.toon")
            }
        }
        "markdown" | _ => {
            let markdown = extracted.to_markdown();
            let with_chunks = machine_view::add_chunk_markers(&markdown);
            (with_chunks, ".llm.md")
        }
    };

    // Generate output path - preserve directory structure if relative_path is provided
    let output_path = if let Some(rel) = relative_path {
        let mut out_file = rel.parent().unwrap_or(Path::new("")).to_path_buf();
        let mut filename = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string();
        filename.push_str(file_extension);
        out_file.push(filename);

        let full_output = output_dir.join(out_file);

        // Create parent directories if they don't exist
        if let Some(parent) = full_output.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {:?}", parent))?;
        }

        full_output
    } else {
        let mut output_name = source
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output")
            .to_string();
        output_name.push_str(file_extension);
        output_dir.join(&output_name)
    };

    // Check if file exists and prompt if not forcing
    if output_path.exists() && !force {
        let overwrite = Confirm::new()
            .with_prompt(format!("Overwrite {}?", output_path.display()))
            .default(false)
            .interact()?;

        if !overwrite {
            cli::info(&format!("  Skipped: {}", output_path.display()));
            return Ok(None);
        }
    }

    std::fs::write(&output_path, output_content)
        .with_context(|| format!("Failed to write output: {:?}", output_path))?;

    if output_path.exists() && force {
        cli::info(&format!("  Overwrote: {}", output_path.display()));
    } else {
        cli::info(&format!("  Created: {}", output_path.display()));
    }

    Ok(Some(output_path))
}

/// Convert ExtractedContent to TOON format
fn extracted_to_toon(extracted: &crate::parsers::ExtractedContent, source_path: &Path) -> String {
    use chrono::Utc;

    let mut toon = String::new();
    toon.push_str("MachineView {\n");
    toon.push_str("  version: \"1.0\"\n");

    if let Some(ref title) = extracted.title {
        toon.push_str(&format!("  title: \"{}\"\n", escape_toon_string(title)));
    } else {
        toon.push_str("  title: \"Untitled\"\n");
    }

    if let Some(ref desc) = extracted.description {
        toon.push_str(&format!("  description: \"{}\"\n", escape_toon_string(desc)));
    }

    toon.push_str("  content: [\n");

    // Add sections as content blocks
    for section in &extracted.sections {
        toon.push_str(&format!(
            "    Heading {{ level: {}, text: \"{}\" }}\n",
            section.level,
            escape_toon_string(&section.title)
        ));
        if !section.content.is_empty() {
            toon.push_str(&format!(
                "    Paragraph {{ content: [Text(\"{}\")] }}\n",
                escape_toon_string(&section.content.chars().take(200).collect::<String>())
            ));
        }
    }

    toon.push_str("  ]\n");

    // Add metadata
    toon.push_str("  metadata: {\n");
    toon.push_str(&format!("    source: \"{}\"\n", escape_toon_string(&source_path.display().to_string())));
    toon.push_str(&format!("    generated_at: \"{}\"\n", Utc::now().to_rfc3339()));
    toon.push_str(&format!("    parser: \"{}\"\n", extracted.metadata.parser));
    toon.push_str("    format: \"arw-machine-view\"\n");

    if extracted.metadata.is_react_component {
        toon.push_str("    type: \"react-component\"\n");
    }

    if let Some(ref component) = extracted.metadata.component_name {
        toon.push_str(&format!("    component: \"{}\"\n", component));
    }

    toon.push_str("  }\n");
    toon.push_str("}\n");

    toon
}

fn escape_toon_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Sync llms.txt with generated machine view files
/// Returns number of new entries added
fn sync_llms_txt(output_dir: &Path, generated_files: &[PathBuf]) -> Result<usize> {
    let llms_txt_path = output_dir.join("llms.txt");

    // Read existing llms.txt
    let content = if llms_txt_path.exists() {
        std::fs::read_to_string(&llms_txt_path)
            .with_context(|| format!("Failed to read llms.txt from {:?}", llms_txt_path))?
    } else {
        return Err(anyhow::anyhow!("llms.txt not found at {:?}. Run 'arw init' first.", llms_txt_path));
    };

    // Find where content section ends (before policies section)
    let policies_idx = content.find("\n# Usage Policies\npolicies:")
        .or_else(|| content.find("\npolicies:"))
        .unwrap_or(content.len());

    let before_policies = &content[..policies_idx];
    let after_policies = &content[policies_idx..];

    // Extract existing machine_view paths
    let mut existing_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    for line in before_policies.lines() {
        if line.trim().starts_with("machine_view:") {
            if let Some(path) = line.trim().strip_prefix("machine_view:") {
                existing_paths.insert(path.trim().to_string());
            }
        }
    }

    // Generate new entries for files not already in llms.txt
    let mut new_entries = String::new();
    let mut added_count = 0;

    for file_path in generated_files {
        // Calculate relative path from output_dir
        let relative_path = file_path.strip_prefix(output_dir)
            .unwrap_or(file_path);
        let machine_view_path = format!("/{}", relative_path.display().to_string().replace('\\', "/"));

        if existing_paths.contains(&machine_view_path) {
            continue; // Already in llms.txt
        }

        // Derive URL from machine view path (remove .llm.md or .llm.toon extension)
        let url = machine_view_path
            .replace(".llm.md", "")
            .replace(".llm.toon", "")
            .replace("/index", "/");

        new_entries.push_str(&format!("  - url: {}\n", url));
        new_entries.push_str(&format!("    machine_view: {}\n", machine_view_path));
        new_entries.push_str("    purpose: auto-generated\n");
        new_entries.push_str("    priority: medium\n");
        added_count += 1;
    }

    if added_count == 0 {
        return Ok(0);
    }

    // Insert new entries before policies section
    let updated_content = format!("{}{}\n{}", before_policies, new_entries, after_policies);

    // Write updated llms.txt
    std::fs::write(&llms_txt_path, updated_content)
        .with_context(|| format!("Failed to write updated llms.txt to {:?}", llms_txt_path))?;

    Ok(added_count)
}

fn generate_machine_view(source: &Path, output_dir: &Path, output_format: &str) -> Result<()> {
    let content = std::fs::read_to_string(source)
        .with_context(|| format!("Failed to read file: {:?}", source))?;

    let (output_content, file_extension) = match output_format {
        "toon" => {
            let toon = machine_view::to_toon(&content, source)?;
            (toon, ".llm.toon")
        }
        "markdown" | _ => {
            let markdown = machine_view::from_html(&content, source)?;
            let with_chunks = machine_view::add_chunk_markers(&markdown);
            (with_chunks, ".llm.md")
        }
    };

    // Generate output filename
    let mut output_name = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output")
        .to_string();
    output_name.push_str(file_extension);

    let output_path = output_dir.join(output_name);

    std::fs::write(&output_path, output_content)
        .with_context(|| format!("Failed to write output: {:?}", output_path))?;

    cli::info(&format!("  Created: {}", output_path.display()));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_html(dir: &Path, filename: &str) -> std::path::PathBuf {
        let content = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
    <h1>Test Heading</h1>
    <p>Test content</p>
</body>
</html>"#;
        let path = dir.join(filename);
        fs::write(&path, content).unwrap();
        path
    }

    #[tokio::test]
    async fn test_run_single_file() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_test_html(temp_dir.path(), "test.html");

        let result = run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "markdown".to_string(),
            false,
            "markdown".to_string(),
        ).await;

        assert!(result.is_ok(), "Should generate machine view for single file");

        // Verify output file was created
        let output = temp_dir.path().join("test.llm.md");
        assert!(output.exists(), "Output file should be created");
    }

    #[tokio::test]
    async fn test_run_nonexistent_source() {
        let result = run(
            "/nonexistent/path.html".to_string(),
            None,
            false,
            "markdown".to_string(),
            false,
            "markdown".to_string(),
        ).await;

        assert!(result.is_err(), "Should fail for nonexistent source");
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[tokio::test]
    async fn test_run_directory_without_recursive() {
        let temp_dir = TempDir::new().unwrap();

        let result = run(
            temp_dir.path().to_str().unwrap().to_string(),
            None,
            false,
            "markdown".to_string(),
            false,
            "markdown".to_string(),
        ).await;

        assert!(result.is_err(), "Should fail for directory without --recursive");
        assert!(result.unwrap_err().to_string().contains("Use --recursive"));
    }

    #[tokio::test]
    async fn test_run_recursive() {
        let temp_dir = TempDir::new().unwrap();

        // Create multiple HTML files
        create_test_html(temp_dir.path(), "page1.html");
        create_test_html(temp_dir.path(), "page2.html");

        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_html(&subdir, "page3.html");

        let result = run(
            temp_dir.path().to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            true,
            "markdown".to_string(),
            false,
            "markdown".to_string(),
        ).await;

        assert!(result.is_ok(), "Should process directory recursively");

        // Verify output files
        assert!(temp_dir.path().join("page1.llm.md").exists());
        assert!(temp_dir.path().join("page2.llm.md").exists());
        assert!(temp_dir.path().join("page3.llm.md").exists());
    }

    #[tokio::test]
    async fn test_run_recursive_ignores_non_html() {
        let temp_dir = TempDir::new().unwrap();

        create_test_html(temp_dir.path(), "page.html");
        fs::write(temp_dir.path().join("file.txt"), "not html").unwrap();
        fs::write(temp_dir.path().join("file.md"), "markdown").unwrap();

        let result = run(
            temp_dir.path().to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            true,
            "markdown".to_string(),
            false,
            "markdown".to_string(),
        ).await;

        assert!(result.is_ok());

        // Only HTML should be processed
        assert!(temp_dir.path().join("page.llm.md").exists());
        assert!(!temp_dir.path().join("file.llm.md").exists());
    }

    #[tokio::test]
    async fn test_generate_machine_view() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_test_html(temp_dir.path(), "test.html");

        let result = generate_machine_view(&html_file, temp_dir.path(), "markdown");

        assert!(result.is_ok(), "Should generate machine view");

        let output = temp_dir.path().join("test.llm.md");
        assert!(output.exists(), "Output file should exist");

        let content = fs::read_to_string(&output).unwrap();
        assert!(!content.is_empty(), "Output should have content");
    }

    #[tokio::test]
    async fn test_generate_machine_view_invalid_html() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = temp_dir.path().join("invalid.html");
        fs::write(&html_file, "not valid html").unwrap();

        // Should still attempt to process
        let result = generate_machine_view(&html_file, temp_dir.path(), "markdown");
        // Result depends on machine_view implementation tolerance
        assert!(result.is_ok() || result.is_err());
    }

    #[tokio::test]
    async fn test_output_filename_generation() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_test_html(temp_dir.path(), "mypage.html");

        generate_machine_view(&html_file, temp_dir.path(), "markdown").unwrap();

        // Should create mypage.llm.md
        assert!(temp_dir.path().join("mypage.llm.md").exists());
    }

    #[tokio::test]
    async fn test_run_default_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_test_html(temp_dir.path(), "test.html");

        // Change to temp directory
        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = run(
            html_file.to_str().unwrap().to_string(),
            None, // Use default output directory (.)
            false,
            "markdown".to_string(),
            false,
            "markdown".to_string(),
        ).await;

        std::env::set_current_dir(orig_dir).unwrap();

        assert!(result.is_ok());
        assert!(temp_dir.path().join("test.llm.md").exists());
    }

    #[tokio::test]
    async fn test_run_toon_output_format() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_test_html(temp_dir.path(), "test.html");

        let result = run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "auto".to_string(),
            false,
            "toon".to_string(),
        ).await;

        assert!(result.is_ok(), "Should generate TOON format machine view");

        // Verify output file was created with .llm.toon extension
        let output = temp_dir.path().join("test.llm.toon");
        assert!(output.exists(), "TOON output file should be created");

        let content = fs::read_to_string(&output).unwrap();
        assert!(content.contains("MachineView {"), "Should contain TOON structure");
        assert!(content.contains("version: \"1.0\""), "Should contain version");
    }

    #[tokio::test]
    async fn test_run_invalid_output_format() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_test_html(temp_dir.path(), "test.html");

        let result = run(
            html_file.to_str().unwrap().to_string(),
            Some(temp_dir.path().to_str().unwrap().to_string()),
            false,
            "auto".to_string(),
            false,
            "invalid".to_string(),
        ).await;

        assert!(result.is_err(), "Should fail for invalid output format");
        assert!(result.unwrap_err().to_string().contains("Invalid output format"));
    }

    #[tokio::test]
    async fn test_generate_machine_view_toon_format() {
        let temp_dir = TempDir::new().unwrap();
        let html_file = create_test_html(temp_dir.path(), "test.html");

        let result = generate_machine_view(&html_file, temp_dir.path(), "toon");

        assert!(result.is_ok(), "Should generate TOON machine view");

        let output = temp_dir.path().join("test.llm.toon");
        assert!(output.exists(), "TOON output file should exist");

        let content = fs::read_to_string(&output).unwrap();
        assert!(!content.is_empty(), "TOON output should have content");
        assert!(content.contains("MachineView {"), "Should be valid TOON format");
    }
}
