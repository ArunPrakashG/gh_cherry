use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub github: GitHubConfig,
    pub tags: TagConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    pub base_branch: String,
    pub target_branch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagConfig {
    pub sprint_pattern: String,
    pub environment: String,
    pub pending_tag: String,
    pub completed_tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub days_back: u32,
    pub page_size: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: GitHubConfig {
                owner: "".to_string(),
                repo: "".to_string(),
                base_branch: "develop".to_string(),
                target_branch: "main".to_string(),
            },
            tags: TagConfig {
                sprint_pattern: r"S\d+".to_string(),
                environment: "DEV".to_string(),
                pending_tag: "pending cherrypick".to_string(),
                completed_tag: "cherry picked".to_string(),
            },
            ui: UiConfig {
                days_back: 28,
                page_size: 20,
            },
        }
    }
}

impl Config {
    pub fn load(path: Option<&str>) -> Result<Self> {
        let config_path = match path {
            Some(p) => p.to_string(),
            None => {
                let config_dir = dirs::config_dir()
                    .context("Failed to get config directory")?
                    .join("gh_cherry");
                config_dir.join("config.toml").to_string_lossy().to_string()
            }
        };

        if Path::new(&config_path).exists() {
            let contents = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path))?;
            let config: Config = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file: {}", config_path))?;
            Ok(config)
        } else {
            tracing::warn!("Config file not found at {}, using defaults", config_path);
            Ok(Config::default())
        }
    }

    pub fn with_overrides(
        mut self,
        owner: Option<String>,
        repo: Option<String>,
        base_branch: Option<String>,
        target_branch: Option<String>,
        days: Option<u32>,
    ) -> Self {
        if let Some(owner) = owner {
            self.github.owner = owner;
        }
        if let Some(repo) = repo {
            self.github.repo = repo;
        }
        if let Some(base_branch) = base_branch {
            self.github.base_branch = base_branch;
        }
        if let Some(target_branch) = target_branch {
            self.github.target_branch = target_branch;
        }
        if let Some(days) = days {
            self.ui.days_back = days;
        }
        self
    }

    pub fn validate(&self) -> Result<()> {
        // Allow empty owner/repo for auto-discovery mode
        // They will be populated later via GitHub API
        Ok(())
    }

    pub fn needs_auto_discovery(&self) -> bool {
        self.github.owner.is_empty() || self.github.repo.is_empty()
    }
}
