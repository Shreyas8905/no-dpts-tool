use anyhow::Result;
use colored::Colorize;
use regex::Regex;
use std::collections::HashMap;

use crate::config::Config;

/// Represents a security finding
#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub file: String,
    pub line_number: usize,
    pub pattern_name: String,
    pub matched_text: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Severity {
    High,
    Medium,
    Low,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::High => write!(f, "{}", "HIGH".red().bold()),
            Severity::Medium => write!(f, "{}", "MEDIUM".yellow().bold()),
            Severity::Low => write!(f, "{}", "LOW".blue()),
        }
    }
}

/// Built-in security patterns
fn get_builtin_patterns() -> Vec<(&'static str, &'static str, Severity)> {
    vec![
        // AWS Keys
        ("AWS Access Key ID", r"AKIA[0-9A-Z]{16}", Severity::High),
        ("AWS Secret Key", r#"(?i)aws(.{0,20})?['"][0-9a-zA-Z/+]{40}['"]"#, Severity::High),
        
        // Google API Keys
        ("Google API Key", r"AIza[0-9A-Za-z\-_]{35}", Severity::High),
        ("Google OAuth", r"[0-9]+-[0-9A-Za-z_]{32}\.apps\.googleusercontent\.com", Severity::High),
        
        // GitHub Tokens
        ("GitHub Token", r"gh[pousr]_[A-Za-z0-9_]{36}", Severity::High),
        ("GitHub Personal Access Token", r"github_pat_[a-zA-Z0-9]{22}_[a-zA-Z0-9]{59}", Severity::High),
        
        // Generic API Keys
        ("Generic API Key", r#"(?i)(api[_-]?key|apikey)['":\s]*[=:]\s*['"][a-zA-Z0-9]{20,}['"]"#, Severity::Medium),
        ("Generic Secret", r#"(?i)(secret|token)['":\s]*[=:]\s*['"][a-zA-Z0-9]{16,}['"]"#, Severity::Medium),
        
        // Passwords
        ("Hardcoded Password", r#"(?i)(password|passwd|pwd)['":\s]*[=:]\s*['"][^'"]{8,}['"]"#, Severity::High),
        
        // Private Keys
        ("RSA Private Key", r"-----BEGIN RSA PRIVATE KEY-----", Severity::High),
        ("Private Key", r"-----BEGIN PRIVATE KEY-----", Severity::High),
        ("EC Private Key", r"-----BEGIN EC PRIVATE KEY-----", Severity::High),
        
        // JWT Tokens
        ("JWT Token", r"eyJ[A-Za-z0-9-_=]+\.eyJ[A-Za-z0-9-_=]+\.[A-Za-z0-9-_.+/=]*", Severity::Medium),
        
        // Slack Tokens
        ("Slack Token", r"xox[baprs]-[0-9]{10,13}-[0-9]{10,13}[a-zA-Z0-9-]*", Severity::High),
        
        // Database URLs with credentials
        ("Database URL with Credentials", r"(?i)(postgres|mysql|mongodb)://[^:]+:[^@]+@", Severity::High),
        
        // Absolute local paths (potential leak of system info)
        ("Absolute Path (Unix)", r#"/Users/[a-zA-Z0-9_-]+/"#, Severity::Low),
        ("Absolute Path (Unix Home)", r#"/home/[a-zA-Z0-9_-]+/"#, Severity::Low),
        ("Absolute Path (Windows)", r#"[Cc]:\\Users\\[a-zA-Z0-9_-]+\\"#, Severity::Low),
        
        // Bearer tokens in code
        ("Bearer Token", r#"(?i)bearer\s+[a-zA-Z0-9\-_.~+/]+=*"#, Severity::Medium),
        
        // npm tokens
        ("NPM Token", r"//registry\.npmjs\.org/:_authToken=.+", Severity::High),
        
        // Stripe Keys
        ("Stripe API Key", r"sk_live_[0-9a-zA-Z]{24}", Severity::High),
        ("Stripe Publishable Key", r"pk_live_[0-9a-zA-Z]{24}", Severity::Medium),
    ]
}

/// Compile patterns into regex objects
fn compile_patterns(config: &Config) -> Result<HashMap<String, (Regex, Severity)>> {
    let mut patterns = HashMap::new();
    
    // Add built-in patterns
    for (name, pattern, severity) in get_builtin_patterns() {
        match Regex::new(pattern) {
            Ok(regex) => {
                patterns.insert(name.to_string(), (regex, severity));
            }
            Err(e) => {
                eprintln!("{} Failed to compile pattern '{}': {}", "⚠".yellow(), name, e);
            }
        }
    }
    
    // Add custom patterns from config
    for (index, custom_pattern) in config.custom_patterns.iter().enumerate() {
        let name = format!("Custom Pattern #{}", index + 1);
        match Regex::new(custom_pattern) {
            Ok(regex) => {
                patterns.insert(name, (regex, Severity::Medium));
            }
            Err(e) => {
                eprintln!("{} Invalid custom pattern '{}': {}", "⚠".yellow(), custom_pattern, e);
            }
        }
    }
    
    Ok(patterns)
}

/// Scan a file's content for security issues
pub fn scan_content(
    file_path: &str,
    content: &str,
    config: &Config,
) -> Result<Vec<SecurityFinding>> {
    let patterns = compile_patterns(config)?;
    let mut findings = Vec::new();
    
    for (line_number, line) in content.lines().enumerate() {
        for (pattern_name, (regex, severity)) in &patterns {
            if let Some(matched) = regex.find(line) {
                // Mask sensitive part of the match for display
                let matched_text = matched.as_str();
                let masked = if matched_text.len() > 10 {
                    format!("{}...{}", &matched_text[..5], &matched_text[matched_text.len()-3..])
                } else {
                    matched_text.to_string()
                };
                
                findings.push(SecurityFinding {
                    file: file_path.to_string(),
                    line_number: line_number + 1,
                    pattern_name: pattern_name.clone(),
                    matched_text: masked,
                    severity: *severity,
                });
            }
        }
    }
    
    Ok(findings)
}

/// Print security findings in a formatted way
pub fn print_findings(findings: &[SecurityFinding]) {
    if findings.is_empty() {
        return;
    }
    
    println!();
    println!("{}", "🔐 Security Findings:".red().bold());
    println!("{}", "─".repeat(60));
    
    for finding in findings {
        println!(
            "  {} [{}] {}:{} - {}",
            finding.severity,
            finding.pattern_name.cyan(),
            finding.file.white(),
            finding.line_number.to_string().yellow(),
            finding.matched_text.dimmed()
        );
    }
    println!("{}", "─".repeat(60));
}
