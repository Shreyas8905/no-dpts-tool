use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub ignored_files: Vec<String>,
    
    #[serde(default)]
    pub custom_patterns: Vec<String>,
    
    #[serde(default = "default_ai_model")]
    pub ai_model: String,
    
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateLimitConfig {
    #[serde(default = "default_requests_per_minute")]
    pub requests_per_minute: u32,
}

fn default_ai_model() -> String {
    "llama-3.3-70b-versatile".to_string()
}

fn default_requests_per_minute() -> u32 {
    30
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: default_requests_per_minute(),
        }
    }
}

impl Config {
    /// Load configuration from no-dpts.toml if it exists
    pub fn load() -> Result<Self> {
        let config_path = PathBuf::from("no-dpts.toml");
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = toml::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }
    
    /// Check if a file should be ignored
    pub fn should_ignore(&self, file_path: &str) -> bool {
        self.ignored_files.iter().any(|pattern| {
            if pattern.contains('*') {
                // Simple glob matching
                let regex_pattern = pattern.replace(".", r"\.").replace("*", ".*");
                regex::Regex::new(&regex_pattern)
                    .map(|re| re.is_match(file_path))
                    .unwrap_or(false)
            } else {
                file_path == pattern || file_path.ends_with(pattern)
            }
        })
    }
    
    /// Get rate limit requests per minute
    pub fn get_rate_limit(&self) -> u32 {
        self.rate_limit
            .as_ref()
            .map(|r| r.requests_per_minute)
            .unwrap_or(default_requests_per_minute())
    }
}
