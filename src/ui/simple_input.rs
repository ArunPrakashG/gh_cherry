use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{prelude::*, widgets::*};
use std::io;

pub struct SimpleInput;

impl SimpleInput {
    /// Prompt for a single line of input using a minimal TUI (no boxes/borders).
    /// Returns Some(input) on Enter, None on Esc/cancel.
    pub fn prompt(title: &str, initial: &str, placeholder: &str) -> Result<Option<String>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let mut terminal = ratatui::Terminal::new(backend)?;

        let mut input = initial.to_string();

        let result = loop {
            terminal.draw(|f| Self::render(f, title, &input, placeholder))?;

            if event::poll(std::time::Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Enter => break Ok(Some(input.trim().to_string())),
                            KeyCode::Esc => break Ok(None),
                            KeyCode::Backspace => {
                                input.pop();
                            }
                            KeyCode::Char(c) => input.push(c),
                            _ => {}
                        }
                    }
                }
            }
        };

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn render(f: &mut Frame, title: &str, input: &str, placeholder: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // title
                Constraint::Length(2), // input
                Constraint::Length(1), // hint
                Constraint::Min(0),
            ])
            .split(f.area());

        let title_p = Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left);
        f.render_widget(title_p, chunks[0]);

        let content = if input.is_empty() {
            Line::from(vec![
                Span::styled("› ", Style::default().fg(Color::Yellow)),
                Span::styled(placeholder, Style::default().fg(Color::DarkGray).italic()),
            ])
        } else {
            Line::from(vec![
                Span::styled("› ", Style::default().fg(Color::Yellow)),
                Span::raw(input.to_string()),
            ])
        };
        f.render_widget(Paragraph::new(content), chunks[1]);

        let hint = Paragraph::new("Enter: Confirm  •  Esc: Cancel  •  Backspace: Delete")
            .style(Style::default().fg(Color::Gray));
        f.render_widget(hint, chunks[2]);
    }
}
