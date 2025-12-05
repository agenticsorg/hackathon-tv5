use anyhow::{Context, Result};
use email_address::EmailAddress;
#[cfg(feature = "jsonschema")]
use jsonschema::{Draft, JSONSchema};
#[cfg(feature = "jsonschema")]
use once_cell::sync::Lazy;
use serde_json::Value;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

// Embed the JSON schema at compile time (native only)
#[cfg(feature = "jsonschema")]
const SCHEMA_JSON: &str = include_str!("../../../schemas/arw_model.json");

// Compile schema once at startup (native only)
#[cfg(feature = "jsonschema")]
static COMPILED_SCHEMA: Lazy<JSONSchema> = Lazy::new(|| {
    let schema: Value = serde_json::from_str(SCHEMA_JSON)
        .expect("Failed to parse embedded JSON schema");

    JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema)
        .expect("Failed to compile JSON schema")
});

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub path: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// Validates an ARW manifest (llms.txt) against the JSON schema (native only)
#[cfg(not(target_arch = "wasm32"))]
pub fn validate(path: &Path) -> Result<Vec<ValidationError>> {
    // Read and parse the manifest
    let manifest_content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest at {:?}", path))?;

    let manifest: Value = serde_yaml::from_str(&manifest_content)
        .context("Failed to parse YAML manifest")?;

    // Validate against schema
    validate_manifest(&manifest)
}

/// Validates a manifest value against the ARW schema
pub fn validate_manifest(manifest: &Value) -> Result<Vec<ValidationError>> {
    // Validate the manifest using the pre-compiled schema
    let mut errors = Vec::new();

    #[cfg(feature = "jsonschema")]
    {
        if let Err(validation_errors) = COMPILED_SCHEMA.validate(manifest) {
            for error in validation_errors {
                errors.push(ValidationError {
                    path: error.instance_path.to_string(),
                    message: error.to_string(),
                });
            }
        }
    }

    // Additional custom validations (works in both WASM and native)
    errors.extend(validate_required_fields(manifest)?);
    errors.extend(validate_field_formats(manifest)?);

    Ok(errors)
}

/// Validates required fields are present and non-empty
fn validate_required_fields(manifest: &Value) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Check version (can be string or number in YAML)
    if let Some(version) = manifest.get("version") {
        let is_valid = version.as_str().map_or(false, |s| !s.is_empty())
            || version.as_f64().is_some()
            || version.as_i64().is_some();

        if !is_valid {
            errors.push(ValidationError {
                path: "version".to_string(),
                message: "version is required and must be non-empty".to_string(),
            });
        }
    } else {
        errors.push(ValidationError {
            path: "version".to_string(),
            message: "version is required".to_string(),
        });
    }

    // Check profile
    if let Some(profile) = manifest.get("profile") {
        let profile_str = profile.as_str().unwrap_or("");
        if !["ARW-1", "ARW-2", "ARW-3", "ARW-4"].contains(&profile_str) {
            errors.push(ValidationError {
                path: "profile".to_string(),
                message: format!(
                    "profile must be one of: ARW-1, ARW-2, ARW-3, ARW-4. Got: {}",
                    profile_str
                ),
            });
        }
    } else {
        errors.push(ValidationError {
            path: "profile".to_string(),
            message: "profile is required".to_string(),
        });
    }

    // Check site
    if let Some(site) = manifest.get("site").and_then(|s| s.as_object()) {
        // Check site.name
        if !site.contains_key("name") || site["name"].as_str().map_or(true, |s| s.is_empty()) {
            errors.push(ValidationError {
                path: "site.name".to_string(),
                message: "site.name is required and must be non-empty".to_string(),
            });
        }

        // Check site.homepage
        if !site.contains_key("homepage")
            || site["homepage"].as_str().map_or(true, |s| s.is_empty())
        {
            errors.push(ValidationError {
                path: "site.homepage".to_string(),
                message: "site.homepage is required and must be a valid URL".to_string(),
            });
        }

        // Check site.contact (optional, but if present must be valid)
        // Validation of email format is handled in validate_field_formats
    } else {
        errors.push(ValidationError {
            path: "site".to_string(),
            message: "site is required".to_string(),
        });
    }

    // Check policies
    if let Some(policies) = manifest.get("policies").and_then(|p| p.as_object()) {
        // Check training policy
        if !policies.contains_key("training") {
            errors.push(ValidationError {
                path: "policies.training".to_string(),
                message: "policies.training is required".to_string(),
            });
        } else if let Some(training) = policies["training"].as_object() {
            if !training.contains_key("allowed") {
                errors.push(ValidationError {
                    path: "policies.training.allowed".to_string(),
                    message: "policies.training.allowed is required".to_string(),
                });
            }
        }

        // Check inference policy
        if !policies.contains_key("inference") {
            errors.push(ValidationError {
                path: "policies.inference".to_string(),
                message: "policies.inference is required".to_string(),
            });
        } else if let Some(inference) = policies["inference"].as_object() {
            if !inference.contains_key("allowed") {
                errors.push(ValidationError {
                    path: "policies.inference.allowed".to_string(),
                    message: "policies.inference.allowed is required".to_string(),
                });
            }
        }

        // Check attribution policy
        if !policies.contains_key("attribution") {
            errors.push(ValidationError {
                path: "policies.attribution".to_string(),
                message: "policies.attribution is required".to_string(),
            });
        } else if let Some(attribution) = policies["attribution"].as_object() {
            if !attribution.contains_key("required") {
                errors.push(ValidationError {
                    path: "policies.attribution.required".to_string(),
                    message: "policies.attribution.required is required".to_string(),
                });
            }
        }
    } else {
        errors.push(ValidationError {
            path: "policies".to_string(),
            message: "policies is required".to_string(),
        });
    }

    Ok(errors)
}

