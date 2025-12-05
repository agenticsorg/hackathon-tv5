use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArwPolicies {
    #[serde(rename = "$schema")]
    pub schema: String,
    pub training: TrainingPolicy,
    pub inference: InferencePolicy,
    pub attribution: AttributionPolicy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrainingPolicy {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InferencePolicy {
    pub allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrictions: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AttributionPolicy {
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
}

/// Generate .well-known/arw-policies.json from llms.txt
pub fn generate<P: AsRef<Path>>(
    site_path: P,
    training_allowed: bool,
    inference_allowed: bool,
    attribution_required: bool,
) -> Result<()> {
    let policies = ArwPolicies {
        schema: "https://arw.dev/schemas/arw-policies.schema.json".to_string(),
        training: TrainingPolicy {
            allowed: training_allowed,
            note: if !training_allowed {
                Some("Content may not be used for training AI models".to_string())
            } else {
                Some("Content may be used for training with proper attribution".to_string())
            },
        },
        inference: InferencePolicy {
            allowed: inference_allowed,
            restrictions: if inference_allowed {
                Some(vec![
                    "Must provide attribution".to_string(),
                    "Must respect rate limits".to_string(),
                ])
            } else {
                None
            },
        },
        attribution: AttributionPolicy {
            required: attribution_required,
            format: if attribution_required {
                Some("link".to_string())
            } else {
                None
            },
        },
    };

    let well_known_dir = site_path.as_ref().join(".well-known");
    fs::create_dir_all(&well_known_dir)
        .context("Failed to create .well-known directory")?;

    let output_path = well_known_dir.join("arw-policies.json");
    let content = serde_json::to_string_pretty(&policies)?;

    fs::write(&output_path, content)
        .with_context(|| format!("Failed to write arw-policies.json to {:?}", output_path))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_policies() {
        let policies = ArwPolicies {
            schema: "https://arw.dev/schemas/arw-policies.schema.json".to_string(),
            training: TrainingPolicy {
                allowed: false,
                note: Some("Training not allowed".to_string()),
            },
            inference: InferencePolicy {
                allowed: true,
                restrictions: Some(vec!["Attribution required".to_string()]),
            },
            attribution: AttributionPolicy {
                required: true,
                format: Some("link".to_string()),
            },
        };

        let json = serde_json::to_string_pretty(&policies).unwrap();
        assert!(json.contains("training"));
        assert!(json.contains("inference"));
        assert!(json.contains("attribution"));
    }
}
