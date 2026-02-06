use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use governor::{Quota, RateLimiter, clock::DefaultClock, state::{InMemoryState, NotKeyed}};
use std::num::NonZeroU32;

use crate::config::Config;

/// AI Review result
#[derive(Debug, Clone)]
pub struct ReviewResult {
    pub passed: bool,
    pub feedback: String,
    pub raw_response: String,
}

/// Groq API request structure
#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

/// Groq API response structure
#[derive(Debug, Deserialize)]
struct GroqResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Debug, Deserialize)]
struct MessageContent {
    content: String,
}

const REVIEW_PROMPT: &str = r#"Act as a Senior Code Reviewer with expertise in security and best practices.

Analyze the following Git diff for:
1. Logic bugs or errors
2. Security vulnerabilities (SQL injection, XSS, auth issues, etc.)
3. Code smells (dead code, duplications, poor naming, etc.)
4. Performance issues
5. Best practice violations

IMPORTANT: Your response MUST start with exactly one of these lines:
- "RESULT: PASS" if the code is acceptable (may have minor suggestions)
- "RESULT: REJECT" if the code has critical issues that must be fixed

After the RESULT line, provide a brief explanation of your findings.

Here is the diff to review:

```diff
{diff}
```"#;

/// Create a rate limiter for the API
fn create_rate_limiter(requests_per_minute: u32) -> Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>> {
    let quota = Quota::per_minute(NonZeroU32::new(requests_per_minute).unwrap_or(NonZeroU32::new(30).unwrap()));
    Arc::new(RateLimiter::direct(quota))
}

/// Review code diff using Groq API
pub async fn review_diff(diff: &str, config: &Config) -> Result<ReviewResult> {
    // Check for API key
    let api_key = std::env::var("GROQ_API_KEY")
        .context("GROQ_API_KEY environment variable not set. Please add it to your .env file.")?;
    
    if diff.trim().is_empty() {
        return Ok(ReviewResult {
            passed: true,
            feedback: "No changes to review.".to_string(),
            raw_response: String::new(),
        });
    }
    
    // Truncate diff if too large (Groq has token limits)
    let max_diff_chars = 15000;
    let truncated_diff = if diff.len() > max_diff_chars {
        format!(
            "{}\n\n... [diff truncated, {} characters omitted] ...",
            &diff[..max_diff_chars],
            diff.len() - max_diff_chars
        )
    } else {
        diff.to_string()
    };
    
    // Create rate limiter
    let rate_limiter = create_rate_limiter(config.get_rate_limit());
    
    // Wait for rate limit
    rate_limiter.until_ready().await;
    
    // Build the request
    let prompt = REVIEW_PROMPT.replace("{diff}", &truncated_diff);
    
    let request = GroqRequest {
        model: config.ai_model.clone(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        temperature: 0.3,
        max_tokens: 1024,
    };
    
    // Make the API call
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;
    
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to connect to Groq API")?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Groq API error ({}): {}", status, error_text);
    }
    
    let groq_response: GroqResponse = response
        .json()
        .await
        .context("Failed to parse Groq API response")?;
    
    let content = groq_response
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response from AI".to_string());
    
    // Parse the result
    let passed = content.contains("RESULT: PASS");
    let rejected = content.contains("RESULT: REJECT");
    
    // Extract feedback (everything after RESULT line)
    let feedback = content
        .lines()
        .skip_while(|line| !line.starts_with("RESULT:"))
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string();
    
    Ok(ReviewResult {
        passed: passed && !rejected,
        feedback: if feedback.is_empty() { content.clone() } else { feedback },
        raw_response: content,
    })
}

/// Print review result in a formatted way
pub fn print_result(result: &ReviewResult) {
    println!();
    if result.passed {
        println!("{}", "🤖 AI Review: PASSED".green().bold());
    } else {
        println!("{}", "🤖 AI Review: REJECTED".red().bold());
    }
    println!("{}", "─".repeat(60));
    
    // Print feedback with nice formatting
    for line in result.feedback.lines() {
        if line.trim().is_empty() {
            println!();
        } else if line.starts_with('-') || line.starts_with('*') || line.starts_with("•") {
            println!("  {}", line.white());
        } else if line.contains(':') && line.len() < 50 {
            println!("  {}", line.cyan());
        } else {
            println!("  {}", line.dimmed());
        }
    }
    println!("{}", "─".repeat(60));
}
