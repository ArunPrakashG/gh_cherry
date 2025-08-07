use anyhow::Result;
use clap::Parser;

mod auth;
mod config;
mod git;
mod github;
mod ui;

use config::Config;
use ui::app::App;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// GitHub repository owner
    #[arg(short, long)]
    owner: Option<String>,

    /// GitHub repository name
    #[arg(short, long)]
    repo: Option<String>,

    /// Path to configuration file
    #[arg(short, long)]
    config: Option<String>,

    /// Base branch to cherry-pick from
    #[arg(short, long)]
    base_branch: Option<String>,

    /// Target branch to cherry-pick to
    #[arg(short, long)]
    target_branch: Option<String>,

    /// Number of days to look back for PRs
    #[arg(short, long)]
    days: Option<u32>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration
    let config = Config::load(cli.config.as_deref())?;

    // Override config with CLI arguments
    let config = config.with_overrides(
        cli.owner,
        cli.repo,
        cli.base_branch,
        cli.target_branch,
        cli.days,
    );

    // Create and run the TUI application
    let mut app = App::new(config).await?;
    app.run().await?;

    Ok(())
}
