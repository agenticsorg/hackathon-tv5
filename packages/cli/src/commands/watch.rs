use anyhow::{Context, Result};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;

use crate::cli;

pub async fn run(path: String, generate: bool, validate_on_change: bool) -> Result<()> {
    cli::info(&format!("Watching {} for changes...", path));

    let watch_path = PathBuf::from(&path);
    if !watch_path.exists() {
        return Err(anyhow::anyhow!("Path does not exist: {}", path));
    }

    let (tx, rx) = channel();

    let mut watcher = notify::recommended_watcher(tx)
        .context("Failed to create file watcher")?;

    watcher
        .watch(watch_path.as_ref(), RecursiveMode::Recursive)
        .context("Failed to start watching directory")?;

    cli::success("Watch mode active. Press Ctrl+C to stop.");
    println!("\nOptions:");
    if generate {
        println!("  ✓ Auto-generate machine views on HTML changes");
    }
    if validate_on_change {
        println!("  ✓ Auto-validate on llms.txt changes");
    }
    println!();

    loop {
        match rx.recv() {
            Ok(Ok(event)) => {
                if should_process_event(&event) {
                    handle_file_change(&event, &path, generate, validate_on_change).await?;
                }
            }
            Ok(Err(e)) => {
                cli::warn(&format!("Watch error: {}", e));
            }
            Err(e) => {
                cli::error(&format!("Channel error: {}", e));
                break;
            }
        }
    }

    Ok(())
}

fn should_process_event(event: &Event) -> bool {
    matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    )
}

async fn handle_file_change(
    event: &Event,
    _base_path: &str,
    generate: bool,
    validate: bool,
) -> Result<()> {
    for path in &event.paths {
        let file_path = path.to_string_lossy().to_string();

        // Check if it's a relevant file
        if file_path.ends_with(".html") && generate {
            cli::info(&format!("Detected change: {}", file_path));
            regenerate_machine_view(&file_path).await?;
        } else if file_path.ends_with("llms.txt") && validate {
            cli::info(&format!("Detected change: {}", file_path));
            validate_manifest(&file_path).await?;
        } else if file_path.ends_with(".llm.md") {
            cli::info(&format!("Machine view updated: {}", file_path));
        }
    }

    Ok(())
}

async fn regenerate_machine_view(html_path: &str) -> Result<()> {
    // Determine output path
    let output_path = html_path.replace(".html", ".llm.md");

    println!("  → Regenerating {}", output_path);

    // Call generate command
    match crate::commands::generate::run(
        html_path.to_string(),
        Some(output_path.clone()),
        false,
        "html".to_string(),
        false,
        "markdown".to_string(), // Default to markdown format for watch mode
        false, // Don't sync in watch mode
    )
    .await
    {
        Ok(()) => {
            cli::success(&format!("  ✓ Generated {}", output_path));
        }
        Err(e) => {
            cli::error(&format!("  ✗ Failed to generate: {}", e));
        }
    }

    Ok(())
}

async fn validate_manifest(manifest_path: &str) -> Result<()> {
    println!("  → Validating manifest...");

    let path = Path::new(manifest_path);
    match crate::validators::llms_txt::validate(path) {
        Ok(errors) => {
            if errors.is_empty() {
                cli::success("  ✓ Manifest is valid");
            } else {
                cli::error(&format!("  ✗ Found {} validation errors:", errors.len()));
                for error in errors.iter().take(5) {
                    println!("    • {}", error);
                }
                if errors.len() > 5 {
                    println!("    ... and {} more", errors.len() - 5);
                }
            }
        }
        Err(e) => {
            cli::error(&format!("  ✗ Validation failed: {}", e));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_process_event() {
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Any,
            )),
            paths: vec![PathBuf::from("test.txt")],
            attrs: Default::default(),
        };

        assert!(should_process_event(&event));
    }
}
