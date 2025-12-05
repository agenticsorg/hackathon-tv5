/// Common test setup and utilities for E2E tests
use std::process::Command;
use tempfile::TempDir;

/// Run the ARW CLI with specified arguments
pub fn run_cli(args: &[&str], work_dir: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_arw"));
    cmd.args(args);

    if let Some(dir) = work_dir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().expect("Failed to execute command");

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    (exit_code, stdout, stderr)
}

/// Run CLI command and expect success
pub fn run_cli_success(args: &[&str], work_dir: Option<&str>) -> String {
    let (exit_code, stdout, stderr) = run_cli(args, work_dir);
    assert_eq!(
        exit_code, 0,
        "Command failed with exit code {}\nStdout:\n{}\nStderr:\n{}",
        exit_code, stdout, stderr
    );
    stdout
}

/// Run CLI command and expect failure
pub fn run_cli_failure(args: &[&str], work_dir: Option<&str>) -> (String, String) {
    let (exit_code, stdout, stderr) = run_cli(args, work_dir);
    assert_ne!(
        exit_code, 0,
        "Command succeeded when it should have failed\nStdout:\n{}",
        stdout
    );
    (stdout, stderr)
}

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Setup test environment with logging
pub fn setup_test_env() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_cli_help_succeeds() {
        let (exit_code, stdout, _) = run_cli(&["--help"], None);
        assert_eq!(exit_code, 0);
        assert!(stdout.contains("ARW CLI"));
    }

    #[test]
    fn test_create_temp_dir_works() {
        let temp_dir = create_temp_dir();
        assert!(temp_dir.path().exists());
    }
}
