//! Validation utilities for Media Gateway data structures
//!
//! Provides validation functions and regex patterns for common validation scenarios.

use crate::error::MediaGatewayError;
use once_cell::sync::Lazy;
use regex::Regex;

/// IMDb ID regex pattern (e.g., tt0111161)
pub static IMDB_ID_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^tt\d{7,8}$").expect("Failed to compile IMDb ID regex"));

/// Email regex pattern (basic validation)
pub static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("Failed to compile email regex")
});

/// ISO 639-1 language code regex (2 lowercase letters)
pub static LANGUAGE_CODE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-z]{2}$").expect("Failed to compile language code regex"));

/// ISO 3166-1 alpha-2 country code regex (2 uppercase letters)
pub static COUNTRY_CODE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[A-Z]{2}$").expect("Failed to compile country code regex"));

/// URL validation regex (basic)
pub static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").expect("Failed to compile URL regex"));

/// Validate IMDb ID format
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_imdb_id;
///
/// assert!(validate_imdb_id("tt0111161").is_ok());
/// assert!(validate_imdb_id("invalid").is_err());
/// ```
pub fn validate_imdb_id(id: &str) -> Result<(), MediaGatewayError> {
    if IMDB_ID_REGEX.is_match(id) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            "Invalid IMDb ID format (expected tt followed by 7-8 digits)",
            "imdb_id",
        ))
    }
}

/// Validate email address format
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_email;
///
/// assert!(validate_email("user@example.com").is_ok());
/// assert!(validate_email("invalid-email").is_err());
/// ```
pub fn validate_email(email: &str) -> Result<(), MediaGatewayError> {
    if EMAIL_REGEX.is_match(email) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            "Invalid email address format",
            "email",
        ))
    }
}

/// Validate ISO 639-1 language code
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_language_code;
///
/// assert!(validate_language_code("en").is_ok());
/// assert!(validate_language_code("fr").is_ok());
/// assert!(validate_language_code("ENG").is_err());
/// ```
pub fn validate_language_code(code: &str) -> Result<(), MediaGatewayError> {
    if LANGUAGE_CODE_REGEX.is_match(code) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            "Invalid language code (expected 2 lowercase letters)",
            "language_code",
        ))
    }
}

/// Validate ISO 3166-1 alpha-2 country code
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_country_code;
///
/// assert!(validate_country_code("US").is_ok());
/// assert!(validate_country_code("CA").is_ok());
/// assert!(validate_country_code("usa").is_err());
/// ```
pub fn validate_country_code(code: &str) -> Result<(), MediaGatewayError> {
    if COUNTRY_CODE_REGEX.is_match(code) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            "Invalid country code (expected 2 uppercase letters)",
            "country_code",
        ))
    }
}

/// Validate URL format
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_url;
///
/// assert!(validate_url("https://example.com").is_ok());
/// assert!(validate_url("http://example.com/path").is_ok());
/// assert!(validate_url("not-a-url").is_err());
/// ```
pub fn validate_url(url: &str) -> Result<(), MediaGatewayError> {
    if URL_REGEX.is_match(url) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            "Invalid URL format",
            "url",
        ))
    }
}

/// Validate release year is within reasonable bounds
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_release_year;
///
/// assert!(validate_release_year(2024).is_ok());
/// assert!(validate_release_year(1900).is_ok());
/// assert!(validate_release_year(1800).is_err());
/// assert!(validate_release_year(2200).is_err());
/// ```
pub fn validate_release_year(year: i32) -> Result<(), MediaGatewayError> {
    if (1850..=2100).contains(&year) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            format!("Release year must be between 1850 and 2100, got {}", year),
            "release_year",
        ))
    }
}

/// Validate runtime is positive
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_runtime;
///
/// assert!(validate_runtime(120).is_ok());
/// assert!(validate_runtime(0).is_err());
/// assert!(validate_runtime(-10).is_err());
/// ```
pub fn validate_runtime(minutes: i32) -> Result<(), MediaGatewayError> {
    if minutes > 0 {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            "Runtime must be positive",
            "runtime_minutes",
        ))
    }
}

/// Validate rating is within 0.0 to 10.0 range
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_rating;
///
/// assert!(validate_rating(7.5).is_ok());
/// assert!(validate_rating(0.0).is_ok());
/// assert!(validate_rating(10.0).is_ok());
/// assert!(validate_rating(-1.0).is_err());
/// assert!(validate_rating(11.0).is_err());
/// ```
pub fn validate_rating(rating: f32) -> Result<(), MediaGatewayError> {
    if (0.0..=10.0).contains(&rating) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            format!("Rating must be between 0.0 and 10.0, got {}", rating),
            "rating",
        ))
    }
}

