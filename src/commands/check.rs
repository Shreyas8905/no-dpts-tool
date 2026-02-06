use anyhow::Result;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::time::Duration;

use crate::ai::reviewer;
use crate::config::Config;
use crate::git;
use crate::scanner::{linter, security};

/// Check result summary
struct CheckSummary {
    security_passed: bool,
    linting_passed: bool,
    ai_passed: bool,
    security_findings: Vec<security::SecurityFinding>,
    linter_results: Vec<linter::LinterResult>,
    ai_result: Option<reviewer::ReviewResult>,
    bypassed: bool,
}

pub async fn run() -> Result<()> {
    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════╗".cyan());
    println!("{}", "║           🛡️  no-dpts-tool Pre-Commit Check              ║".cyan());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".cyan());
    println!();

    // Check for bypass sentinel
    let bypass_path = git::get_bypass_sentinel_path();
    if bypass_path.exists() {
        fs::remove_file(&bypass_path)?;
        println!("{}", "⚡ Bypass token detected - skipping all checks".yellow().bold());
        println!("{}", "   This is a one-time bypass. Future commits will be checked.".dimmed());
        println!();
        return Ok(());
    }

    // Load configuration
    let config = Config::load().unwrap_or_default();

    // Get staged files
    let spinner = create_spinner("Detecting staged files...");
    let staged_files = git::get_staged_files()?;
    spinner.finish_with_message(format!("{} Found {} staged file(s)", "✓".green(), staged_files.len()));

    if staged_files.is_empty() {
        println!();
        println!("{}", "No staged files to check.".dimmed());
        return Ok(());
    }

    // Filter out ignored files
    let files_to_check: Vec<String> = staged_files
        .iter()
        .filter(|f| !config.should_ignore(f))
        .cloned()
        .collect();

    let ignored_count = staged_files.len() - files_to_check.len();
    if ignored_count > 0 {
        println!("  {} {} file(s) ignored per config", "↳".dimmed(), ignored_count);
    }

    println!();

    // Run all checks in parallel
    let (security_result, linting_result, ai_result) = tokio::join!(
        run_security_check(&files_to_check, &config),
        run_linting_check(&files_to_check),
        run_ai_review(&config)
    );

    // Collect results
    let summary = CheckSummary {
        security_passed: security_result.as_ref().map(|f| f.is_empty()).unwrap_or(true),
        linting_passed: linting_result.as_ref().map(|r| r.iter().all(|x| x.passed || x.skipped)).unwrap_or(true),
        ai_passed: ai_result.as_ref().map(|r| r.passed).unwrap_or(true),
        security_findings: security_result.unwrap_or_default(),
        linter_results: linting_result.unwrap_or_default(),
        ai_result,
        bypassed: false,
    };

    // Print detailed results
    print_summary(&summary);

    // Exit with appropriate code
    if !summary.security_passed || !summary.linting_passed || !summary.ai_passed {
        println!();
        println!("{}", "╔══════════════════════════════════════════════════════════╗".red());
        println!("{}", "║              ❌ COMMIT BLOCKED                           ║".red());
        println!("{}", "╠══════════════════════════════════════════════════════════╣".red());
        println!("{}", "║  Fix the issues above, or run:                           ║".red());
        println!("{}", "║  no-dpts-tool bypass  (emergency skip, use sparingly)    ║".red());
        println!("{}", "╚══════════════════════════════════════════════════════════╝".red());
        println!();
        std::process::exit(1);
    }

    println!();
    println!("{}", "╔══════════════════════════════════════════════════════════╗".green());
    println!("{}", "║              ✅ ALL CHECKS PASSED                        ║".green());
    println!("{}", "╚══════════════════════════════════════════════════════════╝".green());
    println!();

    Ok(())
}

/// Run security scan on staged files
async fn run_security_check(files: &[String], config: &Config) -> Result<Vec<security::SecurityFinding>> {
    let spinner = create_spinner("Running security scan...");
    
    let mut all_findings = Vec::new();
    
    for file in files {
        // Read the staged content of the file
        match git::read_staged_file_content(file) {
            Ok(content) => {
                match security::scan_content(file, &content, config) {
                    Ok(findings) => all_findings.extend(findings),
                    Err(e) => eprintln!("{} Error scanning {}: {}", "⚠".yellow(), file, e),
                }
            }
            Err(e) => {
                // File might be deleted or binary
                if !e.to_string().contains("fatal") {
                    eprintln!("{} Could not read {}: {}", "⚠".yellow(), file, e);
                }
            }
        }
    }
    
    let high_severity = all_findings.iter().filter(|f| f.severity == security::Severity::High).count();
    let medium_severity = all_findings.iter().filter(|f| f.severity == security::Severity::Medium).count();
    
    if all_findings.is_empty() {
        spinner.finish_with_message(format!("{} Security scan passed", "✓".green()));
    } else {
        spinner.finish_with_message(format!(
            "{} Security scan found {} issue(s) ({} high, {} medium)",
            "✗".red(),
            all_findings.len(),
            high_severity,
            medium_severity
        ));
    }
    
    Ok(all_findings)
}

/// Run linting on staged files
async fn run_linting_check(files: &[String]) -> Result<Vec<linter::LinterResult>> {
    let spinner = create_spinner("Running linters...");
    
    let results = linter::run_linters(files).await;
    
    let failed_count = results.iter().filter(|r| !r.passed && !r.skipped).count();
    let checked_count = results.iter().filter(|r| !r.skipped).count();
    
    if failed_count == 0 {
        spinner.finish_with_message(format!("{} Linting passed ({} files checked)", "✓".green(), checked_count));
    } else {
        spinner.finish_with_message(format!("{} Linting failed ({}/{} files)", "✗".red(), failed_count, checked_count));
    }
    
    Ok(results)
}

/// Run AI review on staged diff
async fn run_ai_review(config: &Config) -> Option<reviewer::ReviewResult> {
    let spinner = create_spinner("Running AI review...");
    
    // Get the staged diff
    let diff = match git::get_staged_diff() {
        Ok(d) => d,
        Err(e) => {
            spinner.finish_with_message(format!("{} AI review skipped: {}", "⚠".yellow(), e));
            return None;
        }
    };
    
    if diff.trim().is_empty() {
        spinner.finish_with_message(format!("{} AI review skipped: no diff", "⚠".yellow()));
        return None;
    }
    
    // Run the review
    match reviewer::review_diff(&diff, config).await {
        Ok(result) => {
            if result.passed {
                spinner.finish_with_message(format!("{} AI review passed", "✓".green()));
            } else {
                spinner.finish_with_message(format!("{} AI review: changes rejected", "✗".red()));
            }
            Some(result)
        }
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("GROQ_API_KEY") {
                spinner.finish_with_message(format!("{} AI review skipped: API key not set", "⚠".yellow()));
            } else {
                spinner.finish_with_message(format!("{} AI review error: {}", "⚠".yellow(), error_msg));
            }
            None
        }
    }
}

/// Print the check summary
fn print_summary(summary: &CheckSummary) {
    // Print security findings
    if !summary.security_findings.is_empty() {
        security::print_findings(&summary.security_findings);
    }
    
    // Print linting results
    let linter_failures: Vec<_> = summary.linter_results.iter()
        .filter(|r| !r.passed && !r.skipped)
        .collect();
    
    if !linter_failures.is_empty() {
        linter::print_results(&summary.linter_results);
    }
    
    // Print AI review result
    if let Some(ref ai_result) = summary.ai_result {
        if !ai_result.passed {
            reviewer::print_result(ai_result);
        }
    }
}

/// Create a styled spinner
fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner
}
