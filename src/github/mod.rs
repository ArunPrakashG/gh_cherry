use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use octocrab::Octocrab;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::auth::GitHubAuth;
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrInfo {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub labels: Vec<String>,
    pub commits: Vec<CommitInfo>,
    pub head_sha: String,
    pub base_ref: String,
    pub head_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub date: DateTime<Utc>,
}

pub struct GitHubClient {
    octocrab: Octocrab,
    config: Config,
}

impl GitHubClient {
    pub async fn new(config: Config) -> Result<Self> {
        let auth_method = GitHubAuth::authenticate().await?;
        let token = GitHubAuth::get_token(&auth_method);
        
        let octocrab = Octocrab::builder()
            .personal_token(token.to_string())
            .build()
            .context("Failed to create GitHub client")?;

        Ok(Self { octocrab, config })
    }

    /// Lists PRs from the base branch that match the filtering criteria
    pub async fn list_matching_prs(&self) -> Result<Vec<PrInfo>> {
        let since = Utc::now() - chrono::Duration::days(self.config.ui.days_back as i64);
        
        tracing::info!(
            "Fetching PRs from {}/{} on branch {} since {}",
            self.config.github.owner,
            self.config.github.repo,
            self.config.github.base_branch,
            since.format("%Y-%m-%d")
        );

        let pulls = self
            .octocrab
            .pulls(&self.config.github.owner, &self.config.github.repo)
            .list()
            .state(octocrab::params::State::All)
            .base(&self.config.github.base_branch)
            .sort(octocrab::params::pulls::Sort::Updated)
            .direction(octocrab::params::Direction::Descending)
            .per_page(100)
            .send()
            .await
            .context("Failed to fetch pull requests")?;

        let mut matching_prs = Vec::new();
        let sprint_regex = Regex::new(&self.config.tags.sprint_pattern)
            .context("Invalid sprint pattern regex")?;

        for pr in pulls {
            // Filter by date
            let pr_updated_at = pr.updated_at.unwrap_or(pr.created_at.unwrap_or(Utc::now()));
            if pr_updated_at < since {
                continue;
            }

            // Get labels for the PR
            let labels = self.get_pr_labels(pr.number).await?;
            
            // Check if PR has the required tags
            if self.pr_matches_criteria(&labels, &sprint_regex) {
                let commits = self.get_pr_commits(pr.number).await?;
                
                let pr_info = PrInfo {
                    number: pr.number,
                    title: pr.title.unwrap_or_default(),
                    author: pr.user.map(|u| u.login).unwrap_or_default(),
                    created_at: pr.created_at.unwrap_or(Utc::now()),
                    updated_at: pr.updated_at.unwrap_or(pr.created_at.unwrap_or(Utc::now())),
                    labels,
                    commits,
                    head_sha: pr.head.sha,
                    base_ref: pr.base.ref_field,
                    head_ref: pr.head.ref_field,
                };
                
                matching_prs.push(pr_info);
            }
        }

        tracing::info!("Found {} matching PRs", matching_prs.len());
        Ok(matching_prs)
    }

    async fn get_pr_labels(&self, pr_number: u64) -> Result<Vec<String>> {
        let labels = self
            .octocrab
            .issues(&self.config.github.owner, &self.config.github.repo)
            .get(pr_number)
            .await
            .context("Failed to fetch PR labels")?
            .labels
            .into_iter()
            .map(|label| label.name)
            .collect();

        Ok(labels)
    }

    async fn get_pr_commits(&self, pr_number: u64) -> Result<Vec<CommitInfo>> {
        // Get the PR details first to get the commit range
        let pr = self
            .octocrab
            .pulls(&self.config.github.owner, &self.config.github.repo)
            .get(pr_number)
            .await
            .context("Failed to fetch PR details")?;

        // Get commits between base and head
        let commits = self
            .octocrab
            .repos(&self.config.github.owner, &self.config.github.repo)
            .list_commits()
            .sha(&pr.head.sha)
            .per_page(100)
            .send()
            .await
            .context("Failed to fetch PR commits")?;

        let mut commit_infos = Vec::new();
        
        // Convert to our commit info format
        for commit in commits {
            let commit_data = CommitInfo {
                sha: commit.sha,
                message: commit.commit.message,
                author: commit.commit.author
                    .as_ref()
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
                date: commit.commit.author
                    .as_ref()
                    .and_then(|a| a.date)
                    .unwrap_or_else(|| Utc::now()),
            };
            commit_infos.push(commit_data);
        }

        Ok(commit_infos)
    }

    fn pr_matches_criteria(&self, labels: &[String], sprint_regex: &Regex) -> bool {
        let has_sprint_tag = labels.iter().any(|label| sprint_regex.is_match(label));
        let has_env_tag = labels.iter().any(|label| label == &self.config.tags.environment);
        let has_pending_tag = labels.iter().any(|label| label == &self.config.tags.pending_tag);

        has_sprint_tag && has_env_tag && has_pending_tag
    }

    /// Updates a PR's labels after successful cherry-pick
    pub async fn update_pr_labels(&self, pr_number: u64) -> Result<()> {
        tracing::info!("Updating labels for PR #{}", pr_number);

        // Get current labels
        let mut labels = self.get_pr_labels(pr_number).await?;

        // Remove pending tag and add completed tag
        labels.retain(|label| label != &self.config.tags.pending_tag);
        if !labels.contains(&self.config.tags.completed_tag) {
            labels.push(self.config.tags.completed_tag.clone());
        }

        // Update the labels
        self.octocrab
            .issues(&self.config.github.owner, &self.config.github.repo)
            .update(pr_number)
            .labels(&labels)
            .send()
            .await
            .context("Failed to update PR labels")?;

        tracing::info!("Successfully updated labels for PR #{}", pr_number);
        Ok(())
    }

    /// Adds a comment to the PR indicating successful cherry-pick
    pub async fn add_cherry_pick_comment(
        &self,
        pr_number: u64,
        target_branch: &str,
        commit_shas: &[String],
    ) -> Result<()> {
        let comment_body = format!(
            "🍒 **Cherry-picked to `{}`**\n\nCommits:\n{}",
            target_branch,
            commit_shas
                .iter()
                .map(|sha| format!("- {}", &sha[..8]))
                .collect::<Vec<_>>()
                .join("\n")
        );

        self.octocrab
            .issues(&self.config.github.owner, &self.config.github.repo)
            .create_comment(pr_number, comment_body)
            .await
            .context("Failed to add cherry-pick comment")?;

        Ok(())
    }
}
