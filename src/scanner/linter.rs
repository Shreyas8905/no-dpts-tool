use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::process::Command;

/// Represents a linter result
#[derive(Debug, Clone)]
pub struct LinterResult {
    pub tool: String,
    pub file: String,
    pub passed: bool,
    pub output: String,
    pub skipped: bool,
    pub skip_reason: Option<String>,
}

/// Map file extensions to linter commands
fn get_linter_config() -> HashMap<&'static str, (&'static str, Vec<&'static str>)> {
    let mut config = HashMap::new();
    
    // Python - ruff
    config.insert("py", ("ruff", vec!["check"]));
    
    // JavaScript/TypeScript - eslint
    config.insert("js", ("eslint", vec!["--no-error-on-unmatched-pattern"]));
    config.insert("jsx", ("eslint", vec!["--no-error-on-unmatched-pattern"]));
    config.insert("ts", ("eslint", vec!["--no-error-on-unmatched-pattern"]));
    config.insert("tsx", ("eslint", vec!["--no-error-on-unmatched-pattern"]));
    
    // Rust - cargo fmt (check mode)
    config.insert("rs", ("cargo", vec!["fmt", "--check", "--"]));
    
    config
}

/// Check if a command is available on the system
fn is_command_available(cmd: &str) -> bool {
    if cfg!(windows) {
        Command::new("where")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Run linter on a file
pub async fn run_linter(file_path: &str) -> Result<LinterResult> {
    // Get file extension
    let extension = file_path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    
    let linter_config = get_linter_config();
    
    // Check if we have a linter for this file type
    let Some((cmd, args)) = linter_config.get(extension.as_str()) else {
        return Ok(LinterResult {
            tool: "none".to_string(),
            file: file_path.to_string(),
            passed: true,
            output: String::new(),
            skipped: true,
            skip_reason: Some(format!("No linter configured for .{} files", extension)),
        });
    };
    
    // Check if the linter is installed
    if !is_command_available(cmd) {
        return Ok(LinterResult {
            tool: cmd.to_string(),
            file: file_path.to_string(),
            passed: true,
            output: String::new(),
            skipped: true,
            skip_reason: Some(format!("{} is not installed", cmd)),
        });
    }
    
    // Build the command
    let mut command = Command::new(cmd);
    for arg in args {
        command.arg(arg);
    }
    command.arg(file_path);
    
    // Run the linter
    let output = command.output();
    
    match output {
        Ok(result) => {
            let stdout = String::from_utf8_lossy(&result.stdout);
            let stderr = String::from_utf8_lossy(&result.stderr);
            let combined_output = format!("{}{}", stdout, stderr);
            
            Ok(LinterResult {
                tool: cmd.to_string(),
                file: file_path.to_string(),
                passed: result.status.success(),
                output: combined_output.trim().to_string(),
                skipped: false,
                skip_reason: None,
            })
        }
        Err(e) => {
            Ok(LinterResult {
                tool: cmd.to_string(),
                file: file_path.to_string(),
                passed: true,
                output: String::new(),
                skipped: true,
                skip_reason: Some(format!("Failed to run {}: {}", cmd, e)),
            })
        }
    }
}

/// Run linters on multiple files in parallel
pub async fn run_linters(files: &[String]) -> Vec<LinterResult> {
    let mut handles = Vec::new();
    
    for file in files {
        let file = file.clone();
        handles.push(tokio::spawn(async move {
            run_linter(&file).await
        }));
    }
    
    let mut results = Vec::new();
    for handle in handles {
        if let Ok(Ok(result)) = handle.await {
            results.push(result);
        }
    }
    
    results
}

/// Print linter results in a formatted way
pub fn print_results(results: &[LinterResult]) {
    let failures: Vec<_> = results.iter().filter(|r| !r.passed && !r.skipped).collect();
    let skipped: Vec<_> = results.iter().filter(|r| r.skipped).collect();
    
    if !failures.is_empty() {
        println!();
        println!("{}", "🔍 Linting Failures:".red().bold());
        println!("{}", "─".repeat(60));
        
        for result in &failures {
            println!(
                "  {} {} ({})",
                "✗".red(),
                result.file.white(),
                result.tool.cyan()
            );
            if !result.output.is_empty() {
                for line in result.output.lines().take(10) {
                    println!("    {}", line.dimmed());
                }
                let line_count = result.output.lines().count();
                if line_count > 10 {
                    println!("    {} more lines...", format!("... {} ", line_count - 10).dimmed());
                }
            }
        }
        println!("{}", "─".repeat(60));
    }
    
    // Print skip warnings (only in verbose mode or if there are important skips)
    let important_skips: Vec<_> = skipped.iter()
        .filter(|r| r.skip_reason.as_ref().map(|s| s.contains("not installed")).unwrap_or(false))
        .collect();
    
    if !important_skips.is_empty() {
        println!();
        println!("{}", "⚠️  Linter Warnings:".yellow());
        for result in important_skips {
            if let Some(reason) = &result.skip_reason {
                println!("  {} {}", "⚠".yellow(), reason.dimmed());
            }
        }
    }
}
