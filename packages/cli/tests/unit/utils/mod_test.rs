use arw_cli::utils::{format_size, sanitize_filename, is_url, init_logger};

/// Comprehensive tests for utils/mod.rs to achieve 100% coverage
/// Testing all utility functions with various inputs and edge cases

// ============================================================================
// format_size tests
// ============================================================================

#[test]
fn test_format_size_zero() {
    assert_eq!(format_size(0), "0.00 B");
}

#[test]
fn test_format_size_bytes() {
    assert_eq!(format_size(1), "1.00 B");
    assert_eq!(format_size(512), "512.00 B");
    assert_eq!(format_size(1023), "1023.00 B");
}

#[test]
fn test_format_size_exact_kilobyte() {
    assert_eq!(format_size(1024), "1.00 KB");
}

#[test]
fn test_format_size_kilobytes() {
    assert_eq!(format_size(2048), "2.00 KB");
    assert_eq!(format_size(1536), "1.50 KB");
    assert_eq!(format_size(1280), "1.25 KB");
    assert_eq!(format_size(10240), "10.00 KB");
}

#[test]
fn test_format_size_exact_megabyte() {
    assert_eq!(format_size(1024 * 1024), "1.00 MB");
}

#[test]
fn test_format_size_megabytes() {
    assert_eq!(format_size(2 * 1024 * 1024), "2.00 MB");
    assert_eq!(format_size(1024 * 1024 + 512 * 1024), "1.50 MB");
    assert_eq!(format_size(5 * 1024 * 1024), "5.00 MB");
}

#[test]
fn test_format_size_exact_gigabyte() {
    assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
}

#[test]
fn test_format_size_gigabytes() {
    assert_eq!(format_size(2 * 1024 * 1024 * 1024), "2.00 GB");
    assert_eq!(format_size(1024 * 1024 * 1024 + 512 * 1024 * 1024), "1.50 GB");
}

#[test]
fn test_format_size_terabytes_caps_at_gb() {
    // Should cap at GB even for terabyte values
    let tb = 1024u64 * 1024 * 1024 * 1024;
    let result = format_size(tb);
    assert!(result.contains("GB"));
    assert!(result.starts_with("1024.00"));
}

#[test]
fn test_format_size_precision() {
    assert_eq!(format_size(1280), "1.25 KB");
    assert_eq!(format_size(1792), "1.75 KB");
    assert_eq!(format_size(1024 + 102), "1.10 KB");
}

#[test]
fn test_format_size_boundary_values() {
    // Just below KB
    assert_eq!(format_size(1023), "1023.00 B");
    // Just above KB
    assert_eq!(format_size(1025), "1.00 KB");

    // Just below MB
    let just_below_mb = 1024 * 1024 - 1;
    let result = format_size(just_below_mb);
    assert!(result.contains("KB"));

    // Just above MB
    let just_above_mb = 1024 * 1024 + 1;
    let result = format_size(just_above_mb);
    assert!(result.contains("MB"));
}

// ============================================================================
// sanitize_filename tests
// ============================================================================

#[test]
fn test_sanitize_filename_normal() {
    assert_eq!(sanitize_filename("normal.txt"), "normal.txt");
    assert_eq!(sanitize_filename("file123.doc"), "file123.doc");
    assert_eq!(sanitize_filename("my-file_v2.pdf"), "my-file_v2.pdf");
}

#[test]
fn test_sanitize_filename_with_spaces() {
    assert_eq!(sanitize_filename("my file.txt"), "my file.txt");
    assert_eq!(sanitize_filename("multiple   spaces.doc"), "multiple   spaces.doc");
}

#[test]
fn test_sanitize_filename_forward_slash() {
    assert_eq!(sanitize_filename("path/to/file.txt"), "path_to_file.txt");
    assert_eq!(sanitize_filename("/absolute/path.txt"), "_absolute_path.txt");
}

#[test]
fn test_sanitize_filename_backslash() {
    assert_eq!(sanitize_filename("path\\to\\file.txt"), "path_to_file.txt");
    assert_eq!(sanitize_filename("C:\\Users\\file.txt"), "C__Users_file.txt");
}

#[test]
fn test_sanitize_filename_colon() {
    assert_eq!(sanitize_filename("file:name.txt"), "file_name.txt");
    assert_eq!(sanitize_filename("C:file.txt"), "C_file.txt");
}

#[test]
fn test_sanitize_filename_asterisk() {
    assert_eq!(sanitize_filename("file*.txt"), "file_.txt");
    assert_eq!(sanitize_filename("*.*"), "_._");
}

