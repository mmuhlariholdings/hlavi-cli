use crate::commands::{get_cwd, get_storage};
use colored::*;
use hlavi_core::storage::Storage;

pub async fn execute() -> anyhow::Result<()> {
    let storage = get_storage()?;
    let cwd = get_cwd()?;

    // Check if already initialized
    if storage.is_initialized().await {
        println!(
            "{} Hlavi project already initialized in {}",
            "✓".green().bold(),
            cwd.display()
        );
        return Ok(());
    }

    // Initialize the storage
    storage.initialize().await?;

    println!(
        "{} Initialized Hlavi project in {}",
        "✓".green().bold(),
        cwd.display()
    );
    println!("\nCreated:");
    println!("  {} .hlavi/", "→".blue());
    println!("  {} .hlavi/board.json", "→".blue());
    println!("  {} .hlavi/tickets/", "→".blue());
    println!("\nYou can now create tickets with:");
    println!(
        "  {} hlavi tickets create \"Your ticket title\"",
        "$".yellow()
    );

    Ok(())
}
