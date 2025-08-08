use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone)]
pub enum AuthMethod {
    GitHubCli(String),
    PersonalAccessToken(String),
}

pub struct GitHubAuth;

impl GitHubAuth {
    /// Attempts to authenticate using various methods in order of preference:
    /// 1. GitHub CLI (gh)
    /// 2. GITHUB_TOKEN environment variable
    pub async fn authenticate() -> Result<AuthMethod> {
        // Try GitHub CLI first
        if let Ok(token) = Self::get_github_cli_token() {
            tracing::info!("Using GitHub CLI authentication");
            return Ok(AuthMethod::GitHubCli(token));
        }

        // Try environment variable
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            tracing::info!("Using GitHub token from environment variable");
            return Ok(AuthMethod::PersonalAccessToken(token));
        }

        anyhow::bail!(
            "No authentication method found. Please either:\n\
            1. Install and authenticate with GitHub CLI: gh auth login\n\
            2. Set GITHUB_TOKEN environment variable"
        );
    }

    fn get_github_cli_token() -> Result<String> {
        // Check if gh CLI is available
        let output = Command::new("gh")
            .args(["auth", "status", "--show-token"])
            .output()
            .context("Failed to execute gh command. Is GitHub CLI installed?")?;

        if !output.status.success() {
            anyhow::bail!("GitHub CLI not authenticated. Run 'gh auth login'");
        }

        // Parse the token from the output
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some(pos) = line.find("Token:") {
                // handle formats like "Token: ghp_..." possibly followed by extra info
                let rest = &line[pos + "Token:".len()..];
                let token = rest.split_whitespace().next().unwrap_or("");
                if !token.is_empty() {
                    return Ok(token.to_string());
                }
            }
        }

        // If we can't get the token directly, try using gh api
        let output = Command::new("gh")
            .args(["auth", "token"])
            .output()
            .context("Failed to get token from gh auth token")?;

        if output.status.success() {
            let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !token.is_empty() {
                return Ok(token);
            }
        }

        anyhow::bail!("Failed to get authentication token from GitHub CLI");
    }

    pub fn get_token(auth_method: &AuthMethod) -> &str {
        match auth_method {
            AuthMethod::GitHubCli(token) | AuthMethod::PersonalAccessToken(token) => token,
        }
    }
}
