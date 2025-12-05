/// Performance benchmarks for validation speed
use std::fs;
use std::time::Instant;

mod common;
mod helpers;

use common::*;
use helpers::*;

#[test]
fn test_validate_small_manifest_performance() {
    setup_test_env();
    let temp_dir = create_test_site(&create_minimal_llms_txt());

    let start = Instant::now();
    run_cli_success(&["validate", "--path", temp_dir.path().to_str().unwrap()], None);
    let duration = start.elapsed();

    // Small manifest should validate in under 2 seconds
    assert!(
        duration.as_secs() < 2,
        "Validation took too long: {:?}",
        duration
    );
}

#[test]
fn test_validate_large_manifest_performance() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Create manifest with 100 content entries
    let mut content_entries = Vec::new();
    for i in 0..100 {
        content_entries.push(format!(
            r#"  - url: "/page{}"
    machine_view: "/page{}.llm.md"
    purpose: "content"
    priority: "medium""#,
            i, i
        ));
    }

    let manifest = format!(
        r#"version: "1.0"
profile: ARW-2
site:
  name: "Large"
  homepage: "https://example.com"
content:
{}
policies:
  training:
    allowed: false
"#,
        content_entries.join("\n")
    );

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let start = Instant::now();
    run_cli_success(&["validate", "--path", temp_dir.path().to_str().unwrap()], None);
    let duration = start.elapsed();

    // Even large manifest should validate quickly
    assert!(
        duration.as_secs() < 5,
        "Large manifest validation took too long: {:?}",
        duration
    );

    println!("✓ Validated 100 content entries in {:?}", duration);
}

#[test]
fn test_validate_with_chunks_performance() {
    setup_test_env();
    let temp_dir = create_temp_dir();

    // Create manifest with many chunks
    let manifest = r#"version: "1.0"
profile: ARW-2
site:
  name: "Test"
  homepage: "https://example.com"
content:
  - url: "/"
    machine_view: "/index.llm.md"
    purpose: "homepage"
    chunks:
      - id: "chunk1"
        heading: "Section 1"
      - id: "chunk2"
        heading: "Section 2"
      - id: "chunk3"
        heading: "Section 3"
      - id: "chunk4"
        heading: "Section 4"
      - id: "chunk5"
        heading: "Section 5"
      - id: "chunk6"
        heading: "Section 6"
      - id: "chunk7"
        heading: "Section 7"
      - id: "chunk8"
        heading: "Section 8"
      - id: "chunk9"
        heading: "Section 9"
      - id: "chunk10"
        heading: "Section 10"
policies:
  training:
    allowed: false
"#;

    fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

    let start = Instant::now();
    run_cli_success(&["validate", "--path", temp_dir.path().to_str().unwrap()], None);
    let duration = start.elapsed();

    assert!(
        duration.as_secs() < 3,
        "Chunk validation took too long: {:?}",
        duration
    );
}

#[test]
fn test_validation_scales_linearly() {
    setup_test_env();

    let sizes = vec![10, 50, 100];
    let mut timings = Vec::new();

    for size in sizes.iter() {
        let temp_dir = create_temp_dir();

        let mut content_entries = Vec::new();
        for i in 0..*size {
            content_entries.push(format!(
                r#"  - url: "/page{}"
    machine_view: "/page{}.llm.md"
    purpose: "content""#,
                i, i
            ));
        }

        let manifest = format!(
            r#"version: "1.0"
profile: ARW-2
site:
  name: "Test"
  homepage: "https://example.com"
content:
{}
policies:
  training:
    allowed: false
"#,
            content_entries.join("\n")
        );

        fs::write(temp_dir.path().join("llms.txt"), manifest).unwrap();

        let start = Instant::now();
        run_cli_success(&["validate", "--path", temp_dir.path().to_str().unwrap()], None);
        let duration = start.elapsed();

        timings.push((*size, duration));
        println!("✓ {} entries: {:?}", size, duration);
    }

    // Ensure reasonable scaling
    assert!(timings[2].1.as_millis() < timings[0].1.as_millis() * 20);
}