/// Validate data quality score is within 0.0 to 1.0 range
///
/// # Examples
///
/// ```
/// use media_gateway_core::validation::validate_quality_score;
///
/// assert!(validate_quality_score(0.85).is_ok());
/// assert!(validate_quality_score(0.0).is_ok());
/// assert!(validate_quality_score(1.0).is_ok());
/// assert!(validate_quality_score(-0.1).is_err());
/// assert!(validate_quality_score(1.1).is_err());
/// ```
pub fn validate_quality_score(score: f32) -> Result<(), MediaGatewayError> {
    if (0.0..=1.0).contains(&score) {
        Ok(())
    } else {
        Err(MediaGatewayError::validation_field(
            format!("Quality score must be between 0.0 and 1.0, got {}", score),
            "data_quality_score",
        ))
    }
}

/// Validate string length is within bounds
pub fn validate_string_length(
    value: &str,
    field: &str,
    min: Option<usize>,
    max: Option<usize>,
) -> Result<(), MediaGatewayError> {
    let len = value.len();

    if let Some(min_len) = min {
        if len < min_len {
            return Err(MediaGatewayError::validation_field(
                format!(
                    "Field '{}' must be at least {} characters, got {}",
                    field, min_len, len
                ),
                field,
            ));
        }
    }

    if let Some(max_len) = max {
        if len > max_len {
            return Err(MediaGatewayError::validation_field(
                format!(
                    "Field '{}' must be at most {} characters, got {}",
                    field, max_len, len
                ),
                field,
            ));
        }
    }

    Ok(())
}

/// Validate a vector is not empty
pub fn validate_not_empty<T>(vec: &[T], field: &str) -> Result<(), MediaGatewayError> {
    if vec.is_empty() {
        Err(MediaGatewayError::validation_field(
            format!("Field '{}' must not be empty", field),
            field,
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_imdb_id_validation() {
        assert!(validate_imdb_id("tt0111161").is_ok());
        assert!(validate_imdb_id("tt1234567").is_ok());
        assert!(validate_imdb_id("tt12345678").is_ok());

        assert!(validate_imdb_id("invalid").is_err());
        assert!(validate_imdb_id("tt123").is_err());
        assert!(validate_imdb_id("123456789").is_err());
    }

    #[test]
    fn test_email_validation() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test.user+tag@domain.co.uk").is_ok());

        assert!(validate_email("invalid").is_err());
        assert!(validate_email("@example.com").is_err());
        assert!(validate_email("user@").is_err());
    }

    #[test]
    fn test_language_code_validation() {
        assert!(validate_language_code("en").is_ok());
        assert!(validate_language_code("fr").is_ok());
        assert!(validate_language_code("es").is_ok());

        assert!(validate_language_code("ENG").is_err());
        assert!(validate_language_code("e").is_err());
        assert!(validate_language_code("eng").is_err());
    }

    #[test]
    fn test_country_code_validation() {
        assert!(validate_country_code("US").is_ok());
        assert!(validate_country_code("CA").is_ok());
        assert!(validate_country_code("GB").is_ok());

        assert!(validate_country_code("usa").is_err());
        assert!(validate_country_code("U").is_err());
        assert!(validate_country_code("USA").is_err());
    }

    #[test]
    fn test_url_validation() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com/path").is_ok());
        assert!(validate_url("https://example.com/path?query=value").is_ok());

        assert!(validate_url("not-a-url").is_err());
        assert!(validate_url("ftp://example.com").is_err());
    }

    #[test]
    fn test_release_year_validation() {
        assert!(validate_release_year(2024).is_ok());
        assert!(validate_release_year(1900).is_ok());
        assert!(validate_release_year(2050).is_ok());

        assert!(validate_release_year(1800).is_err());
        assert!(validate_release_year(2200).is_err());
    }

    #[test]
    fn test_runtime_validation() {
        assert!(validate_runtime(120).is_ok());
        assert!(validate_runtime(1).is_ok());

        assert!(validate_runtime(0).is_err());
        assert!(validate_runtime(-10).is_err());
    }

    #[test]
    fn test_rating_validation() {
        assert!(validate_rating(7.5).is_ok());
        assert!(validate_rating(0.0).is_ok());
        assert!(validate_rating(10.0).is_ok());

        assert!(validate_rating(-1.0).is_err());
        assert!(validate_rating(11.0).is_err());
    }

    #[test]
    fn test_quality_score_validation() {
        assert!(validate_quality_score(0.85).is_ok());
        assert!(validate_quality_score(0.0).is_ok());
        assert!(validate_quality_score(1.0).is_ok());

        assert!(validate_quality_score(-0.1).is_err());
        assert!(validate_quality_score(1.1).is_err());
    }

    #[test]
    fn test_string_length_validation() {
        assert!(validate_string_length("hello", "test", Some(1), Some(10)).is_ok());
        assert!(validate_string_length("hello", "test", Some(5), Some(5)).is_ok());

        assert!(validate_string_length("hi", "test", Some(5), None).is_err());
        assert!(validate_string_length("too long string", "test", None, Some(5)).is_err());
    }

    #[test]
    fn test_not_empty_validation() {
        assert!(validate_not_empty(&[1, 2, 3], "test").is_ok());
        assert!(validate_not_empty(&["a"], "test").is_ok());

        let empty: Vec<i32> = vec![];
        assert!(validate_not_empty(&empty, "test").is_err());
    }
}
