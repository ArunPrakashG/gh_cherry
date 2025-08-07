use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::io;

use crate::github::{OrganizationInfo, RepositoryInfo};

pub struct SelectorApp {
    should_quit: bool,
    selected_index: usize,
    scroll_offset: usize,
    search_query: String,
    search_mode: bool,
}

impl SelectorApp {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            selected_index: 0,
            scroll_offset: 0,
            search_query: String::new(),
            search_mode: false,
        }
    }

    pub fn run_organization_selector(
        user_login: &str,
        orgs: &[OrganizationInfo],
    ) -> Result<String> {
        // Create options list (user account + organizations)
        let mut options = vec![format!("{} (Your personal account)", user_login)];
        for org in orgs {
            let desc = if org.description.is_empty() {
                "No description".to_string()
            } else {
                org.description.clone()
            };
            options.push(format!("{} - {}", org.login, desc));
        }

        let selected_index = Self::run_selector("Select Organization", &options)?;

        if selected_index == 0 {
            Ok(user_login.to_string())
        } else {
            Ok(orgs[selected_index - 1].login.clone())
        }
    }

    pub fn run_repository_selector(repos: &[RepositoryInfo]) -> Result<String> {
        let mut app = SelectorApp::new();
        let selected_index = app.run_repository_selector_internal(repos)?;
        Ok(repos[selected_index].name.clone())
    }

    fn run_repository_selector_internal(&mut self, repos: &[RepositoryInfo]) -> Result<usize> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut filtered_indices: Vec<usize> = (0..repos.len()).collect();

        let result = loop {
            // Filter repos based on search query
            if self.search_mode && !self.search_query.is_empty() {
                filtered_indices = repos
                    .iter()
                    .enumerate()
                    .filter(|(_, repo)| {
                        let search_text = format!("{} {}", repo.name, repo.description).to_lowercase();
                        search_text.contains(&self.search_query.to_lowercase())
                    })
                    .map(|(index, _)| index)
                    .collect();
            } else if !self.search_mode {
                filtered_indices = (0..repos.len()).collect();
            }

            // Adjust selected index if it's out of bounds
            if self.selected_index >= filtered_indices.len() && !filtered_indices.is_empty() {
                self.selected_index = filtered_indices.len() - 1;
            }

            terminal.draw(|f| {
                self.render_repository_selector(f, repos, &filtered_indices);
            })?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                self.should_quit = true;
                                break Err(anyhow::anyhow!("Selection cancelled"));
                            }
                            KeyCode::Enter => {
                                if !filtered_indices.is_empty() {
                                    break Ok(filtered_indices[self.selected_index]);
                                }
                            }
                            KeyCode::Up => {
                                if self.selected_index > 0 {
                                    self.selected_index -= 1;
                                    if self.selected_index < self.scroll_offset {
                                        self.scroll_offset = self.selected_index;
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if self.selected_index + 1 < filtered_indices.len() {
                                    self.selected_index += 1;
                                    // Calculate max_visible items the same way as in render function
                                    let available_height = 15; // Approximate height available for list content
                                    let max_visible = (available_height / 3) as usize; // 3 lines per item (name + desc + separator)
                                    if self.selected_index >= self.scroll_offset + max_visible {
                                        self.scroll_offset = self.selected_index - max_visible + 1;
                                    }
                                }
                            }
                            KeyCode::Char('/') => {
                                self.search_mode = true;
                                self.search_query.clear();
                            }
                            KeyCode::Backspace if self.search_mode => {
                                self.search_query.pop();
                                if self.search_query.is_empty() {
                                    self.search_mode = false;
                                }
                            }
                            KeyCode::Char(c) if self.search_mode => {
                                self.search_query.push(c);
                            }
                            _ => {}
                        }
                    }
                }
            }
        };

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

    fn run_selector(title: &str, options: &[String]) -> Result<usize> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = SelectorApp::new();
        let mut filtered_indices: Vec<usize> = (0..options.len()).collect();

        let result = loop {
            // Filter options based on search query
            if app.search_mode && !app.search_query.is_empty() {
                filtered_indices = options
                    .iter()
                    .enumerate()
                    .filter(|(_, option)| {
                        option
                            .to_lowercase()
                            .contains(&app.search_query.to_lowercase())
                    })
                    .map(|(index, _)| index)
                    .collect();
            } else if !app.search_mode {
                filtered_indices = (0..options.len()).collect();
            }

            // Adjust selected index if it's out of bounds
            if app.selected_index >= filtered_indices.len() && !filtered_indices.is_empty() {
                app.selected_index = filtered_indices.len() - 1;
            }

            terminal.draw(|f| {
                app.render_selector(f, title, options, &filtered_indices);
            })?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                                break Err(anyhow::anyhow!("Selection cancelled"));
                            }
                            KeyCode::Enter => {
                                if !filtered_indices.is_empty() {
                                    break Ok(filtered_indices[app.selected_index]);
                                }
                            }
                            KeyCode::Up => {
                                if app.selected_index > 0 {
                                    app.selected_index -= 1;
                                    if app.selected_index < app.scroll_offset {
                                        app.scroll_offset = app.selected_index;
                                    }
                                }
                            }
                            KeyCode::Down => {
                                if app.selected_index + 1 < filtered_indices.len() {
                                    app.selected_index += 1;
                                    let max_visible = 10; // Single line items for organizations
                                    if app.selected_index >= app.scroll_offset + max_visible {
                                        app.scroll_offset = app.selected_index - max_visible + 1;
                                    }
                                }
                            }
                            KeyCode::Char('/') => {
                                app.search_mode = true;
                                app.search_query.clear();
                            }
                            KeyCode::Backspace if app.search_mode => {
                                app.search_query.pop();
                                if app.search_query.is_empty() {
                                    app.search_mode = false;
                                }
                            }
                            KeyCode::Char(c) if app.search_mode => {
                                app.search_query.push(c);
                            }
                            _ => {}
                        }
                    }
                }
            }
        };

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

    fn render_repository_selector(
        &self,
        f: &mut Frame,
        repos: &[RepositoryInfo],
        filtered_indices: &[usize],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // List
                Constraint::Length(3), // Search bar
                Constraint::Length(3), // Instructions
            ])
            .split(f.area());

        // Title
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        let title_paragraph = Paragraph::new("Select Repository")
            .block(title_block)
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(title_paragraph, chunks[0]);

        // List with multi-line items
        let max_visible = (chunks[1].height.saturating_sub(2) / 3) as usize; // 3 lines per item (name + desc + separator)
        
        // Ensure scroll_offset doesn't exceed filtered indices
        let scroll_offset = self.scroll_offset.min(filtered_indices.len().saturating_sub(1));
        let end_index = (scroll_offset + max_visible).min(filtered_indices.len());
        let visible_indices = if end_index > scroll_offset {
            &filtered_indices[scroll_offset..end_index]
        } else {
            &[]
        };

        let items: Vec<ListItem> = visible_indices
            .iter()
            .enumerate()
            .map(|(i, &original_index)| {
                let repo = &repos[original_index];
                let is_selected = scroll_offset + i == self.selected_index;

                // Main line - repository name with fork indication
                let name_line = if repo.fork {
                    format!("{} (fork)", repo.name)
                } else {
                    repo.name.clone()
                };

                // Description line (smaller/dimmed)
                let desc_line = if repo.description.is_empty() {
                    "No description available".to_string()
                } else {
                    repo.description.clone()
                };

                // Separator line
                let separator_line = "─".repeat(60);

                let lines = vec![
                    Line::from(Span::styled(name_line, 
                        if is_selected { 
                            Style::default().fg(Color::Black).bg(Color::LightBlue).add_modifier(Modifier::BOLD)
                        } else { 
                            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
                        }
                    )),
                    Line::from(Span::styled(desc_line,
                        if is_selected {
                            Style::default().fg(Color::DarkGray).bg(Color::LightBlue)
                        } else {
                            Style::default().fg(Color::Gray)
                        }
                    )),
                    Line::from(Span::styled(separator_line,
                        // Separator is never highlighted - always use dim styling
                        Style::default().fg(Color::DarkGray)
                    )),
                ];

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} repositories (showing {}-{}) ", 
                        filtered_indices.len(),
                        scroll_offset + 1,
                        (scroll_offset + visible_indices.len()).min(filtered_indices.len())
                    ))
                    .title_style(Style::default().fg(Color::Yellow)),
            );

        f.render_widget(list, chunks[1]);

        // Search bar
        let search_title = if self.search_mode {
            format!(" Search: {} ", self.search_query)
        } else {
            " Press '/' to search ".to_string()
        };

        let search_style = if self.search_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let search_block = Block::default()
            .borders(Borders::ALL)
            .title(search_title)
            .title_style(search_style)
            .style(search_style);

        let search_paragraph = Paragraph::new("").block(search_block);
        f.render_widget(search_paragraph, chunks[2]);

        // Instructions
        let instructions = vec!["↑/↓: Navigate | Enter: Select | /: Search | Esc/q: Cancel"];
        let instructions_paragraph = Paragraph::new(instructions.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Instructions ")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions_paragraph, chunks[3]);
    }

    fn render_selector(
        &self,
        f: &mut Frame,
        title: &str,
        options: &[String],
        filtered_indices: &[usize],
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(10),   // List
                Constraint::Length(3), // Search bar
                Constraint::Length(3), // Instructions
            ])
            .split(f.area());

        // Title
        let title_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan));
        let title_paragraph = Paragraph::new(title)
            .block(title_block)
            .alignment(Alignment::Center)
            .style(Style::default().add_modifier(Modifier::BOLD));
        f.render_widget(title_paragraph, chunks[0]);

        // List
        let max_visible = chunks[1].height.saturating_sub(2) as usize; // Account for borders
        let end_index = (self.scroll_offset + max_visible).min(filtered_indices.len());
        let visible_indices = &filtered_indices[self.scroll_offset..end_index];

        let items: Vec<ListItem> = visible_indices
            .iter()
            .enumerate()
            .map(|(i, &original_index)| {
                let content = &options[original_index];
                let style = if self.scroll_offset + i == self.selected_index {
                    Style::default()
                        .bg(Color::LightBlue)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(content.as_str()).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" {} items ", filtered_indices.len()))
                    .title_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White));

        f.render_widget(list, chunks[1]);

        // Search bar
        let search_title = if self.search_mode {
            format!(" Search: {} ", self.search_query)
        } else {
            " Press '/' to search ".to_string()
        };

        let search_style = if self.search_mode {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Gray)
        };

        let search_block = Block::default()
            .borders(Borders::ALL)
            .title(search_title)
            .title_style(search_style)
            .style(search_style);

        let search_paragraph = Paragraph::new("").block(search_block);
        f.render_widget(search_paragraph, chunks[2]);

        // Instructions
        let instructions = vec!["↑/↓: Navigate | Enter: Select | /: Search | Esc/q: Cancel"];
        let instructions_paragraph = Paragraph::new(instructions.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Instructions ")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions_paragraph, chunks[3]);
    }
}
