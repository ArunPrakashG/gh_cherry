use anyhow::Result;
use clap::Parser;

mod auth;
mod config;
mod git;
mod github;
mod ui;

use config::Config;
use github::GitHubClient;
use ui::app::App;
use ui::config_selector::ConfigSelectorApp;
use ui::selector::SelectorApp;

#[derive(Parser)]
#[command(author, version, about = "A TUI application for cherry-picking GitHub PRs to target branches. Auto-discovers organizations and repositories when not specified.", long_about = None)]
struct Cli {
    /// GitHub repository owner (auto-discovered if not provided)
    #[arg(short, long)]
    owner: Option<String>,

    /// GitHub repository name (auto-discovered if not provided)
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

    /// Only show forked repositories in selection
    #[arg(long)]
    only_forks: bool,

    /// Source branch to create cherry-pick branch from
    #[arg(long)]
    source_branch: Option<String>,

    /// Task ID for branch naming
    #[arg(long)]
    task_id: Option<String>,

    /// Save current settings to cherry.env file
    #[arg(long)]
    save_config: bool,

    /// Skip interactive configuration loading prompt
    #[arg(long)]
    no_prompt: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration with optional interactive prompt
    let mut config = if cli.no_prompt {
        Config::load(cli.config.as_deref())?
    } else {
        Config::load_with_prompt(cli.config.as_deref())?
    };

    // Override config with CLI arguments
    config = config.with_overrides(
        cli.owner,
        cli.repo,
        cli.base_branch,
        cli.target_branch,
        cli.days,
        if cli.only_forks { Some(true) } else { None },
        cli.source_branch,
    );

    // Handle task ID for branch naming
    if let Some(task_id) = cli.task_id {
        // Replace {task_id} placeholder in branch name template
        config.github.branch_name_template = config
            .github
            .branch_name_template
            .replace("{task_id}", &task_id);
    } else {
        // If no task ID provided, prompt user for it
        if config.github.branch_name_template.contains("{task_id}") {
            let task_id =
                ConfigSelectorApp::get_task_id_input(&config.github.branch_name_template)?;
            config.github.branch_name_template = config
                .github
                .branch_name_template
                .replace("{task_id}", &task_id);
        }
    }

    // Handle auto-discovery if needed
    if config.needs_auto_discovery() {
        println!("No owner/repo specified, discovering available options...");
        config = handle_auto_discovery(config).await?;
    }

    // If source branch is still default or not set, prompt user for customization
    if config.github.cherry_pick_source_branch == "develop"
        || config.github.cherry_pick_source_branch.is_empty()
    {
        println!(
            "Current source branch for cherry-pick: {}",
            config.github.cherry_pick_source_branch
        );
        println!("Press Enter to use current, or type a different branch:");
        use std::io::{self, Write};
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if !input.is_empty() {
            config.github.cherry_pick_source_branch = input.to_string();
        }
    }

    // Validate final configuration
    config.validate()?;

    // Save config to cherry.env if requested
    if cli.save_config {
        config.save_env_overrides()?;
        println!("Configuration saved to cherry.env");
    }

    // Create and run the TUI application
    let mut app = App::new(config).await?;
    app.run().await?;

    Ok(())
}

async fn handle_auto_discovery(mut config: Config) -> Result<Config> {
    // Create a temporary GitHub client for discovery
    let github_client = GitHubClient::new(config.clone()).await?;

    // Fetch user info for context
    let user = github_client.get_authenticated_user().await?;
    println!("Authenticated as: {} ({})", user.name, user.login);

    // If no owner specified, try to discover
    if config.github.owner.is_empty() {
        let orgs = github_client.list_user_organizations().await?;

        if orgs.is_empty() {
            // Only user account available
            config.github.owner = user.login.clone();
            println!("Using owner: {}", config.github.owner);
        } else {
            // Multiple options available - use TUI selector
            println!("Opening organization selector...");
            config.github.owner = SelectorApp::run_organization_selector(&user.login, &orgs)?;
            println!("Selected owner: {}", config.github.owner);
        }
    }

    // If no repo specified, try to find repos for the owner
    if config.github.repo.is_empty() {
        let repos = github_client.list_user_repositories().await?;

        // Filter repos by owner and fork preference
        let owner_repos: Vec<_> = repos
            .iter()
            .filter(|r| r.owner == config.github.owner && (!config.ui.only_forked_repos || r.fork))
            .cloned()
            .collect();

        if owner_repos.is_empty() {
            let filter_msg = if config.ui.only_forked_repos {
                " (forked repositories only)"
            } else {
                ""
            };
            anyhow::bail!(
                "No repositories found for owner: {}{}",
                config.github.owner,
                filter_msg
            );
        } else if owner_repos.len() == 1 {
            // Only one repo available
            config.github.repo = owner_repos[0].name.clone();
            println!("Using repository: {}", config.github.repo);
        } else {
            // Multiple repos available - use TUI selector
            println!("Opening repository selector...");
            config.github.repo = SelectorApp::run_repository_selector(&owner_repos)?;
            println!("Selected repository: {}", config.github.repo);
        }
    }

    Ok(config)
}
