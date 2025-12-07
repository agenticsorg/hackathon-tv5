use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use sqlx::{postgres::PgPoolOptions, PgPool, Row};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

const MIGRATIONS_DIR: &str = "migrations";
const SCHEMA_MIGRATIONS_TABLE: &str = "schema_migrations";

#[derive(Debug, Clone)]
struct Migration {
    version: String,
    name: String,
    #[allow(dead_code)]
    path: PathBuf,
    sql: String,
}

#[derive(Debug, PartialEq)]
struct AppliedMigration {
    version: String,
    applied_at: chrono::DateTime<chrono::Utc>,
}

pub async fn up(database_url: &str, dry_run: bool) -> Result<()> {
    println!("{}", "Running migrations...".cyan().bold());

    let pool = create_pool(database_url).await?;
    ensure_migrations_table(&pool).await?;

    let migrations = load_migrations()?;
    let applied = get_applied_migrations(&pool).await?;

    let applied_versions: HashSet<String> = applied.iter().map(|m| m.version.clone()).collect();

    let pending: Vec<_> = migrations
        .iter()
        .filter(|m| !applied_versions.contains(&m.version))
        .collect();

    if pending.is_empty() {
        println!("{}", "No pending migrations.".green());
        return Ok(());
    }

    println!(
        "\n{} {} migration(s) to apply:\n",
        "Found".cyan(),
        pending.len()
    );

    for migration in &pending {
        println!("  {} {}", "→".cyan(), migration.name.white());
    }

    if dry_run {
        println!("\n{}", "DRY RUN - No changes applied".yellow().bold());
        return Ok(());
    }

    println!();

    for migration in pending {
        apply_migration(&pool, migration).await?;
    }

    println!(
        "\n{}",
        "All migrations applied successfully!".green().bold()
    );

    Ok(())
}

pub async fn down(database_url: &str, steps: usize, dry_run: bool) -> Result<()> {
    println!("{}", "Rolling back migrations...".cyan().bold());

    let pool = create_pool(database_url).await?;
    ensure_migrations_table(&pool).await?;

    let applied = get_applied_migrations(&pool).await?;

    if applied.is_empty() {
        println!("{}", "No migrations to rollback.".yellow());
        return Ok(());
    }

    let to_rollback: Vec<_> = applied.iter().rev().take(steps).collect();

    println!(
        "\n{} {} migration(s) to rollback:\n",
        "Found".cyan(),
        to_rollback.len()
    );

    for migration in &to_rollback {
        println!(
            "  {} {} (applied at {})",
            "→".cyan(),
            migration.version.white(),
            migration.applied_at.format("%Y-%m-%d %H:%M:%S")
        );
    }

    if dry_run {
        println!("\n{}", "DRY RUN - No changes applied".yellow().bold());
        return Ok(());
    }

    println!();

    for migration in to_rollback {
        rollback_migration(&pool, &migration.version).await?;
    }

    println!(
        "\n{}",
        "Migrations rolled back successfully!".green().bold()
    );

    Ok(())
}

pub async fn status(database_url: &str) -> Result<()> {
    let pool = create_pool(database_url).await?;
    ensure_migrations_table(&pool).await?;

    let migrations = load_migrations()?;
    let applied_set: HashSet<String> = get_applied_migrations(&pool)
        .await?
        .into_iter()
        .map(|m| m.version)
        .collect();

    println!("{}\n", "Migration Status".cyan().bold());
    println!(
        "{:<20} {:<40} {}",
        "Version".bold(),
        "Name".bold(),
        "Status".bold()
    );
    println!("{}", "─".repeat(80).dimmed());

    for migration in &migrations {
        let status = if applied_set.contains(&migration.version) {
            "APPLIED".green()
        } else {
            "PENDING".yellow()
        };

        println!(
            "{:<20} {:<40} {}",
            migration.version, migration.name, status
        );
    }

    let total = migrations.len();
    let applied_count = applied_set.len();
    let pending = total - applied_count;

    println!();
    println!(
        "{} {} total, {} applied, {} pending",
        "Summary:".cyan().bold(),
        total,
        applied_count.to_string().green(),
        pending.to_string().yellow()
    );

    Ok(())
}

