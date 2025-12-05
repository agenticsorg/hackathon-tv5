/// Custom assertions for CLI output and file validation
use std::fs;
use std::path::Path;

/// Assert that a file exists and contains expected content
pub fn assert_file_contains<P: AsRef<Path>>(path: P, expected: &str) {
    let path = path.as_ref();
    assert!(
        path.exists(),
        "File does not exist: {}",
        path.display()
    );

    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e));

    assert!(
        content.contains(expected),
        "File {} does not contain expected text: '{}'\nActual content:\n{}",
        path.display(),
        expected,
        content
    );
}

/// Assert that a file is valid JSON
pub fn assert_valid_json<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(
        path.exists(),
        "File does not exist: {}",
        path.display()
    );

    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e));

    serde_json::from_str::<serde_json::Value>(&content).unwrap_or_else(|e| {
        panic!("File {} is not valid JSON: {}\nContent:\n{}", path.display(), e, content)
    });
}

/// Assert that a file is valid YAML
pub fn assert_valid_yaml<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    assert!(
        path.exists(),
        "File does not exist: {}",
        path.display()
    );

    let content = fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("Failed to read file {}: {}", path.display(), e));

    serde_yaml::from_str::<serde_yaml::Value>(&content).unwrap_or_else(|e| {
        panic!("File {} is not valid YAML: {}\nContent:\n{}", path.display(), e, content)
    });
}

/// Assert that a JSON file has a specific field with expected value
pub fn assert_json_field<P: AsRef<Path>>(path: P, field_path: &str, expected: &str) {
    let path = path.as_ref();
    let content = fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    let value = field_path.split('.').fold(&json, |acc, key| {
        &acc[key]
    });

    assert_eq!(
        value.as_str().unwrap(),
        expected,
        "JSON field {} has unexpected value",
        field_path
    );
}

/// Assert that a directory contains all expected files
pub fn assert_directory_contains<P: AsRef<Path>>(dir: P, expected_files: &[&str]) {
    let dir = dir.as_ref();
    assert!(dir.exists(), "Directory does not exist: {}", dir.display());
    assert!(dir.is_dir(), "Path is not a directory: {}", dir.display());

    for file in expected_files {
        let file_path = dir.join(file);
        assert!(
            file_path.exists(),
            "Expected file not found: {}",
            file_path.display()
        );
    }
}

/// Assert that command output contains expected text
pub fn assert_output_contains(output: &str, expected: &str) {
    assert!(
        output.contains(expected),
        "Output does not contain expected text: '{}'\nActual output:\n{}",
        expected,
        output
    );
}

/// Assert that command succeeded (exit code 0)
pub fn assert_command_success(exit_code: i32, output: &str) {
    assert_eq!(
        exit_code, 0,
        "Command failed with exit code {}\nOutput:\n{}",
        exit_code, output
    );
}

/// Assert that command failed (exit code != 0)
pub fn assert_command_failed(exit_code: i32) {
    assert_ne!(
        exit_code, 0,
        "Command succeeded when it should have failed"
    );
}

/// Assert that llms.txt and llms.json are equivalent
pub fn assert_llms_files_equivalent<P: AsRef<Path>>(base_dir: P) {
    let base_dir = base_dir.as_ref();
    let txt_path = base_dir.join("llms.txt");
    let json_path = base_dir.join("llms.json");

    assert!(txt_path.exists(), "llms.txt does not exist");
    assert!(json_path.exists(), "llms.json does not exist");

    let txt_content = fs::read_to_string(&txt_path).unwrap();
    let json_content = fs::read_to_string(&json_path).unwrap();

    let yaml_value: serde_yaml::Value = serde_yaml::from_str(&txt_content).unwrap();
    let json_value: serde_json::Value = serde_json::from_str(&json_content).unwrap();

    // Convert YAML to JSON for comparison
    let yaml_as_json = serde_json::to_value(&yaml_value).unwrap();

    assert_eq!(
        yaml_as_json, json_value,
        "llms.txt and llms.json are not equivalent"
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_assert_file_contains_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello World").unwrap();

        assert_file_contains(&file_path, "Hello");
    }

    #[test]
    #[should_panic(expected = "does not contain expected text")]
    fn test_assert_file_contains_failure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello World").unwrap();

        assert_file_contains(&file_path, "Goodbye");
    }

    #[test]
    fn test_assert_valid_json_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, r#"{"key": "value"}"#).unwrap();

        assert_valid_json(&file_path);
    }

    #[test]
    #[should_panic(expected = "is not valid JSON")]
    fn test_assert_valid_json_failure() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.json");
        fs::write(&file_path, "not json").unwrap();

        assert_valid_json(&file_path);
    }
}