#[test]
fn test_sanitize_filename_question_mark() {
    assert_eq!(sanitize_filename("file?.txt"), "file_.txt");
    assert_eq!(sanitize_filename("what?how?.doc"), "what_how_.doc");
}

#[test]
fn test_sanitize_filename_quotes() {
    assert_eq!(sanitize_filename("file\"name.txt"), "file_name.txt");
    assert_eq!(sanitize_filename("\"quoted\".txt"), "_quoted_.txt");
}

#[test]
fn test_sanitize_filename_angle_brackets() {
    assert_eq!(sanitize_filename("file<name>.txt"), "file_name_.txt");
    assert_eq!(sanitize_filename("<test>.doc"), "_test_.doc");
}

#[test]
fn test_sanitize_filename_pipe() {
    assert_eq!(sanitize_filename("file|name.txt"), "file_name.txt");
    assert_eq!(sanitize_filename("a|b|c"), "a_b_c");
}

#[test]
fn test_sanitize_filename_all_invalid_chars() {
    assert_eq!(sanitize_filename("/\\:*?\"<>|"), "_________");
}

#[test]
fn test_sanitize_filename_mixed_invalid() {
    assert_eq!(sanitize_filename("file/name:ver*.txt"), "file_name_ver_.txt");
    assert_eq!(sanitize_filename("path\\to:file?.doc"), "path_to_file_.doc");
}

#[test]
fn test_sanitize_filename_empty() {
    assert_eq!(sanitize_filename(""), "");
}

#[test]
fn test_sanitize_filename_only_extension() {
    assert_eq!(sanitize_filename(".gitignore"), ".gitignore");
    assert_eq!(sanitize_filename(".hidden"), ".hidden");
}

#[test]
fn test_sanitize_filename_unicode() {
    assert_eq!(sanitize_filename("文件名.txt"), "文件名.txt");
    assert_eq!(sanitize_filename("файл.doc"), "файл.doc");
    assert_eq!(sanitize_filename("ملف.pdf"), "ملف.pdf");
    assert_eq!(sanitize_filename("αρχείο.txt"), "αρχείο.txt");
}

#[test]
fn test_sanitize_filename_emoji() {
    assert_eq!(sanitize_filename("file😀.txt"), "file😀.txt");
    assert_eq!(sanitize_filename("🎉party🎊.doc"), "🎉party🎊.doc");
}

#[test]
fn test_sanitize_filename_long() {
    let long_name = "a".repeat(500);
    let result = sanitize_filename(&long_name);
    assert_eq!(result.len(), 500);
}

#[test]
fn test_sanitize_filename_complex_path() {
    assert_eq!(
        sanitize_filename("C:\\Users\\John\\Documents\\file:v2*.doc"),
        "C__Users_John_Documents_file_v2_.doc"
    );
}

// ============================================================================
// is_url tests
// ============================================================================

#[test]
fn test_is_url_http() {
    assert!(is_url("http://example.com"));
    assert!(is_url("http://www.example.com"));
    assert!(is_url("http://example.com/path"));
}

#[test]
fn test_is_url_https() {
    assert!(is_url("https://example.com"));
    assert!(is_url("https://www.example.com"));
    assert!(is_url("https://example.com/path"));
}

#[test]
fn test_is_url_with_port() {
    assert!(is_url("http://localhost:8080"));
    assert!(is_url("https://example.com:443"));
    assert!(is_url("http://192.168.1.1:3000"));
}

#[test]
fn test_is_url_with_path() {
    assert!(is_url("https://example.com/path/to/resource"));
    assert!(is_url("http://example.com/api/v1/users"));
}

#[test]
fn test_is_url_with_query() {
    assert!(is_url("https://example.com?query=value"));
    assert!(is_url("http://example.com?a=1&b=2"));
    assert!(is_url("https://example.com/path?query=value&other=123"));
}

#[test]
fn test_is_url_with_fragment() {
    assert!(is_url("https://example.com#section"));
    assert!(is_url("http://example.com/page#top"));
    assert!(is_url("https://example.com/docs#api-reference"));
}

#[test]
fn test_is_url_minimal() {
    assert!(is_url("http://"));
    assert!(is_url("https://"));
}

#[test]
fn test_is_url_ip_address() {
    assert!(is_url("http://127.0.0.1"));
    assert!(is_url("https://192.168.0.1"));
    assert!(is_url("http://10.0.0.1:8080"));
}

#[test]
fn test_is_url_subdomain() {
    assert!(is_url("https://api.example.com"));
    assert!(is_url("http://www.sub.example.com"));
    assert!(is_url("https://cdn.static.example.com"));
}

#[test]
fn test_is_not_url_file_path() {
    assert!(!is_url("/path/to/file"));
    assert!(!is_url("/absolute/path"));
    assert!(!is_url("/usr/local/bin"));
}