pub fn create(name: &str) -> Result<()> {
    let migrations_dir = Path::new(MIGRATIONS_DIR);

    if !migrations_dir.exists() {
        fs::create_dir_all(migrations_dir).context("Failed to create migrations directory")?;
    }

    let existing = load_migrations()?;
    let next_version = if existing.is_empty() {
        1
    } else {
        let max_version = existing
            .iter()
            .filter_map(|m| m.version.parse::<u32>().ok())
            .max()
            .unwrap_or(0);
        max_version + 1
    };

    let version = format!("{:03}", next_version);
    let sanitized_name = sanitize_migration_name(name);
    let filename = format!("{}_{}.sql", version, sanitized_name);
    let filepath = migrations_dir.join(&filename);

    let template = format!(
        "-- Migration: {}\n\
         -- Description: TODO: Add description\n\
         \n\
         -- Add your SQL migration here\n\
         \n",
        sanitized_name
    );

    fs::write(&filepath, template).context("Failed to write migration file")?;

    println!(
        "{} {}",
        "Created migration:".green().bold(),
        filepath.display()
    );

    Ok(())
}

async fn create_pool(database_url: &str) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .connect(database_url)
        .await
        .context("Failed to connect to database")
}

async fn ensure_migrations_table(pool: &PgPool) -> Result<()> {
    sqlx::query(&format!(
        r#"
        CREATE TABLE IF NOT EXISTS {} (
            version VARCHAR(255) PRIMARY KEY,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
        SCHEMA_MIGRATIONS_TABLE
    ))
    .execute(pool)
    .await
    .context("Failed to create schema_migrations table")?;

    Ok(())
}

fn load_migrations() -> Result<Vec<Migration>> {
    let migrations_dir = Path::new(MIGRATIONS_DIR);

    if !migrations_dir.exists() {
        return Ok(Vec::new());
    }

    let version_regex = Regex::new(r"^(\d+)_(.+)\.sql$").unwrap();
    let mut migrations = Vec::new();

    for entry in fs::read_dir(migrations_dir).context("Failed to read migrations directory")? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid filename")?;

        if let Some(captures) = version_regex.captures(filename) {
            let version = captures.get(1).unwrap().as_str().to_string();
            let name = captures.get(2).unwrap().as_str().to_string();

            let sql = fs::read_to_string(&path).context("Failed to read migration file")?;

            migrations.push(Migration {
                version,
                name,
                path,
                sql,
            });
        }
    }

    migrations.sort_by(|a, b| {
        a.version
            .parse::<u32>()
            .unwrap_or(0)
            .cmp(&b.version.parse::<u32>().unwrap_or(0))
    });

    Ok(migrations)
}

async fn get_applied_migrations(pool: &PgPool) -> Result<Vec<AppliedMigration>> {
    let rows = sqlx::query(&format!(
        "SELECT version, applied_at FROM {} ORDER BY version",
        SCHEMA_MIGRATIONS_TABLE
    ))
    .fetch_all(pool)
    .await
    .context("Failed to fetch applied migrations")?;

    let migrations = rows
        .into_iter()
        .map(|row| AppliedMigration {
            version: row.get("version"),
            applied_at: row.get("applied_at"),
        })
        .collect();

    Ok(migrations)
}

async fn apply_migration(pool: &PgPool, migration: &Migration) -> Result<()> {
    print!("Applying {} ... ", migration.name.white());

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    sqlx::query(&migration.sql)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("Failed to execute migration {}", migration.version))?;

    sqlx::query(&format!(
        "INSERT INTO {} (version) VALUES ($1)",
        SCHEMA_MIGRATIONS_TABLE
    ))
    .bind(&migration.version)
    .execute(&mut *tx)
    .await
    .context("Failed to record migration")?;

    tx.commit().await.context("Failed to commit transaction")?;

    println!("{}", "DONE".green());

    Ok(())
}

async fn rollback_migration(pool: &PgPool, version: &str) -> Result<()> {
    print!("Rolling back {} ... ", version.white());

    let mut tx = pool.begin().await.context("Failed to begin transaction")?;

    sqlx::query(&format!(
        "DELETE FROM {} WHERE version = $1",
        SCHEMA_MIGRATIONS_TABLE
    ))
    .bind(version)
    .execute(&mut *tx)
    .await
    .context("Failed to delete migration record")?;

    tx.commit().await.context("Failed to commit transaction")?;

    println!("{}", "DONE".yellow());
    println!(
        "{}",
        "Note: Rollback does not execute down migrations - manual cleanup may be required"
            .yellow()
            .italic()
    );

    Ok(())
}

fn sanitize_migration_name(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .split('_')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_migration_name() {
        assert_eq!(sanitize_migration_name("add_users"), "add_users");
        assert_eq!(
            sanitize_migration_name("Create User Table"),
            "create_user_table"
        );
        assert_eq!(
            sanitize_migration_name("add-oauth-providers"),
            "add_oauth_providers"
        );
        assert_eq!(sanitize_migration_name("Update_V2"), "update_v2");
    }
}
