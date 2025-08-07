use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};

mod auth;
mod config;
mod git;
mod github;
mod ui;

use config::Config;
use github::GitHubClient;
use ui::app::App;

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
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load configuration
    let mut config = Config::load(cli.config.as_deref())?;

    // Override config with CLI arguments
    config = config.with_overrides(
        cli.owner,
        cli.repo,
        cli.base_branch,
        cli.target_branch,
        cli.days,
    );

    // Handle auto-discovery if needed
    if config.needs_auto_discovery() {
        println!("No owner/repo specified, discovering available options...");
        config = handle_auto_discovery(config).await?;
    }

    // Validate final configuration
    config.validate()?;

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
            // Multiple options available - let user choose interactively
            config.github.owner = select_organization(&user.login, &orgs)?;
            println!("Selected owner: {}", config.github.owner);
        }
    }

    // If no repo specified, try to find repos for the owner
    if config.github.repo.is_empty() {
        let repos = github_client.list_user_repositories().await?;

        // Filter repos by owner
        let owner_repos: Vec<_> = repos
            .iter()
            .filter(|r| r.owner == config.github.owner)
            .cloned()
            .collect();

        if owner_repos.is_empty() {
            anyhow::bail!("No repositories found for owner: {}", config.github.owner);
        } else if owner_repos.len() == 1 {
            // Only one repo available
            config.github.repo = owner_repos[0].name.clone();
            println!("Using repository: {}", config.github.repo);
        } else {
            // Multiple repos available - let user choose interactively
            config.github.repo = select_repository(&owner_repos)?;
            println!("Selected repository: {}", config.github.repo);
        }
    }

    Ok(config)
}

fn select_organization(user_login: &str, orgs: &[github::OrganizationInfo]) -> Result<String> {
    println!("\nMultiple organizations available:");
    println!("0. {} (Your personal account)", user_login);

    for (i, org) in orgs.iter().enumerate() {
        let desc = if org.description.is_empty() {
            "No description"
        } else {
            &org.description
        };
        println!("{}. {} - {}", i + 1, org.login, desc);
    }

    loop {
        print!("\nSelect organization (0-{}): ", orgs.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(choice) if choice == 0 => return Ok(user_login.to_string()),
            Ok(choice) if choice > 0 && choice <= orgs.len() => {
                return Ok(orgs[choice - 1].login.clone());
            }
            _ => {
                println!(
                    "Invalid selection. Please enter a number between 0 and {}.",
                    orgs.len()
                );
                continue;
            }
        }
    }
}

fn select_repository(repos: &[github::RepositoryInfo]) -> Result<String> {
    println!("\nMultiple repositories available:");

    for (i, repo) in repos.iter().enumerate() {
        let visibility = if repo.private { "Private" } else { "Public" };
        let desc = if repo.description.is_empty() {
            "No description"
        } else {
            &repo.description
        };
        println!("{}. {} ({}) - {}", i + 1, repo.name, visibility, desc);
    }

    loop {
        print!("\nSelect repository (1-{}): ", repos.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        match input.trim().parse::<usize>() {
            Ok(choice) if choice > 0 && choice <= repos.len() => {
                return Ok(repos[choice - 1].name.clone());
            }
            _ => {
                println!(
                    "Invalid selection. Please enter a number between 1 and {}.",
                    repos.len()
                );
                continue;
            }
        }
    }
}