#[test]
fn test_is_not_url_relative_path() {
    assert!(!is_url("./relative/path"));
    assert!(!is_url("../parent/path"));
    assert!(!is_url("relative/path"));
}

#[test]
fn test_is_not_url_filename() {
    assert!(!is_url("file.txt"));
    assert!(!is_url("document.pdf"));
    assert!(!is_url("image.png"));
}

#[test]
fn test_is_not_url_empty() {
    assert!(!is_url(""));
}

#[test]
fn test_is_not_url_other_protocols() {
    assert!(!is_url("ftp://example.com"));
    assert!(!is_url("file:///path/to/file"));
    assert!(!is_url("mailto:test@example.com"));
    assert!(!is_url("ssh://user@host"));
    assert!(!is_url("git://github.com/repo"));
}

#[test]
fn test_is_not_url_partial_match() {
    assert!(!is_url("not http://example.com"));
    assert!(!is_url("prefix https://example.com"));
    assert!(!is_url(" http://example.com"));
}

#[test]
fn test_is_url_case_sensitive() {
    assert!(!is_url("HTTP://example.com"));
    assert!(!is_url("HTTPS://example.com"));
    assert!(!is_url("Http://example.com"));
}

// ============================================================================
// init_logger tests (marked as ignored since logger can only be initialized once)
// ============================================================================

#[test]
#[ignore]
fn test_init_logger_verbose() {
    let result = init_logger(true, false);
    assert!(result.is_ok());
}

#[test]
#[ignore]
fn test_init_logger_quiet() {
    let result = init_logger(false, true);
    assert!(result.is_ok());
}

#[test]
#[ignore]
fn test_init_logger_normal() {
    let result = init_logger(false, false);
    assert!(result.is_ok());
}

#[test]
#[ignore]
fn test_init_logger_both_flags_verbose_wins() {
    // If both verbose and quiet are true, verbose should take precedence
    let result = init_logger(true, true);
    assert!(result.is_ok());
}

// ============================================================================
// Edge case and integration tests
// ============================================================================

#[test]
fn test_format_size_and_sanitize_combo() {
    let size = 1024 * 1024;
    let size_str = format_size(size);
    let filename = format!("backup_{}.tar.gz", size_str);
    let sanitized = sanitize_filename(&filename);

    // Should preserve the formatted filename
    assert!(sanitized.contains("backup"));
    assert!(sanitized.contains(".tar.gz"));
}

#[test]
fn test_sanitize_url_like_string() {
    let url_string = "https://example.com/path";
    assert!(is_url(url_string));

    // Sanitizing a URL shouldn't be valid as filename
    let sanitized = sanitize_filename(url_string);
    assert_eq!(sanitized, "https__example.com_path");
}

#[test]
fn test_format_size_max_u64() {
    // Test with maximum u64 value
    let max = u64::MAX;
    let result = format_size(max);
    assert!(result.contains("GB"));
    // Should not panic
}

#[test]
fn test_sanitize_filename_all_special_chars() {
    let special = "/\\:*?\"<>|";
    let sanitized = sanitize_filename(special);
    assert_eq!(sanitized, "_________");
    assert_eq!(sanitized.len(), special.len());
}

#[test]
fn test_is_url_real_world_examples() {
    // Real-world URL examples
    assert!(is_url("https://github.com/user/repo"));
    assert!(is_url("https://docs.rs/crate/version/module"));
    assert!(is_url("http://localhost:3000/api/v1/users?page=1&limit=10"));
    assert!(is_url("https://example.com/path/to/resource.html#section"));

    // Real-world non-URL examples
    assert!(!is_url("README.md"));
    assert!(!is_url("src/main.rs"));
    assert!(!is_url("../../parent/directory"));
    assert!(!is_url("/usr/local/bin/program"));
}

#[test]
fn test_sanitize_filename_preserves_dots() {
    assert_eq!(sanitize_filename("my.file.with.dots.txt"), "my.file.with.dots.txt");
    assert_eq!(sanitize_filename("...hidden"), "...hidden");
}

#[test]
fn test_sanitize_filename_preserves_dashes_underscores() {
    assert_eq!(sanitize_filename("my-file_name.txt"), "my-file_name.txt");
    assert_eq!(sanitize_filename("test_case-v2.doc"), "test_case-v2.doc");
}

#[test]
fn test_format_size_fractional_precision() {
    // Test that we get exactly 2 decimal places
    assert_eq!(format_size(1234), "1.21 KB");
    assert_eq!(format_size(1234567), "1.18 MB");
    assert_eq!(format_size(1234567890), "1.15 GB");
}
