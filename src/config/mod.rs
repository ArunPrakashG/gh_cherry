use crate::ui::config_selector::{ConfigChoice, ConfigSelectorApp};
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
    pub cherry_pick_source_branch: String,
    pub branch_name_template: String,
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
    pub only_forked_repos: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            github: GitHubConfig {
                owner: "".to_string(),
                repo: "".to_string(),
                base_branch: "master".to_string(),
                target_branch: "master".to_string(),
                cherry_pick_source_branch: "master".to_string(),
                branch_name_template: "cherry-pick/{task_id}".to_string(),
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
                only_forked_repos: false,
            },
        }
    }
}

impl Config {
    #[allow(clippy::too_many_arguments)] // Accepting many optional overrides keeps CLI mapping straightforward
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

        let mut config = if Path::new(&config_path).exists() {
            let contents = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {}", config_path))?;
            let config: Config = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file: {}", config_path))?;
            config
        } else {
            tracing::warn!("Config file not found at {}, using defaults", config_path);
            Config::default()
        };

        // Always load project-specific cherry.env file if it exists
        config.load_env_overrides()?;

        Ok(config)
    }

    pub fn load_with_prompt(path: Option<&str>) -> Result<Self> {
        // Check if cherry.env exists
        let env_exists = Path::new("cherry.env").exists();

        if env_exists {
            // Use TUI selector
            let choice = ConfigSelectorApp::run_config_selector()?;

            match choice {
                ConfigChoice::UseDefaults => {
                    println!("Using default configuration only...");
                    Ok(Config::default())
                }
                ConfigChoice::UseGlobalConfig => {
                    println!("Loading global config file only...");
                    Self::load_global_only(path)
                }
                ConfigChoice::LoadFromEnv => {
                    println!("Loading configuration from cherry.env...");
                    Self::load(path)
                }
            }
        } else {
            // No cherry.env file, just load normally
            Self::load(path)
        }
    }

    fn load_global_only(path: Option<&str>) -> Result<Self> {
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

    fn load_env_overrides(&mut self) -> Result<()> {
        let env_path = Path::new("cherry.env");
        if env_path.exists() {
            let contents =
                std::fs::read_to_string(env_path).context("Failed to read cherry.env file")?;

            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                if let Some((key, value)) = line.split_once('=') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');

                    match key {
                        "GITHUB_OWNER" => self.github.owner = value.to_string(),
                        "GITHUB_REPO" => self.github.repo = value.to_string(),
                        "BASE_BRANCH" => self.github.base_branch = value.to_string(),
                        "TARGET_BRANCH" => self.github.target_branch = value.to_string(),
                        "CHERRY_PICK_SOURCE_BRANCH" => {
                            self.github.cherry_pick_source_branch = value.to_string()
                        }
                        "BRANCH_NAME_TEMPLATE" => {
                            self.github.branch_name_template = value.to_string()
                        }
                        "ONLY_FORKED_REPOS" => {
                            self.ui.only_forked_repos = value.parse().unwrap_or(false)
                        }
                        "DAYS_BACK" => self.ui.days_back = value.parse().unwrap_or(28),
                        _ => {} // Ignore unknown keys
                    }
                }
            }

            tracing::info!("Loaded project configuration from cherry.env");
        }

        Ok(())
    }

    pub fn save_env_overrides(&self) -> Result<()> {
        let env_content = format!(
            "# GitHub Cherry Pick Configuration\n\
            # This file contains project-specific settings\n\
            \n\
            GITHUB_OWNER=\"{}\"\n\
            GITHUB_REPO=\"{}\"\n\
            BASE_BRANCH=\"{}\"\n\
            TARGET_BRANCH=\"{}\"\n\
            CHERRY_PICK_SOURCE_BRANCH=\"{}\"\n\
            BRANCH_NAME_TEMPLATE=\"{}\"\n\
            ONLY_FORKED_REPOS={}\n\
            DAYS_BACK={}\n",
            self.github.owner,
            self.github.repo,
            self.github.base_branch,
            self.github.target_branch,
            self.github.cherry_pick_source_branch,
            self.github.branch_name_template,
            self.ui.only_forked_repos,
            self.ui.days_back
        );

        std::fs::write("cherry.env", env_content).context("Failed to write cherry.env file")?;

        tracing::info!("Saved project configuration to cherry.env");
        Ok(())
    }

    #[allow(clippy::too_many_arguments)] // Many optional CLI-driven overrides; refactor later with a builder if needed
    pub fn with_overrides(
        mut self,
        owner: Option<String>,
        repo: Option<String>,
        base_branch: Option<String>,
        target_branch: Option<String>,
        days: Option<u32>,
        only_forks: Option<bool>,
        source_branch: Option<String>,
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
        if let Some(only_forks) = only_forks {
            self.ui.only_forked_repos = only_forks;
        }
        if let Some(source_branch) = source_branch {
            self.github.cherry_pick_source_branch = source_branch;
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
