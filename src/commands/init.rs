use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::io::Write;

use crate::git;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const PRECOMMIT_HOOK_CONTENT: &str = r#"#!/bin/sh
# no-dpts-tool pre-commit hook
# This hook is installed by no-dpts-tool and runs security, linting, and AI checks

# Run no-dpts-tool check
no-dpts-tool check

# Capture the exit code
exit_code=$?

# If the check failed, block the commit
if [ $exit_code -ne 0 ]; then
    echo ""
    echo "Commit blocked by no-dpts-tool."
    echo "Fix the issues above or run 'no-dpts-tool bypass' to skip checks once."
    exit 1
fi

exit 0
"#;

pub async fn run() -> Result<()> {
    println!("{}", "🔧 Initializing no-dpts-tool...".cyan().bold());
    println!();

    // Step 1: Verify we're in a Git repository
    if !git::is_git_repo() {
        anyhow::bail!(
            "Not a Git repository. Please run this command from the root of a Git repository."
        );
    }
    println!("{} Git repository detected", "✓".green());

    // Step 2: Ensure hooks directory exists
    let hooks_path = git::get_hooks_path();
    if !hooks_path.exists() {
        fs::create_dir_all(&hooks_path)
            .context("Failed to create .git/hooks directory")?;
        println!("{} Created hooks directory", "✓".green());
    }

    // Step 3: Create the pre-commit hook
    let precommit_path = git::get_precommit_hook_path();
    let mut file = fs::File::create(&precommit_path)
        .context("Failed to create pre-commit hook file")?;
    
    file.write_all(PRECOMMIT_HOOK_CONTENT.as_bytes())
        .context("Failed to write pre-commit hook content")?;
    
    println!("{} Created pre-commit hook", "✓".green());

    // Step 4: Make the hook executable (Unix only, Windows Git handles this differently)
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&precommit_path)?.permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&precommit_path, permissions)
            .context("Failed to set executable permissions on pre-commit hook")?;
        println!("{} Set executable permissions", "✓".green());
    }

    #[cfg(windows)]
    {
        println!("{} Pre-commit hook created (executable via Git Bash)", "✓".green());
    }

    // Step 5: Create example config file if it doesn't exist
    let config_path = std::path::PathBuf::from("no-dpts.toml");
    if !config_path.exists() {
        let example_config = r#"# no-dpts-tool configuration file

# Files to ignore during scanning (supports glob patterns)
ignored_files = [
    "*.lock",
    "*.min.js",
    "*.min.css",
    "package-lock.json",
    "yarn.lock",
]

# Custom regex patterns for project-specific secrets
# These are in addition to the built-in patterns
custom_patterns = [
    # "MY_SECRET_[A-Z0-9]{32}"
]

# AI model to use for code review (Groq models)
ai_model = "llama-3.3-70b-versatile"

# Rate limiting for AI API calls
[rate_limit]
requests_per_minute = 30
"#;
        fs::write(&config_path, example_config)
            .context("Failed to create example config file")?;
        println!("{} Created example no-dpts.toml config", "✓".green());
    }

    println!();
    println!("{}", "✅ no-dpts-tool initialized successfully!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Set your {} in a .env file", "GROQ_API_KEY".yellow());
    println!("  2. Customize {} as needed", "no-dpts.toml".yellow());
    println!("  3. Stage your changes and commit - checks will run automatically!");
    println!();
    println!("To bypass checks once: {}", "no-dpts-tool bypass".cyan());

    Ok(())
}
