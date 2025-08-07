use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::ui::state::AppState;

pub struct MainMenu;

impl MainMenu {
    pub fn render(f: &mut Frame, _state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new("🍒 GitHub Cherry-Pick TUI")
            .style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Menu items
        let menu_items = vec![
            "1. View and Cherry-pick PRs",
            "r. Refresh data",
            "q. Quit",
        ];

        let menu: Vec<ListItem> = menu_items
            .iter()
            .map(|item| ListItem::new(*item))
            .collect();

        let menu_list = List::new(menu)
            .block(Block::default().title("Menu").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Yellow).fg(Color::Black));

        f.render_widget(menu_list, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Use numbers to select options, 'q' to quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[2]);
    }
}

pub struct PrList;

impl PrList {
    pub fn render(f: &mut Frame, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new(format!("📋 Pull Requests ({} found)", state.prs.len()))
            .style(Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // PR List
        if state.prs.is_empty() {
            let empty_message = Paragraph::new("No PRs found matching the criteria.\n\nPress 'r' to refresh or 'Esc' to go back.")
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: true })
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(empty_message, chunks[1]);
        } else {
            let items: Vec<ListItem> = state
                .prs
                .iter()
                .map(|pr| {
                    let style = if pr.labels.contains(&"cherry picked".to_string()) {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    };

                    let content = format!(
                        "#{} - {} (by {} - {} commits)",
                        pr.number,
                        pr.title,
                        pr.author,
                        pr.commits.len()
                    );

                    ListItem::new(content).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().title("PRs").borders(Borders::ALL))
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                );

            let mut list_state = ratatui::widgets::ListState::default();
            list_state.select(state.pr_list_state.selected());
            f.render_stateful_widget(list, chunks[1], &mut list_state);
        }

        // Instructions
        let instructions = Paragraph::new("↑/↓: Navigate | Enter: Cherry-pick | r: Refresh | Esc: Back | q: Quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(instructions, chunks[2]);

        // Show success message if any
        if let Some(message) = &state.success_message {
            Self::render_popup(f, message, Color::Green);
        }
    }

    fn render_popup(f: &mut Frame, message: &str, color: Color) {
        let popup_area = Self::centered_rect(60, 20, f.area());
        f.render_widget(Clear, popup_area);
        let popup = Paragraph::new(message)
            .style(Style::default().fg(color))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(color)),
            );
        f.render_widget(popup, popup_area);
    }

    fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }
}

pub struct ProgressView;

impl ProgressView {
    pub fn render(f: &mut Frame, state: &AppState) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),
                Constraint::Min(5),
            ])
            .split(f.area());

        // Title
        let title = Paragraph::new("⏳ Processing...")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Progress bar (indeterminate)
        let progress = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(50) // Static for now, could be animated
            .label("Working...");
        f.render_widget(progress, chunks[1]);

        // Status message
        let message = state
            .loading_message
            .as_deref()
            .unwrap_or("Please wait...");
        
        let status = Paragraph::new(message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, chunks[2]);
    }
}
