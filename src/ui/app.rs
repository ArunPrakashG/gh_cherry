use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Frame, Terminal};
use std::io;

use crate::config::Config;
use crate::git::GitOperations;
use crate::github::GitHubClient;
use crate::util::short_sha;

use super::components::{MainMenu, PrList, ProgressView};
use super::state::{AppState, Screen};

pub struct App {
    state: AppState,
    github_client: GitHubClient,
    git_ops: GitOperations,
    config: Config,
    should_quit: bool,
}

impl App {
    pub async fn new(config: Config) -> Result<Self> {
        // Validate configuration
        config.validate()?;

        // Initialize GitHub client
        let github_client = GitHubClient::new(config.clone()).await?;

        // Initialize Git operations
        let git_ops = GitOperations::discover()?;

        Ok(Self {
            state: AppState::new(),
            github_client,
            git_ops,
            config,
            should_quit: false,
        })
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Load initial data
        self.load_prs().await?;

        // Main loop
        let result = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app<B: ratatui::backend::Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match self.handle_key_event(key.code).await {
                        Ok(should_continue) => {
                            if !should_continue {
                                break;
                            }
                        }
                        Err(e) => {
                            self.state.set_error(format!("Error: {}", e));
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    fn ui(&self, f: &mut Frame) {
        match &self.state.current_screen {
            Screen::MainMenu => {
                MainMenu::render(f, &self.state);
            }
            Screen::PrList => {
                PrList::render(f, &self.state, &self.config);
            }
            Screen::Progress => {
                ProgressView::render(f, &self.state);
            }
            Screen::Error => {
                self.render_error(f);
            }
        }
    }

    fn render_error(&self, f: &mut Frame) {
        use ratatui::{
            layout::{Constraint, Direction, Layout},
            style::{Color, Style},
            widgets::{Block, Borders, Paragraph, Wrap},
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(f.area());

        let error_message = self
            .state
            .error_message
            .as_deref()
            .unwrap_or("Unknown error");
        let paragraph = Paragraph::new(error_message)
            .block(Block::default().title("Error").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[0]);
    }

    async fn handle_key_event(&mut self, key: KeyCode) -> Result<bool> {
        match key {
            KeyCode::Char('q') => {
                self.should_quit = true;
                return Ok(false);
            }
            KeyCode::Esc => match &self.state.current_screen {
                Screen::MainMenu => {
                    self.should_quit = true;
                    return Ok(false);
                }
                _ => {
                    self.state.current_screen = Screen::MainMenu;
                }
            },
            _ => {
                match &self.state.current_screen {
                    Screen::MainMenu => self.handle_main_menu_input(key).await?,
                    Screen::PrList => self.handle_pr_list_input(key).await?,
                    Screen::Progress => self.handle_progress_input(key).await?,
                    Screen::Error => {
                        // Any key from error screen goes back to main menu
                        self.state.current_screen = Screen::MainMenu;
                    }
                }
            }
        }

        Ok(true)
    }

    async fn handle_main_menu_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('1') | KeyCode::Enter => {
                self.state.current_screen = Screen::PrList;
            }
            KeyCode::Char('r') => {
                self.load_prs().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_pr_list_input(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.pr_list_state.select_previous();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.state.pr_list_state.select_next();
            }
            KeyCode::Enter => {
                if let Some(selected) = self.state.pr_list_state.selected() {
                    self.cherry_pick_pr(selected).await?;
                }
            }
            KeyCode::Char('r') => {
                self.load_prs().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_progress_input(&mut self, _key: KeyCode) -> Result<()> {
        // Progress screen doesn't handle input
        Ok(())
    }

    async fn load_prs(&mut self) -> Result<()> {
        self.state.set_loading("Loading PRs...");
        self.state.current_screen = Screen::Progress;

        match self.github_client.list_matching_prs().await {
            Ok(prs) => {
                self.state.set_prs(prs);
                self.state.current_screen = Screen::PrList;
            }
            Err(e) => {
                self.state.set_error(format!("Failed to load PRs: {}", e));
                self.state.current_screen = Screen::Error;
            }
        }

        Ok(())
    }

    async fn cherry_pick_pr(&mut self, pr_index: usize) -> Result<()> {
        // Get PR details before borrowing mutably
        let pr = if let Some(pr) = self.state.prs.get(pr_index) {
            pr.clone()
        } else {
            return Ok(());
        };

        self.state
            .set_loading(&format!("Cherry-picking PR #{}: {}", pr.number, pr.title));
        self.state.current_screen = Screen::Progress;

        // Switch to target branch
        if let Err(e) = self
            .git_ops
            .checkout_branch(&self.config.github.target_branch)
        {
            self.state
                .set_error(format!("Failed to checkout target branch: {}", e));
            self.state.current_screen = Screen::Error;
            return Ok(());
        }

        let mut success = true;
        let mut cherry_picked_commits = Vec::new();

        // Cherry-pick each commit in the PR
        for commit in &pr.commits {
            match self.git_ops.cherry_pick(&commit.sha) {
                Ok(result) => {
                    if result.success {
                        if let Some(sha) = result.commit_sha {
                            cherry_picked_commits.push(sha);
                        }
                    } else {
                        // Handle conflicts
                        let short = short_sha(&commit.sha);
                        self.state.set_error(format!(
                            "Conflicts in commit {}: {:?}. Please resolve manually and press any key to continue.",
                            short,
                            result.conflicts
                        ));
                        self.state.current_screen = Screen::Error;
                        success = false;
                        break;
                    }
                }
                Err(e) => {
                    let short = short_sha(&commit.sha);
                    self.state
                        .set_error(format!("Failed to cherry-pick commit {}: {}", short, e));
                    self.state.current_screen = Screen::Error;
                    success = false;
                    break;
                }
            }
        }

        if success {
            // Update PR labels
            if let Err(e) = self.github_client.update_pr_labels(pr.number).await {
                tracing::warn!("Failed to update PR labels: {}", e);
            }

            // Add comment to PR
            if let Err(e) = self
                .github_client
                .add_cherry_pick_comment(
                    pr.number,
                    &self.config.github.target_branch,
                    &cherry_picked_commits,
                )
                .await
            {
                tracing::warn!("Failed to add cherry-pick comment: {}", e);
            }

            self.state
                .set_success(&format!("Successfully cherry-picked PR #{}", pr.number));
            self.state.current_screen = Screen::PrList;
        }

        Ok(())
    }
}
