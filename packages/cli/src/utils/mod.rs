pub mod config;
pub mod chunking;
pub mod crawler;

use anyhow::Result;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize the logging system
pub fn init_logger(verbose: bool, quiet: bool) -> Result<()> {
    let filter = if verbose {
        EnvFilter::new("debug")
    } else if quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::new("info")
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_target(false))
        .init();

    Ok(())
}

/// Format file size for display
#[allow(dead_code)]
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_idx])
}

/// Sanitize a string for use as a filename
#[allow(dead_code)]
pub fn sanitize_filename(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Check if a path is likely a URL
#[allow(dead_code)]
pub fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0.00 B");
        assert_eq!(format_size(100), "100.00 B");
        assert_eq!(format_size(1023), "1023.00 B");
    }

    #[test]
    fn test_format_size_kilobytes() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_size_megabytes() {
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 2), "2.00 MB");
        assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    }

    #[test]
    fn test_format_size_gigabytes() {
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 * 2), "2.00 GB");
        assert_eq!(format_size(1024 * 1024 * 1024 + 512 * 1024 * 1024), "1.50 GB");
    }

    #[test]
    fn test_format_size_large() {
        // Should cap at GB even for very large values
        let large = 1024u64 * 1024 * 1024 * 1024; // 1 TB
        let result = format_size(large);
        assert!(result.contains("GB"));
    }

    #[test]
    fn test_sanitize_filename_basic() {
        assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
        assert_eq!(sanitize_filename("file name.doc"), "file name.doc");
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("file/name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("file\\name.txt"), "file_name.txt");
        assert_eq!(sanitize_filename("file:name.txt"), "file_name.txt");
    }

    #[test]
    fn test_sanitize_filename_all_invalid() {
        assert_eq!(sanitize_filename("*?<>|"), "_____");
        assert_eq!(sanitize_filename("file*?name"), "file__name");
    }

    #[test]
    fn test_sanitize_filename_quotes() {
        assert_eq!(sanitize_filename("file\"name.txt"), "file_name.txt");
    }

    #[test]
    fn test_sanitize_filename_multiple_invalid() {
        assert_eq!(sanitize_filename("file/\\:*?\"<>|name.txt"), "file_________name.txt");
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "");
    }

    #[test]
    fn test_sanitize_filename_unicode() {
        assert_eq!(sanitize_filename("文件名.txt"), "文件名.txt");
        assert_eq!(sanitize_filename("файл.doc"), "файл.doc");
    }

    #[test]
    fn test_is_url_http() {
        assert!(is_url("http://example.com"));
        assert!(is_url("http://localhost:8080"));
        assert!(is_url("http://192.168.1.1"));
    }

    #[test]
    fn test_is_url_https() {
        assert!(is_url("https://example.com"));
        assert!(is_url("https://www.example.com/path"));
        assert!(is_url("https://api.example.com/v1/resource"));
    }

    #[test]
    fn test_is_url_not_url() {
        assert!(!is_url("/path/to/file"));
        assert!(!is_url("file.txt"));
        assert!(!is_url("./relative/path"));
        assert!(!is_url("../parent/path"));
    }

    #[test]
    fn test_is_url_edge_cases() {
        assert!(!is_url(""));
        assert!(!is_url("ftp://example.com"));
        assert!(!is_url("file:///path/to/file"));
        assert!(is_url("https://"));
        assert!(is_url("http://"));
    }

    #[test]
    #[ignore] // Logger can only be initialized once per process
    fn test_init_logger_verbose() {
        // Test that verbose mode doesn't panic
        let result = init_logger(true, false);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Logger can only be initialized once per process
    fn test_init_logger_quiet() {
        // Test that quiet mode doesn't panic
        let result = init_logger(false, true);
        assert!(result.is_ok());
    }

    #[test]
    #[ignore] // Logger can only be initialized once per process
    fn test_init_logger_normal() {
        // Test that normal mode doesn't panic
        let result = init_logger(false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_size_precision() {
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1024 + 256), "1.25 KB");
        assert_eq!(format_size(1024 + 512), "1.50 KB");
        assert_eq!(format_size(1024 + 768), "1.75 KB");
    }

    #[test]
    fn test_sanitize_filename_path_separators() {
        assert_eq!(sanitize_filename("dir/subdir/file.txt"), "dir_subdir_file.txt");
        assert_eq!(sanitize_filename("C:\\Users\\file.txt"), "C__Users_file.txt");
    }

    #[test]
    fn test_is_url_with_query() {
        assert!(is_url("https://example.com?query=value"));
        assert!(is_url("http://example.com?a=1&b=2"));
    }

    #[test]
    fn test_is_url_with_fragment() {
        assert!(is_url("https://example.com#section"));
        assert!(is_url("http://example.com/page#top"));
    }
}
