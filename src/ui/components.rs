use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Gauge, List, ListItem, Paragraph, Wrap},
    text::{Line, Span},
    Frame,
};

use crate::ui::state::AppState;
use crate::config::Config;

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
        let title = Paragraph::new("🍒 GitHub Cherry-Pick")
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Minimal prompt-like menu (no boxes)
        let menu_text = ">> Press Enter to view PRs  •  r: Refresh  •  q: Quit";
        let menu_para = Paragraph::new(menu_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        f.render_widget(menu_para, chunks[1]);

        // Instructions
        let instructions = Paragraph::new("Use numbers to select options, 'q' to quit")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[2]);
    }
}

pub struct PrList;

impl PrList {
    pub fn render(f: &mut Frame, state: &AppState, config: &Config) {
    let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
        Constraint::Length(1),  // header
        Constraint::Length(1),  // prompt bar
        Constraint::Min(8),     // list
        Constraint::Length(1),  // status/instructions
            ])
            .split(f.area());

        // Title
        let total = state.prs.len();
        let shown = state.display_indices.len();
        let title = Paragraph::new(format!(
                "📋 Pull Requests  —  showing {} of {}",
                shown, total
            ))
            .style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Inline prompt bar (minimal, no boxes)
        let prompt_line = if state.input_active {
            let input = if state.input_buffer.is_empty() {
                Line::from(vec![
                    Span::styled(">> ", Style::default().fg(Color::Yellow)),
                    Span::styled(
                        state.input_placeholder.as_str(),
                        Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(">> ", Style::default().fg(Color::Yellow)),
                    Span::raw(state.input_buffer.clone()),
                ])
            };
            Paragraph::new(vec![
                Line::from(Span::styled(state.input_title.clone(), Style::default().fg(Color::Cyan))),
                input,
            ])
        } else {
            let hint = match &state.filter_query {
                Some(q) => format!("f: Filter (active: '{}')  •  Enter: Cherry-pick  •  Esc: Back", q),
                None => "f: Filter  •  Enter: Cherry-pick  •  Esc: Back".to_string(),
            };
            Paragraph::new(Line::from(vec![
                Span::styled(">> ", Style::default().fg(Color::Yellow)),
                Span::raw(hint),
            ]))
        };
        f.render_widget(prompt_line, chunks[1]);

        // PR List
    if shown == 0 {
            let criteria_info = format!(
                "No PRs found matching the criteria.\n\n\
                📋 Search Criteria:\n\
                • Repository: {}/{}\n\
                • Base Branch: {}\n\
                • Environment: {}\n\
                • Pending Tag: \"{}\"\n\
                • Days Back: {}\n\n\
                💡 Tips:\n\
                • Ensure PRs are tagged with \"{}\"\n\
                • Check if PRs are merged to \"{}\" branch\n\
                • Verify the tag pattern matches: {}\n\n\
                🔄 Press 'r' to refresh or 'Esc' to go back.",
                config.github.owner,
                config.github.repo,
                config.github.base_branch,
                config.tags.environment,
                config.tags.pending_tag,
                config.ui.days_back,
                config.tags.pending_tag,
                config.github.base_branch,
                config.tags.sprint_pattern
            );
            
            let empty_message = Paragraph::new(criteria_info)
                .style(Style::default().fg(Color::Gray))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            f.render_widget(empty_message, chunks[2]);
        } else {
            let items: Vec<ListItem> = state
                .display_indices
                .iter()
                .map(|&idx| {
                    let pr = &state.prs[idx];
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
                .highlight_style(
                    Style::default()
                        .bg(Color::Yellow)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD),
                );

            let mut list_state = ratatui::widgets::ListState::default();
            list_state.select(state.pr_list_state.selected());
            f.render_stateful_widget(list, chunks[2], &mut list_state);
        }

    // Instructions
    let mut status = String::new();
        if let Some(message) = &state.success_message {
            status.push_str(message);
            status.push_str("   •   ");
        }
        status.push_str("↑/↓ Navigate  •  Enter Cherry-pick  •  r Refresh  •  f Filter  •  Esc Back  •  q Quit");
        let instructions = Paragraph::new(status)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(instructions, chunks[3]);

    // Popups removed for a cleaner, less "boxy" look
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
            .style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center);
        f.render_widget(title, chunks[0]);

        // Progress bar (indeterminate)
        let progress = Gauge::default()
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(50) // Static for now, could be animated
            .label("Working...");
        f.render_widget(progress, chunks[1]);

        // Status message
        let message = state.loading_message.as_deref().unwrap_or("Please wait...");

        let status = Paragraph::new(message)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(status, chunks[2]);
    }
}
