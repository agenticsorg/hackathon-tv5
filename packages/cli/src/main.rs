use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod cli;
mod commands;
mod generators;
mod parsers;
mod server;
mod utils;
mod validators;

use commands::*;

#[derive(Parser)]
#[command(
    name = "arw",
    version,
    about = "Agent-Ready Web (ARW) CLI - Make your website accessible to AI agents",
    long_about = "A command-line tool for implementing the Agent-Ready Web (ARW) specification.\n\
                  Generate machine views, discovery files, and validate ARW compliance.\n\n\
                  Learn more: https://github.com/agent-ready-web/agent-ready-web"
)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Suppress output except errors
    #[arg(short, long, global = true)]
    quiet: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize ARW structure for your site
    #[command(visible_alias = "i")]
    Init {
        /// Site root directory
        #[arg(short, long, default_value = "public")]
        path: String,

        /// Skip interactive prompts and use defaults
        #[arg(short = 'y', long)]
        yes: bool,

        /// Also generate sitemap.xml and arw-content.json
        #[arg(short = 'm', long)]
        with_manifests: bool,

        /// Base URL for manifest generation (required if --with-manifests)
        #[arg(short = 'b', long)]
        base_url: Option<String>,
    },

    /// Generate machine views (.llm.md) from HTML files
    #[command(visible_alias = "gen")]
    Generate {
        /// Source file or directory
        source: String,

        /// Output directory for machine views
        #[arg(short, long)]
        output: Option<String>,

        /// Process directories recursively
        #[arg(short, long)]
        recursive: bool,

        /// Input format (html, markdown, auto)
        #[arg(short = 'i', long, default_value = "auto")]
        format: String,

        /// Force overwrite existing files
        #[arg(short = 'f', long)]
        force: bool,

        /// Output format (markdown, toon)
        #[arg(long, default_value = "markdown")]
        output_format: String,

        /// Sync generated files to llms.txt
        #[arg(short = 's', long)]
        sync: bool,
    },

    /// Generate sitemap.llm.json from site structure
    #[command(visible_alias = "sm")]
    Sitemap {
        /// Site URL or local path
        #[arg(default_value = "public")]
        source: String,

        /// Output file path
        #[arg(short, long, default_value = "sitemap.llm.json")]
        output: String,

        /// Maximum crawl depth
        #[arg(short, long, default_value = "5")]
        depth: usize,

        /// Base URL for the site
        #[arg(short, long)]
        base_url: Option<String>,
    },

    /// Validate ARW implementation
    #[command(visible_alias = "val")]
    Validate {
        /// Site root directory
        #[arg(short, long, default_value = "public")]
        path: String,

        /// Strict validation mode
        #[arg(short, long)]
        strict: bool,

        /// Attempt to auto-fix issues
        #[arg(short = 'f', long)]
        fix: bool,
    },

    /// Start development server for testing
    #[command(visible_alias = "dev")]
    Serve {
        /// Site root directory
        #[arg(short, long, default_value = "public")]
        path: String,

        /// Server port
        #[arg(short = 'p', long, default_value = "3000")]
        port: u16,

        /// Enable hot reload
        #[arg(short, long)]
        watch: bool,

        /// Open browser automatically
        #[arg(short, long)]
        open: bool,
    },

    /// Scan and analyze a website for ARW implementation
    #[command(visible_alias = "analyze")]
    Scan {
        /// Site URL to scan
        url: String,

        /// Maximum crawl depth
        #[arg(short, long, default_value = "3")]
        depth: usize,

        /// Output directory for generated files
        #[arg(short, long)]
        output: Option<String>,

        /// Dry run (don't generate files)
        #[arg(short = 'n', long)]
        dry_run: bool,
    },

    /// Manage policy.json configuration
    #[command(visible_alias = "pol")]
    Policy {
        /// Site root directory
        #[arg(short, long, default_value = "public")]
        path: String,

        /// Use a template (e.g., ecommerce, documentation, blog)
        #[arg(short, long)]
        template: Option<String>,

        /// Edit existing policy interactively
        #[arg(short, long)]
        edit: bool,
    },

    /// Generate robots.txt from llms.txt manifest
    #[command(visible_alias = "rob")]
    Robots {
        /// Path to llms.txt manifest
        #[arg(short, long, default_value = "public/llms.txt")]
        manifest: String,

        /// Output file path
        #[arg(short, long, default_value = "public/robots.txt")]
        output: String,
    },

    /// Watch for file changes and auto-regenerate
    #[command(visible_alias = "w")]
    Watch {
        /// Directory to watch
        #[arg(short, long, default_value = "public")]
        path: String,

        /// Auto-generate machine views from HTML changes
        #[arg(short, long)]
        generate: bool,

        /// Auto-validate llms.txt on changes
        #[arg(short = 'V', long)]
        validate: bool,
    },

    /// Manage and test actions (ARW-3)
    #[command(visible_alias = "act")]
    Actions {
        /// Path to llms.txt manifest
        #[arg(short, long, default_value = "public/llms.txt")]
        manifest: String,

        /// Test action endpoints for reachability
        #[arg(short, long)]
        test: bool,

        /// Filter by specific action ID
        #[arg(short = 'i', long)]
        action_id: Option<String>,
    },

    /// Build all ARW files from llms.txt (manifest → .well-known, sitemap, robots)
    Build {
        /// Site root directory containing llms.txt
        #[arg(short, long, default_value = "public")]
        source: String,

        /// Base URL for the site (defaults to homepage in llms.txt)
        #[arg(short, long)]
        base_url: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    utils::init_logger(cli.verbose, cli.quiet)?;

    // Print banner unless quiet mode
    if !cli.quiet {
        print_banner();
    }

    // Execute command
    let result = match cli.command {
        Commands::Init { path, yes, with_manifests, base_url } => init::run(path, yes, with_manifests, base_url).await,

        Commands::Generate {
            source,
            output,
            recursive,
            format,
            force,
            output_format,
            sync,
        } => generate::run(source, output, recursive, format, force, output_format, sync).await,

        Commands::Sitemap {
            source,
            output,
            depth,
            base_url,
        } => sitemap::run(source, output, depth, base_url).await,

        Commands::Validate { path, strict, fix } => validate::run(path, strict, fix).await,

        Commands::Serve {
            path,
            port,
            watch,
            open,
        } => serve::run(path, port, watch, open).await,

        Commands::Scan {
            url,
            depth,
            output,
            dry_run,
        } => scan::run(url, depth, output, dry_run).await,

        Commands::Policy {
            path,
            template,
            edit,
        } => policy::run(path, template, edit).await,

        Commands::Robots { manifest, output } => robots::run(manifest, output).await,

        Commands::Watch {
            path,
            generate,
            validate,
        } => watch::run(path, generate, validate).await,

        Commands::Actions {
            manifest,
            test,
            action_id,
        } => actions::run(manifest, test, action_id).await,

        Commands::Build { source, base_url } => build::run(source, base_url).await,
    };

    // Handle result
    match result {
        Ok(()) => {
            if !cli.quiet {
                println!("\n{}", "✓ Success!".green().bold());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("\n{} {}", "✗ Error:".red().bold(), e);
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!(
        r#"
    {}
    {}
    {}
    "#,
        "╔═══════════════════════════════════════════╗".cyan(),
        "║  ARW CLI - Agent-Ready Web Toolkit       ║".cyan().bold(),
        "╚═══════════════════════════════════════════╝".cyan()
    );
}
