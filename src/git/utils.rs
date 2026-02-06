use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Check if the current directory is a Git repository
pub fn is_git_repo() -> bool {
    PathBuf::from(".git").exists()
}

/// Get the path to the Git hooks directory
pub fn get_hooks_path() -> PathBuf {
    PathBuf::from(".git/hooks")
}

/// Get the path to the pre-commit hook
pub fn get_precommit_hook_path() -> PathBuf {
    get_hooks_path().join("pre-commit")
}

/// Get the path to the bypass sentinel file
pub fn get_bypass_sentinel_path() -> PathBuf {
    PathBuf::from(".git/NO_DPTS_SKIP")
}

/// Get list of staged files
pub fn get_staged_files() -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .output()
        .context("Failed to execute git diff --cached --name-only")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<String> = stdout
        .lines()
        .filter(|line| !line.is_empty())
        .map(|s| s.to_string())
        .collect();

    Ok(files)
}

/// Get the staged diff for AI review
pub fn get_staged_diff() -> Result<String> {
    let output = Command::new("git")
        .args(["diff", "--cached"])
        .output()
        .context("Failed to execute git diff --cached")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Read file content for a staged file
pub fn read_staged_file_content(file_path: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["show", &format!(":{}", file_path)])
        .output()
        .context(format!("Failed to read staged content of {}", file_path))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git command failed: {}", stderr);
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