/// Validates field formats (URLs, emails, etc.)
fn validate_field_formats(manifest: &Value) -> Result<Vec<ValidationError>> {
    let mut errors = Vec::new();

    // Validate site.homepage is a valid URL
    if let Some(homepage) = manifest
        .get("site")
        .and_then(|s| s.get("homepage"))
        .and_then(|h| h.as_str())
    {
        if !homepage.starts_with("http://") && !homepage.starts_with("https://") {
            errors.push(ValidationError {
                path: "site.homepage".to_string(),
                message: format!("site.homepage must be a valid URL starting with http:// or https://. Got: {}", homepage),
            });
        }
    }

    // Validate site.contact is a valid email (RFC 5322)
    if let Some(contact) = manifest
        .get("site")
        .and_then(|s| s.get("contact"))
        .and_then(|c| c.as_str())
    {
        if !EmailAddress::is_valid(contact) {
            errors.push(ValidationError {
                path: "site.contact".to_string(),
                message: format!(
                    "site.contact must be a valid email address (RFC 5322). Got: {}",
                    contact
                ),
            });
        }
    }

    // Validate content items
    if let Some(content) = manifest.get("content").and_then(|c| c.as_array()) {
        for (idx, item) in content.iter().enumerate() {
            if let Some(obj) = item.as_object() {
                // Validate url field
                if !obj.contains_key("url") {
                    errors.push(ValidationError {
                        path: format!("content[{}].url", idx),
                        message: "content.url is required".to_string(),
                    });
                }

                // Validate machine_view field
                if !obj.contains_key("machine_view") {
                    errors.push(ValidationError {
                        path: format!("content[{}].machine_view", idx),
                        message: "content.machine_view is required".to_string(),
                    });
                }

                // Validate priority enum if present
                if let Some(priority) = obj.get("priority").and_then(|p| p.as_str()) {
                    if !["high", "medium", "low"].contains(&priority) {
                        errors.push(ValidationError {
                            path: format!("content[{}].priority", idx),
                            message: format!(
                                "priority must be one of: high, medium, low. Got: {}",
                                priority
                            ),
                        });
                    }
                }

                // Validate chunks
                if let Some(chunks) = obj.get("chunks").and_then(|c| c.as_array()) {
                    for (chunk_idx, chunk) in chunks.iter().enumerate() {
                        if let Some(chunk_obj) = chunk.as_object() {
                            if !chunk_obj.contains_key("id") {
                                errors.push(ValidationError {
                                    path: format!("content[{}].chunks[{}].id", idx, chunk_idx),
                                    message: "chunk.id is required".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Validate actions
    if let Some(actions) = manifest.get("actions").and_then(|a| a.as_array()) {
        for (idx, action) in actions.iter().enumerate() {
            if let Some(obj) = action.as_object() {
                // Check required fields
                for field in &["id", "name", "endpoint", "method", "auth"] {
                    if !obj.contains_key(*field) {
                        errors.push(ValidationError {
                            path: format!("actions[{}].{}", idx, field),
                            message: format!("actions.{} is required", field),
                        });
                    }
                }

                // Validate method enum
                if let Some(method) = obj.get("method").and_then(|m| m.as_str()) {
                    if !["GET", "POST", "PUT", "PATCH", "DELETE"].contains(&method) {
                        errors.push(ValidationError {
                            path: format!("actions[{}].method", idx),
                            message: format!(
                                "method must be one of: GET, POST, PUT, PATCH, DELETE. Got: {}",
                                method
                            ),
                        });
                    }
                }

                // Validate auth enum
                if let Some(auth) = obj.get("auth").and_then(|a| a.as_str()) {
                    if !["oauth2", "api_key", "none"].contains(&auth) {
                        errors.push(ValidationError {
                            path: format!("actions[{}].auth", idx),
                            message: format!(
                                "auth must be one of: oauth2, api_key, none. Got: {}",
                                auth
                            ),
                        });
                    }
                }
            }
        }
    }

    Ok(errors)
}
