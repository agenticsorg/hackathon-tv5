use anyhow::{Context, Result};
use clap::Subcommand;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use colored::Colorize;

#[derive(Subcommand, Debug)]
pub enum GeoCommands {
    /// Analyze content for GEO optimization
    Analyze {
        /// Path to content file or directory
        #[arg(value_name = "PATH")]
        path: PathBuf,

        /// Domain for optimization (e.g., healthcare, ecommerce)
        #[arg(short, long)]
        domain: Option<String>,

        /// Use LLM for enhanced analysis
        #[arg(long)]
        use_llm: bool,

        /// Output format (text, json, html)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Generate GEO report
    Report {
        /// Output directory for reports
        #[arg(short, long, default_value = "./output/geo-reports")]
        output: PathBuf,

        /// Report format (json, html, markdown)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Include all analysis details
        #[arg(long)]
        detailed: bool,
    },

    /// Validate GEO metadata
    Validate {
        /// Path to validate
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoAnalysisResult {
    pub profile: String,
    pub citations: Vec<Citation>,
    pub statistics: Vec<Statistic>,
    pub quotations: Vec<Quotation>,
    pub quality: QualityScore,
    pub entities: Vec<Entity>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Citation {
    pub url: String,
    pub title: Option<String>,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistic {
    pub value: String,
    pub context: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Quotation {
    pub text: String,
    pub source: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QualityScore {
    pub overall: f64,
    pub readability: f64,
    pub credibility: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entity {
    pub name: String,
    pub entity_type: String,
    pub count: usize,
}

pub fn handle_geo_command(cmd: GeoCommands) -> Result<()> {
    match cmd {
        GeoCommands::Analyze {
            path,
            domain,
            use_llm,
            format,
            output,
        } => analyze_content(path, domain, use_llm, format, output),

        GeoCommands::Report {
            output,
            format,
            detailed,
        } => generate_report(output, format, detailed),

        GeoCommands::Validate { path } => validate_geo(path),
    }
}

fn analyze_content(
    path: PathBuf,
    domain: Option<String>,
    use_llm: bool,
    format: String,
    output: Option<PathBuf>,
) -> Result<()> {
    println!("{}", "🔍 Analyzing content for GEO optimization...".cyan().bold());

    // Check if path exists
    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    // Call Node.js bridge to use @arw/geo package
    let bridge_path = get_bridge_path()?;

    let mut cmd = Command::new("node");
    cmd.arg(&bridge_path)
        .arg("analyze")
        .arg(&path);

    if let Some(d) = domain {
        cmd.arg("--domain").arg(d);
    }

    if use_llm {
        cmd.arg("--use-llm");
    }

    cmd.arg("--format").arg(&format);

    if let Some(o) = &output {
        cmd.arg("--output").arg(o);
    }

    let output_result = cmd.output()
        .context("Failed to execute GEO analysis")?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        anyhow::bail!("GEO analysis failed: {}", stderr);
    }

    // Print output
    let stdout = String::from_utf8_lossy(&output_result.stdout);
    print!("{}", stdout);

    if output.is_some() {
        println!("\n{}", "✓ Analysis complete!".green().bold());
    }

    Ok(())
}

fn generate_report(output_dir: PathBuf, format: String, detailed: bool) -> Result<()> {
    println!("{}", "📊 Generating GEO report...".cyan().bold());

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir)
        .context("Failed to create output directory")?;

    let bridge_path = get_bridge_path()?;

    let mut cmd = Command::new("node");
    cmd.arg(&bridge_path)
        .arg("report")
        .arg("--output").arg(&output_dir)
        .arg("--format").arg(&format);

    if detailed {
        cmd.arg("--detailed");
    }

    let output_result = cmd.output()
        .context("Failed to generate GEO report")?;

    if !output_result.status.success() {
        let stderr = String::from_utf8_lossy(&output_result.stderr);
        anyhow::bail!("Report generation failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output_result.stdout);
    print!("{}", stdout);

    println!("\n{}", "✓ Report generated successfully!".green().bold());

    Ok(())
}

fn validate_geo(path: PathBuf) -> Result<()> {
    println!("{}", "🔍 Validating GEO metadata...".cyan().bold());

    if !path.exists() {
        anyhow::bail!("Path does not exist: {}", path.display());
    }

    let bridge_path = get_bridge_path()?;

    let output = Command::new("node")
        .arg(&bridge_path)
        .arg("validate")
        .arg(&path)
        .output()
        .context("Failed to validate GEO metadata")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Validation failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    print!("{}", stdout);

    Ok(())
}

fn get_bridge_path() -> Result<PathBuf> {
    // Look for bridge in several possible locations
    let possible_paths = vec![
        PathBuf::from("packages/cli/bridge/geo-bridge.js"),
        PathBuf::from("./bridge/geo-bridge.js"),
        PathBuf::from("../bridge/geo-bridge.js"),
    ];

    for path in possible_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("GEO bridge not found. Please ensure @arw/geo is installed.")
}
