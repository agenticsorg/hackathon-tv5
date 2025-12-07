use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use dotenvy::dotenv;
use std::env;

mod commands;

#[derive(Parser)]
#[command(name = "mg-migrate")]
#[command(about = "Database migration CLI tool for Media Gateway", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true, help = "Preview changes without applying them")]
    dry_run: bool,

    #[arg(
        long,
        global = true,
        env = "DATABASE_URL",
        help = "Database connection URL"
    )]
    database_url: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Apply all pending migrations")]
    Up,

    #[command(about = "Rollback the last migration")]
    Down {
        #[arg(
            short,
            long,
            default_value = "1",
            help = "Number of migrations to rollback"
        )]
        steps: usize,
    },

    #[command(about = "Show migration status")]
    Status,

    #[command(about = "Create a new migration file")]
    Create {
        #[arg(help = "Name of the migration (e.g., add_users)")]
        name: String,
    },
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{} {}", "Error:".red().bold(), err);
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    // Create command doesn't need database URL
    if let Commands::Create { name } = cli.command {
        commands::create(&name)?;
        return Ok(());
    }

    let database_url = cli
        .database_url
        .or_else(|| env::var("DATABASE_URL").ok())
        .context(
            "DATABASE_URL must be set either as environment variable or --database-url flag",
        )?;

    match cli.command {
        Commands::Up => {
            commands::up(&database_url, cli.dry_run).await?;
        }
        Commands::Down { steps } => {
            commands::down(&database_url, steps, cli.dry_run).await?;
        }
        Commands::Status => {
            commands::status(&database_url).await?;
        }
        Commands::Create { .. } => {
            unreachable!("Create command handled above")
        }
    }

    Ok(())
}
