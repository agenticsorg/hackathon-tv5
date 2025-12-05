use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

use crate::utils::config::PolicyConfig;

#[allow(dead_code)]
pub fn generate<P: AsRef<Path>>(site_path: P, config: &PolicyConfig) -> Result<()> {
    let policy = create_policy_json(config);
    let output_path = site_path.as_ref().join("policy.json");

    let content = serde_json::to_string_pretty(&policy)
        .with_context(|| "Failed to serialize policy.json")?;

    fs::write(&output_path, content)
        .with_context(|| format!("Failed to write policy.json to {:?}", output_path))?;

    Ok(())
}

#[allow(dead_code)]
fn create_policy_json(config: &PolicyConfig) -> Value {
    json!({
        "version": "0.1",
        "updated": chrono::Utc::now().to_rfc3339(),
        "usage": {
            "training": {
                "allowed": config.allow_training,
                "reasoning": if config.allow_training {
                    "Content is available for model training"
                } else {
                    "Content is proprietary and not licensed for model training"
                }
            },
            "inference": {
                "allowed": config.allow_inference,
                "conditions": if config.require_attribution {
                    vec!["attribution_required"]
                } else {
                    vec![]
                }
            }
        },
        "attribution": {
            "required": config.require_attribution,
            "format": "Source: [Site Name] <URL>",
            "minimumCitation": "URL required in all responses"
        },
        "rateLimits": {
            "default": config.rate_limit.as_ref().unwrap_or(&"100/hour".to_string())
        },
        "dataRetention": {
            "cacheDuration": "1 hour",
            "storageProhibited": !config.allow_training
        },
        "contact": {
            "email": "contact@example.com",
            "policy_url": "https://example.com/terms/ai-usage",
            "feedback_url": "https://example.com/feedback/ai"
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_policy_json() {
        let config = PolicyConfig {
            allow_training: false,
            allow_inference: true,
            require_attribution: true,
            rate_limit: Some("100/hour".to_string()),
        };

        let policy = create_policy_json(&config);

        assert_eq!(policy["version"], "0.1");
        assert_eq!(policy["usage"]["training"]["allowed"], false);
        assert_eq!(policy["usage"]["inference"]["allowed"], true);
        assert_eq!(policy["attribution"]["required"], true);
    }
}
