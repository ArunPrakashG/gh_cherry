use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use octocrab::{Octocrab, Page};
use regex::Regex;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::auth::GitHubAuth;
use crate::util::short_sha;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInfo {
    pub login: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    pub name: String,
    pub full_name: String,
    pub owner: String,
    pub description: String,
    pub default_branch: String,
    pub private: bool,
    pub fork: bool,
    pub stargazers_count: u32,
    pub forks_count: u32,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub login: String,
    pub name: String,
    pub email: String,
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

        let mut page: Page<octocrab::models::pulls::PullRequest> = self
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
        let sprint_regex =
            Regex::new(&self.config.tags.sprint_pattern).context("Invalid sprint pattern regex")?;

        loop {
            let mut stop_due_to_date = false;
            for pr in &page {
                // Filter by date
                let pr_updated_at = pr.updated_at.unwrap_or(pr.created_at.unwrap_or(Utc::now()));
                if pr_updated_at < since {
                    stop_due_to_date = true;
                    break;
                }

                // Get labels for the PR
                let labels = self.get_pr_labels(pr.number).await?;

                // Check if PR has the required tags
                if crate::github::pr_matches_criteria(&self.config, &labels, &sprint_regex) {
                    let commits = self.get_pr_commits(pr.number).await?;

                    let pr_info = PrInfo {
                        number: pr.number,
                        title: pr.title.clone().unwrap_or_default(),
                        author: pr.user.clone().map(|u| u.login).unwrap_or_default(),
                        created_at: pr.created_at.unwrap_or(Utc::now()),
                        updated_at: pr.updated_at.unwrap_or(pr.created_at.unwrap_or(Utc::now())),
                        labels,
                        commits,
                        head_sha: pr.head.sha.clone(),
                        base_ref: pr.base.ref_field.clone(),
                        head_ref: pr.head.ref_field.clone(),
                    };

                    matching_prs.push(pr_info);
                }
            }

            if stop_due_to_date {
                break;
            }

            // Next page
            if let Some(next_page) = self
                .octocrab
                .get_page::<octocrab::models::pulls::PullRequest>(&page.next)
                .await?
            {
                page = next_page;
            } else {
                break;
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
        // Get the PR details first
        let pr = self
            .octocrab
            .pulls(&self.config.github.owner, &self.config.github.repo)
            .get(pr_number)
            .await
            .context("Failed to fetch PR details")?;

        // For now, we'll just use the head commit of the PR
        // This is typically what you want to cherry-pick
        let commit_info = CommitInfo {
            sha: pr.head.sha.clone(),
            message: pr.title.unwrap_or_else(|| format!("PR #{}", pr_number)),
            author: pr.user.map(|u| u.login).unwrap_or_else(|| "Unknown".to_string()),
            date: pr.created_at.unwrap_or(Utc::now()),
        };

        tracing::info!("Using head commit {} for PR #{}", pr.head.sha, pr_number);
        Ok(vec![commit_info])
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
        let comment_body = {
            let mut lines = Vec::with_capacity(commit_shas.len());
            for sha in commit_shas {
                lines.push(format!("- {}", short_sha(sha)));
            }
            format!(
                "🍒 **Cherry-picked to `{}`**\n\nCommits:\n{}",
                target_branch,
                lines.join("\n")
            )
        };

        self.octocrab
            .issues(&self.config.github.owner, &self.config.github.repo)
            .create_comment(pr_number, comment_body)
            .await
            .context("Failed to add cherry-pick comment")?;

        Ok(())
    }

    /// Fetches user organizations that the authenticated user belongs to
    pub async fn list_user_organizations(&self) -> Result<Vec<OrganizationInfo>> {
        tracing::info!("Fetching user organizations");

        let orgs = self
            .octocrab
            .current()
            .list_org_memberships_for_authenticated_user()
            .per_page(100)
            .send()
            .await
            .context("Failed to fetch user organizations")?;

        let mut org_infos = Vec::new();
        for org in orgs {
            let org_info = OrganizationInfo {
                login: org.organization.login,
                name: org.organization.name.unwrap_or_default(),
                description: org.organization.description.unwrap_or_default(),
            };
            org_infos.push(org_info);
        }

        tracing::info!("Found {} organizations", org_infos.len());
        Ok(org_infos)
    }

    /// Fetches repositories accessible to the authenticated user
    pub async fn list_user_repositories(&self) -> Result<Vec<RepositoryInfo>> {
        tracing::info!("Fetching user repositories");

        let mut page = self
            .octocrab
            .current()
            .list_repos_for_authenticated_user()
            .per_page(100)
            .send()
            .await
            .context("Failed to fetch user repositories")?;

        let mut repo_infos = Vec::new();
        loop {
            for repo in &page {
            let repo_info = RepositoryInfo {
                    name: repo.name.clone(),
                    full_name: repo.full_name.clone().unwrap_or_default(),
                    owner: repo.owner.clone().map(|o| o.login).unwrap_or_default(),
                    description: repo.description.clone().unwrap_or_default(),
                    default_branch: repo.default_branch.clone().unwrap_or_else(|| "main".to_string()),
                    private: repo.private.unwrap_or(false),
                    fork: repo.fork.unwrap_or(false),
                    stargazers_count: repo.stargazers_count.unwrap_or(0),
                    forks_count: repo.forks_count.unwrap_or(0),
                    language: repo
                        .language
                        .as_ref()
                        .and_then(|v| v.as_str().map(|s| s.to_string())),
            };
            repo_infos.push(repo_info);
            }

            if let Some(next_page) = self.octocrab.get_page(&page.next).await? {
                page = next_page;
            } else {
                break;
            }
        }

        tracing::info!("Found {} repositories", repo_infos.len());
        Ok(repo_infos)
    }

    /// Gets information about the authenticated user
    pub async fn get_authenticated_user(&self) -> Result<UserInfo> {
        tracing::info!("Fetching authenticated user information");

        let user = self
            .octocrab
            .current()
            .user()
            .await
            .context("Failed to fetch authenticated user")?;

        let user_info = UserInfo {
            login: user.login,
            name: user.name.unwrap_or_default(),
            email: user.email.unwrap_or_default(),
        };

        Ok(user_info)
    }
}

pub(crate) fn pr_matches_criteria(config: &Config, labels: &[String], sprint_regex: &Regex) -> bool {
    let has_sprint_tag = labels.iter().any(|label| sprint_regex.is_match(label));
    let has_env_tag = labels.iter().any(|label| label == &config.tags.environment);
    let has_pending_tag = labels.iter().any(|label| label == &config.tags.pending_tag);
    has_sprint_tag && has_env_tag && has_pending_tag
}

/// Trait abstraction to allow mocking PR listing in tests without network calls.
#[async_trait]
#[allow(dead_code)]
pub trait PrLister: Send + Sync {
    async fn list_matching_prs(&self) -> Result<Vec<PrInfo>>;
    fn config(&self) -> &Config;
}

#[async_trait]
impl PrLister for GitHubClient {
    async fn list_matching_prs(&self) -> Result<Vec<PrInfo>> {
        // Call inherent async method
        GitHubClient::list_matching_prs(self).await
    }
    fn config(&self) -> &Config { &self.config }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config_with(env: &str, pending: &str, sprint: &str) -> Config {
        Config {
            github: crate::config::GitHubConfig {
                owner: String::new(),
                repo: String::new(),
                base_branch: "main".into(),
                target_branch: "main".into(),
                cherry_pick_source_branch: "main".into(),
                branch_name_template: "ch/{task_id}".into(),
            },
            tags: crate::config::TagConfig {
                sprint_pattern: sprint.into(),
                environment: env.into(),
                pending_tag: pending.into(),
                completed_tag: "done".into(),
            },
            ui: crate::config::UiConfig { days_back: 7, page_size: 20, only_forked_repos: false },
        }
    }

    #[test]
    fn pr_label_matching_works() {
    let cfg = test_config_with("DEV", "pending cherrypick", r"S\d+");
    let re = Regex::new(&cfg.tags.sprint_pattern).unwrap();
        let labels = vec![
            "S12".to_string(),
            "DEV".to_string(),
            "pending cherrypick".to_string(),
        ];
    assert!(crate::github::pr_matches_criteria(&cfg, &labels, &re));

    let labels2 = vec!["S12".to_string(), "QA".to_string(), "pending cherrypick".to_string()];
    assert!(!crate::github::pr_matches_criteria(&cfg, &labels2, &re));
    }

    struct MockLister { #[allow(dead_code)] cfg: Config, prs: Vec<PrInfo> }

    #[async_trait]
    impl super::PrLister for MockLister {
        async fn list_matching_prs(&self) -> Result<Vec<PrInfo>> { Ok(self.prs.clone()) }
        fn config(&self) -> &Config { &self.cfg }
    }

    #[tokio::test]
    async fn mock_lister_returns_data_without_network() {
        let cfg = test_config_with("DEV", "pending cherrypick", r"S\d+");
        let prs = vec![PrInfo {
            number: 1,
            title: "Test".into(),
            author: "alice".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            labels: vec!["S1".into(), "DEV".into(), "pending cherrypick".into()],
            commits: vec![],
            head_sha: "abcd1234".into(),
            base_ref: "main".into(),
            head_ref: "feature".into(),
        }];
        let mock = MockLister { cfg, prs: prs.clone() };
        let got = mock.list_matching_prs().await.unwrap();
        assert_eq!(got.len(), prs.len());
    }
}
