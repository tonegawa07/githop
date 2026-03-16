use crate::app::{App, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(f.area());

    let main_area = chunks[0];
    let message_area = chunks[1];
    let help_area = chunks[2];

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_area);

    draw_branch_list(f, app, main_chunks[0]);
    draw_preview(f, app, main_chunks[1]);
    draw_message_bar(f, app, message_area);
    draw_help_bar(f, app, help_area);
}

fn draw_branch_list(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_indices();
    let items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .map(|(i, &idx)| {
            let branch = &app.branches[idx];
            let marker = if branch.is_current { "* " } else { "  " };
            let text = format!("{}{}", marker, branch.name);

            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if branch.is_current {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(Line::from(Span::styled(text, style)))
        })
        .collect();

    let title = if app.filter.is_empty() {
        format!(" Branches ({}) ", filtered.len())
    } else {
        format!(" Branches ({}) [filter: {}] ", filtered.len(), app.filter)
    };

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(list, area);
}

fn draw_preview(f: &mut Frame, app: &App, area: Rect) {
    let title = match app.selected_branch() {
        Some(b) => format!(" Preview: {} ", b.name),
        None => " Preview ".to_string(),
    };

    let lines: Vec<Line> = app
        .preview_commits
        .iter()
        .map(|c| Line::from(c.as_str()))
        .collect();

    let preview = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(preview, area);
}

fn draw_message_bar(f: &mut Frame, app: &App, area: Rect) {
    let (content, style) = match &app.input_mode {
        InputMode::Filter => (
            format!("/{}█", app.filter),
            Style::default().fg(Color::Cyan),
        ),
        InputMode::Create => (
            format!(
                "{} > {}█",
                app.status_message.as_deref().unwrap_or("New branch:"),
                app.input_buf
            ),
            Style::default().fg(Color::Cyan),
        ),
        InputMode::Rename => (
            format!("Rename to: {}█", app.input_buf),
            Style::default().fg(Color::Cyan),
        ),
        InputMode::Confirm => (
            app.status_message.clone().unwrap_or_default(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        InputMode::Normal => (
            app.status_message.clone().unwrap_or_default(),
            Style::default().fg(Color::Green),
        ),
    };

    let bar = Paragraph::new(Line::from(Span::styled(content, style)));
    f.render_widget(bar, area);
}

fn draw_help_bar(f: &mut Frame, app: &App, area: Rect) {
    let help = match app.input_mode {
        InputMode::Confirm => " [y/n] or [d/n] see above",
        InputMode::Filter => " [Enter] apply  [Esc] cancel",
        InputMode::Create | InputMode::Rename => " [Enter] confirm  [Esc] cancel",
        InputMode::Normal => " [Enter]switch [y]copy [d]delete [n]new [r]rename [/]filter [q]quit",
    };

    let bar = Paragraph::new(Line::from(Span::styled(
        help,
        Style::default().fg(Color::DarkGray),
    )));
    f.render_widget(bar, area);
}
