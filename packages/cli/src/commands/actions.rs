use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;

use crate::cli;

pub async fn run(manifest_path: String, test: bool, action_id: Option<String>) -> Result<()> {
    cli::info(&format!("Analyzing actions in {}", manifest_path));

    // Load manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read manifest at {}", manifest_path))?;

    let manifest: Value = serde_yaml::from_str(&manifest_content)
        .context("Failed to parse manifest YAML")?;

    // Get actions
    let actions = manifest
        .get("actions")
        .and_then(|a| a.as_array())
        .context("No actions found in manifest")?;

    if actions.is_empty() {
        cli::warn("No actions defined in manifest");
        return Ok(());
    }

    cli::success(&format!("Found {} action(s)", actions.len()));
    println!();

    // Display or test actions
    for (idx, action) in actions.iter().enumerate() {
        let id = action.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
        let name = action.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");

        // Skip if filtering by action_id
        if let Some(filter_id) = &action_id {
            if id != filter_id {
                continue;
            }
        }

        println!("{}. {} ({})", idx + 1, name, id);

        // Display action details
        display_action_details(action)?;

        // Test endpoint if requested
        if test {
            println!();
            test_action_endpoint(action).await?;
        }

        println!();
    }

    Ok(())
}

fn display_action_details(action: &Value) -> Result<()> {
    if let Some(description) = action.get("description").and_then(|d| d.as_str()) {
        println!("   Description: {}", description);
    }

    if let Some(endpoint) = action.get("endpoint").and_then(|e| e.as_str()) {
        println!("   Endpoint: {}", endpoint);
    }

    if let Some(method) = action.get("method").and_then(|m| m.as_str()) {
        println!("   Method: {}", method);
    }

    if let Some(auth) = action.get("auth").and_then(|a| a.as_str()) {
        println!("   Auth: {}", auth);
    }

    if let Some(scopes) = action.get("scopes").and_then(|s| s.as_array()) {
        let scope_strs: Vec<String> = scopes
            .iter()
            .filter_map(|s| s.as_str().map(String::from))
            .collect();
        println!("   Scopes: {}", scope_strs.join(", "));
    }

    if let Some(schema) = action.get("schema").and_then(|s| s.as_str()) {
        println!("   Schema: {}", schema);
    }

    Ok(())
}

async fn test_action_endpoint(action: &Value) -> Result<()> {
    let endpoint = action
        .get("endpoint")
        .and_then(|e| e.as_str())
        .context("Action missing endpoint")?;

    cli::info(&format!("Testing endpoint: {}", endpoint));

    // Check if endpoint is a full URL or relative path
    let test_url = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        endpoint.to_string()
    } else {
        // For relative paths, we can't test without a base URL
        cli::warn("Cannot test relative endpoint without base URL");
        return Ok(());
    };

    // Try to reach the endpoint (OPTIONS request to check if it exists)
    match reqwest::Client::new()
        .request(reqwest::Method::OPTIONS, &test_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            let status = response.status();
            if status.is_success() || status.as_u16() == 405 {
                // 405 Method Not Allowed is fine - endpoint exists
                cli::success(&format!("   ✓ Endpoint reachable (status: {})", status));

                // Check for required headers
                let headers = response.headers();

                if headers.contains_key("access-control-allow-origin") {
                    cli::success("   ✓ CORS enabled");
                } else {
                    cli::warn("   ⚠ CORS not configured");
                }

                if let Some(allow) = headers.get("allow") {
                    cli::info(&format!("   Allowed methods: {}", allow.to_str().unwrap_or("unknown")));
                }
            } else {
                cli::warn(&format!("   ⚠ Endpoint returned {}", status));
            }
        }
        Err(e) => {
            if e.is_timeout() {
                cli::error("   ✗ Endpoint timeout (>5s)");
            } else if e.is_connect() {
                cli::error("   ✗ Cannot connect to endpoint");
            } else {
                cli::error(&format!("   ✗ Endpoint error: {}", e));
            }
        }
    }

    // Validate auth configuration
    if let Some(auth) = action.get("auth").and_then(|a| a.as_str()) {
        match auth {
            "oauth2" => {
                cli::info("   OAuth2 required - check authorization flow");
            }
            "api_key" => {
                cli::info("   API key required");
            }
            "none" => {
                cli::warn("   ⚠ No authentication required - ensure this is intentional");
            }
            _ => {
                cli::warn(&format!("   ⚠ Unknown auth type: {}", auth));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_display_action_details() {
        let action = json!({
            "id": "test_action",
            "name": "Test Action",
            "description": "A test action",
            "endpoint": "/api/test",
            "method": "POST",
            "auth": "oauth2",
            "scopes": ["test:write"],
            "schema": "https://schema.org/Action"
        });

        let result = display_action_details(&action);
        assert!(result.is_ok());
    }
}
