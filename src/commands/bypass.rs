use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

use crate::git;

pub async fn run() -> Result<()> {
    println!("{}", "⚡ Creating bypass token...".yellow().bold());
    println!();

    // Verify we're in a Git repository
    if !git::is_git_repo() {
        anyhow::bail!(
            "Not a Git repository. Please run this command from the root of a Git repository."
        );
    }

    // Create the bypass sentinel file
    let sentinel_path = git::get_bypass_sentinel_path();
    fs::write(&sentinel_path, "BYPASS_TOKEN")
        .context("Failed to create bypass sentinel file")?;

    println!("{} Bypass token created", "✓".green());
    println!();
    println!("{}", "⚠️  Your next commit will skip all checks.".yellow().bold());
    println!();
    println!("The bypass token will be automatically deleted after one use.");
    println!("This is intended for emergency situations only.");
    println!();
    println!("{}", "Now run your commit:".cyan());
    println!("  git commit -m \"your message\"");

    Ok(())
}
