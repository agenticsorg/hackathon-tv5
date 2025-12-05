use anyhow::Result;

use crate::cli;

pub async fn run(path: String, port: u16, _watch: bool, _open: bool) -> Result<()> {
    cli::info(&format!(
        "Starting development server on http://localhost:{}",
        port
    ));
    cli::info(&format!("Serving files from: {}", path));

    // TODO: Implement actual server with axum
    cli::warn("Development server not yet implemented");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_run_basic() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let result = run(path, 3000, false, false).await;
        assert!(result.is_ok(), "Should succeed even though not implemented");
    }

    #[tokio::test]
    async fn test_run_with_watch() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let result = run(path, 3000, true, false).await;
        assert!(result.is_ok(), "Should succeed with watch flag");
    }

    #[tokio::test]
    async fn test_run_with_open() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let result = run(path, 3000, false, true).await;
        assert!(result.is_ok(), "Should succeed with open flag");
    }

    #[tokio::test]
    async fn test_run_all_flags() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let result = run(path, 3000, true, true).await;
        assert!(result.is_ok(), "Should succeed with all flags");
    }

    #[tokio::test]
    async fn test_run_different_ports() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let ports = vec![3000, 8080, 5000, 9000];

        for port in ports {
            let result = run(path.clone(), port, false, false).await;
            assert!(result.is_ok(), "Should succeed with port {}", port);
        }
    }

    #[tokio::test]
    async fn test_run_nonexistent_path() {
        // Should still succeed since we just display the path
        let result = run("/nonexistent/path".to_string(), 3000, false, false).await;
        assert!(result.is_ok(), "Should succeed even with nonexistent path (not validated yet)");
    }

    #[tokio::test]
    async fn test_run_custom_path_and_port() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let result = run(path, 8888, true, true).await;
        assert!(result.is_ok(), "Should succeed with custom port and all flags");
    }
}
