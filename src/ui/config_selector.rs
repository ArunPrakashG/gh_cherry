use crate::util::render_branch_name;
use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::prelude::*;
use ratatui::text::{Line, Span};
use ratatui::widgets::*;
use std::io;

pub struct ConfigSelectorApp {
    should_quit: bool,
    selected_index: usize,
    options: Vec<ConfigOption>,
}

#[derive(Clone)]
pub struct ConfigOption {
    pub title: String,
    pub description: String,
    pub choice: ConfigChoice,
}

#[derive(Clone, PartialEq)]
pub enum ConfigChoice {
    LoadFromEnv,
    UseDefaults,
    UseGlobalConfig,
}

impl ConfigSelectorApp {
    pub fn new() -> Self {
        let options = vec![
            ConfigOption {
                title: "Load from cherry.env".to_string(),
                description:
                    "Use project-specific configuration from cherry.env file (recommended)"
                        .to_string(),
                choice: ConfigChoice::LoadFromEnv,
            },
            ConfigOption {
                title: "Use defaults only".to_string(),
                description: "Start with default settings, ignore all configuration files"
                    .to_string(),
                choice: ConfigChoice::UseDefaults,
            },
            ConfigOption {
                title: "Use global config only".to_string(),
                description: "Load from global config.toml file, ignore cherry.env".to_string(),
                choice: ConfigChoice::UseGlobalConfig,
            },
        ];

        Self {
            should_quit: false,
            selected_index: 0,
            options,
        }
    }

    pub fn run_config_selector() -> Result<ConfigChoice> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut app = ConfigSelectorApp::new();

        let result = loop {
            terminal.draw(|f| {
                app.render_config_selector(f);
            })?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => {
                                app.should_quit = true;
                                break Err(anyhow::anyhow!("Configuration selection cancelled"));
                            }
                            KeyCode::Enter => {
                                break Ok(app.options[app.selected_index].choice.clone());
                            }
                            KeyCode::Up => {
                                if app.selected_index > 0 {
                                    app.selected_index -= 1;
                                }
                            }
                            KeyCode::Down => {
                                if app.selected_index + 1 < app.options.len() {
                                    app.selected_index += 1;
                                }
                            }
                            KeyCode::Char('1') => {
                                app.selected_index = 0;
                                break Ok(app.options[0].choice.clone());
                            }
                            KeyCode::Char('2') => {
                                app.selected_index = 1;
                                break Ok(app.options[1].choice.clone());
                            }
                            KeyCode::Char('3') => {
                                app.selected_index = 2;
                                break Ok(app.options[2].choice.clone());
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

    fn render_config_selector(&self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5), // Title and info
                Constraint::Min(10),   // Options list
                Constraint::Length(3), // Instructions
            ])
            .split(f.area());

        // Title and info
        let title_text = vec![
            Line::from(vec![Span::styled(
                "Configuration Loader",
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::Cyan),
            )]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Found cherry.env file. Choose how to load configuration:",
                Style::default().fg(Color::White),
            )]),
        ];

        let title_paragraph = Paragraph::new(title_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Configuration Setup ")
                    .title_style(Style::default().fg(Color::Cyan)),
            )
            .alignment(Alignment::Center);
        f.render_widget(title_paragraph, chunks[0]);

        // Options list
        let items: Vec<ListItem> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let is_selected = i == self.selected_index;

                let number = format!("{}.", i + 1);
                let title_line = format!("{} {}", number, option.title);
                let desc_line = format!("   {}", option.description);

                let lines = vec![
                    Line::from(Span::styled(
                        title_line,
                        if is_selected {
                            Style::default()
                                .fg(Color::Black)
                                .bg(Color::LightBlue)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        },
                    )),
                    Line::from(Span::styled(
                        desc_line,
                        if is_selected {
                            Style::default().fg(Color::DarkGray).bg(Color::LightBlue)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    )),
                    Line::from(""), // Separator
                ];

                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Options ")
                .title_style(Style::default().fg(Color::Yellow)),
        );

        f.render_widget(list, chunks[1]);

        // Instructions
        let instructions = ["↑/↓: Navigate | 1-3: Quick select | Enter: Confirm | Esc/q: Cancel"];
        let instructions_paragraph = Paragraph::new(instructions.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Instructions ")
                    .title_style(Style::default().fg(Color::Green)),
            )
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions_paragraph, chunks[2]);
    }

    /// TUI-based task ID input
    pub fn get_task_id_input(template: &str) -> Result<String> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut input = String::new();

        let result = loop {
            terminal.draw(|f| {
                Self::render_task_id_input(f, &input, template);
            })?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => {
                                if !input.trim().is_empty() {
                                    break Ok(input.trim().to_string());
                                }
                            }
                            KeyCode::Char(c) => {
                                input.push(c);
                            }
                            KeyCode::Backspace => {
                                input.pop();
                            }
                            KeyCode::Esc => {
                                break Err(anyhow::anyhow!("Task ID input cancelled"));
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

    fn render_task_id_input(f: &mut Frame, input: &str, template: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Length(2), // prompt
                Constraint::Length(2), // preview
                Constraint::Length(1), // instructions
                Constraint::Min(0),
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new(Line::from(Span::styled(
            "Task ID",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )))
        .alignment(Alignment::Left);
        f.render_widget(title, chunks[0]);

        // Prompt line (no boxes)
        let placeholder = "e.g., GH-123";
        let prompt_line = if input.is_empty() {
            Line::from(vec![
                Span::styled(">> ", Style::default().fg(Color::Yellow)),
                Span::styled(
                    placeholder,
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::ITALIC),
                ),
            ])
        } else {
            Line::from(vec![
                Span::styled(">> ", Style::default().fg(Color::Yellow)),
                Span::raw(input.to_string()),
            ])
        };
        f.render_widget(Paragraph::new(prompt_line), chunks[1]);

        // Preview line
        let sample = if input.is_empty() { "GH-123" } else { input };
        let preview = render_branch_name(template, sample);
        let preview_line = Line::from(vec![
            Span::styled("Branch: ", Style::default().fg(Color::Gray)),
            Span::styled(
                preview,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        f.render_widget(Paragraph::new(preview_line), chunks[2]);

        // Instructions/status line
        let status = Paragraph::new("Enter: Confirm  •  Esc: Cancel  •  Backspace: Delete")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Left);
        f.render_widget(status, chunks[3]);
    }
}
