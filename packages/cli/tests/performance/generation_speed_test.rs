/// Performance benchmarks for generation speed
use std::fs;
use std::time::Instant;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_generate_single_file_performance() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("test.html");

    fs::write(&html_path, create_test_html_page()).unwrap();

    let start = Instant::now();
    run_cli_success(&["generate", html_path.to_str().unwrap()], None);
    let duration = start.elapsed();

    // Single file generation should be very fast
    assert!(
        duration.as_secs() < 1,
        "Single file generation took too long: {:?}",
        duration
    );

    println!("✓ Generated machine view in {:?}", duration);
}

#[test]
fn test_generate_multiple_files_performance() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Create 50 HTML files
    for i in 0..50 {
        fs::write(
            temp_dir.path().join(format!("page{}.html", i)),
            create_test_html_page(),
        )
        .unwrap();
    }

    let start = Instant::now();
    run_cli_success(
        &[
            "generate",
            temp_dir.path().to_str().unwrap(),
            "--recursive",
        ],
        None,
    );
    let duration = start.elapsed();

    // 50 files should generate in reasonable time
    assert!(
        duration.as_secs() < 10,
        "50 file generation took too long: {:?}",
        duration
    );

    println!("✓ Generated 50 machine views in {:?}", duration);
}

#[test]
fn test_generate_large_html_performance() {
    setup_test_env();
    let temp_dir = create_temp_dir();
    let html_path = temp_dir.path().join("large.html");

    // Create large HTML file (100 sections)
    let mut sections = vec!["<!DOCTYPE html><html><body>".to_string()];
    for i in 0..100 {
        sections.push(format!("<h2>Section {}</h2><p>Content for section {}.</p>", i, i));
    }
    sections.push("</body></html>".to_string());

    fs::write(&html_path, sections.join("\n")).unwrap();

    let start = Instant::now();
    run_cli_success(&["generate", html_path.to_str().unwrap()], None);
    let duration = start.elapsed();

    assert!(
        duration.as_secs() < 5,
        "Large HTML generation took too long: {:?}",
        duration
    );

    println!("✓ Generated large HTML (100 sections) in {:?}", duration);
}

#[test]
fn test_build_command_performance() {
    setup_test_env();
    let temp_dir = create_test_site(&create_complete_llms_txt());

    let start = Instant::now();
    run_cli_success(&["build", "--source", temp_dir.path().to_str().unwrap()], None);
    let duration = start.elapsed();

    // Full build should be fast
    assert!(
        duration.as_secs() < 5,
        "Build took too long: {:?}",
        duration
    );

    println!("✓ Complete build in {:?}", duration);
}
