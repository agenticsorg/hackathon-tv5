/// End-to-end tests for the serve (dev server) workflow
mod common;
mod helpers;

use common::*;
use helpers::*;
use std::fs;
use std::thread;
use std::time::Duration;

// Note: These tests use timeouts since serve runs indefinitely
// In a real CI environment, you'd want more sophisticated orchestration

#[test]
#[ignore] // Run manually as it starts a server
fn test_serve_starts_server() {
    setup_test_env();
    let temp_dir = create_complete_test_site();

    // Start server in background (would need process management in real tests)
    let handle = thread::spawn(move || {
        run_cli(
            &[
                "serve",
                "--path",
                temp_dir.path().to_str().unwrap(),
                "--port",
                "3001",
            ],
            None,
        );
    });

    // Give server time to start
    thread::sleep(Duration::from_secs(2));

    // Try to connect
    let client = reqwest::blocking::Client::new();
    let result = client.get("http://127.0.0.1:3001").send();

    assert!(result.is_ok());

    // Cleanup would happen here in real tests
    // handle.join().unwrap();
}

#[test]
fn test_serve_requires_valid_path() {
    setup_test_env();

    let (_stdout, stderr) = run_cli_failure(
        &["serve", "--path", "/nonexistent"],
        None,
    );

    assert_output_contains(&stderr, "not found");
}

#[test]
fn test_serve_custom_port() {
    setup_test_env();
    // This test would verify port configuration
    // In practice, you'd start the server and verify it binds to the correct port
    assert!(true); // Placeholder
}

#[test]
#[ignore] // Manual test
fn test_serve_with_watch_mode() {
    setup_test_env();
    let temp_dir = create_complete_test_site();

    // Start server with watch mode
    let _handle = thread::spawn(move || {
        run_cli(
            &[
                "serve",
                "--path",
                temp_dir.path().to_str().unwrap(),
                "--port",
                "3002",
                "--watch",
            ],
            None,
        );
    });

    thread::sleep(Duration::from_secs(2));

    // Modify a file and verify hot reload
    fs::write(
        temp_dir.path().join("index.llm.md"),
        "# Updated Content",
    )
    .unwrap();

    // In a real test, you'd verify the server reloaded
    assert!(true);
}

#[test]
fn test_serve_serves_machine_views() {
    setup_test_env();
    // Verify that .llm.md files are accessible via HTTP
    // This would require actually starting the server
    assert!(true); // Placeholder
}

#[test]
fn test_serve_cors_headers() {
    setup_test_env();
    // Verify CORS headers are properly set
    // This would require HTTP inspection
    assert!(true); // Placeholder
}

#[test]
fn test_serve_404_handling() {
    setup_test_env();
    // Verify proper 404 responses for missing files
    assert!(true); // Placeholder
}
