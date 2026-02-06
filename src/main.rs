use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

mod ai;
mod commands;
mod config;
mod git;
mod scanner;

#[derive(Parser)]
#[command(name = "no-dpts-tool")]
#[command(author = "Your Team")]
#[command(version = "0.1.0")]
#[command(about = "A high-performance Git-integrated CLI gatekeeper", long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize no-dpts-tool in the current Git repository
    Init,
    /// Run all checks on staged files (called by pre-commit hook)
    Check,
    /// Bypass checks for the next commit (creates a one-time skip token)
    Bypass,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::run().await,
        Commands::Check => commands::check::run().await,
        Commands::Bypass => commands::bypass::run().await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }

    Ok(())
}
