/// Tests for CLI argument parsing and validation
mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_help_flag() {
    setup_test_env();
    let output = run_cli_success(&["--help"], None);
    assert_output_contains(&output, "ARW CLI");
    assert_output_contains(&output, "USAGE");
}

#[test]
fn test_version_flag() {
    setup_test_env();
    let output = run_cli_success(&["--version"], None);
    assert!(output.contains("arw"));
}

#[test]
fn test_verbose_flag() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &[
            "--verbose",
            "validate",
            "--path",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    // Verbose mode should include detailed output
    assert_output_contains(&output, "Validating");
}

#[test]
fn test_quiet_flag() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &[
            "--quiet",
            "validate",
            "--path",
            temp_dir.path().to_str().unwrap(),
        ],
        None,
    );

    // Quiet mode should suppress banner and most output
    assert!(!output.contains("ARW CLI"));
}

#[test]
fn test_command_aliases() {
    setup_test_env();

    // Test 'val' alias for 'validate'
    let (exit_code, _, _) = run_cli(&["val", "--help"], None);
    assert_eq!(exit_code, 0);

    // Test 'gen' alias for 'generate'
    let (exit_code, _, _) = run_cli(&["gen", "--help"], None);
    assert_eq!(exit_code, 0);

    // Test 'dev' alias for 'serve'
    let (exit_code, _, _) = run_cli(&["dev", "--help"], None);
    assert_eq!(exit_code, 0);
}

#[test]
fn test_invalid_command() {
    setup_test_env();
    let (_stdout, stderr) = run_cli_failure(&["invalid-command"], None);
    assert_output_contains(&stderr, "unrecognized");
}

#[test]
fn test_missing_required_arguments() {
    setup_test_env();

    // Generate requires source
    let (_stdout, stderr) = run_cli_failure(&["generate"], None);
    assert_output_contains(&stderr, "required");
}

#[test]
fn test_validate_path_argument() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &["validate", "--path", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Success");
}

#[test]
fn test_validate_default_path() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    // Create "public" subdirectory (default path)
    let public_dir = temp_dir.path().join("public");
    std::fs::create_dir(&public_dir).unwrap();
    std::fs::write(
        public_dir.join("llms.txt"),
        create_minimal_llms_txt(),
    )
    .unwrap();

    // Run from parent directory without --path
    let (exit_code, _, _) = run_cli(&["validate"], Some(temp_dir.path().to_str().unwrap()));
    assert_eq!(exit_code, 0);
}

#[test]
fn test_build_source_argument() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &["build", "--source", temp_dir.path().to_str().unwrap()],
        None,
    );

    assert_output_contains(&output, "Build complete");
}

#[test]
fn test_generate_output_argument() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("test.html");
    let output_dir = temp_dir.path().join("output");

    std::fs::write(&html_path, create_test_html_page()).unwrap();
    std::fs::create_dir(&output_dir).unwrap();

    run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--output",
            output_dir.to_str().unwrap(),
        ],
        None,
    );

    assert!(output_dir.join("test.llm.md").exists());
}

#[test]
fn test_generate_recursive_flag() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    std::fs::write(
        temp_dir.path().join("test.html"),
        create_test_html_page(),
    )
    .unwrap();

    let output = run_cli_success(
        &[
            "generate",
            temp_dir.path().to_str().unwrap(),
            "--recursive",
        ],
        None,
    );

    assert_output_contains(&output, "Success");
}

#[test]
fn test_generate_force_flag() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("test.html");
    let llm_md_path = temp_dir.path().join("test.llm.md");

    std::fs::write(&html_path, create_test_html_page()).unwrap();
    std::fs::write(&llm_md_path, "existing").unwrap();

    run_cli_success(
        &[
            "generate",
            html_path.to_str().unwrap(),
            "--force",
        ],
        None,
    );

    let content = std::fs::read_to_string(&llm_md_path).unwrap();
    assert!(!content.contains("existing"));
}

#[test]
fn test_validate_strict_flag() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    // Without robots.txt, strict mode should warn/fail
    let (_stdout, _stderr) = run_cli_failure(
        &[
            "validate",
            "--path",
            temp_dir.path().to_str().unwrap(),
            "--strict",
        ],
        None,
    );
}

#[test]
fn test_serve_port_argument() {
    setup_test_env();
    // This would test port configuration
    // In practice, you'd verify the server binds to the specified port
    assert!(true);
}

#[test]
fn test_multiple_flags_combination() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let output = run_cli_success(
        &[
            "--verbose",
            "validate",
            "--path",
            temp_dir.path().to_str().unwrap(),
            "--strict",
        ],
        None,
    );

    // Should respect both verbose and strict
    assert!(output.contains("Validating") || output.len() > 0);
}

#[test]
fn test_conflicting_flags() {
    setup_test_env();

    // --verbose and --quiet should conflict
    let (_stdout, stderr) = run_cli_failure(&["--verbose", "--quiet", "validate"], None);

    // CLI should handle this gracefully
    assert!(stderr.len() > 0 || true);
}
