use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};
use crate::app::{AppState, TabKind};
use crate::types::SearchResult;
use super::layout::build_layout;

/// Render the entire UI from current AppState. Called every frame.
pub fn draw_ui(frame: &mut Frame, app: &AppState) {
    let [search_area, tab_area, results_area, status_area] = build_layout(frame.area());

    draw_search_bar(frame, app, search_area);
    draw_tab_bar(frame, app, tab_area);
    draw_results(frame, app, results_area);
    draw_status_bar(frame, app, status_area);
}

fn draw_search_bar(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    // Show query with a trailing cursor character
    let display = format!("{}_", app.query);
    let paragraph = Paragraph::new(display)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search "),
        );
    frame.render_widget(paragraph, area);
}

fn draw_tab_bar(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let tab_titles: Vec<Line> = TabKind::all()
        .iter()
        .map(|tab| {
            let style = if *tab == app.active_tab {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if !tab.is_enabled() {
                // AI Conversations: grayed out
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };
            Line::from(Span::styled(tab.label(), style))
        })
        .collect();

    let active_idx = TabKind::all()
        .iter()
        .position(|t| *t == app.active_tab)
        .unwrap_or(0);

    let tabs = Tabs::new(tab_titles)
        .select(active_idx)
        .block(Block::default().borders(Borders::ALL))
        .divider("|");

    frame.render_widget(tabs, area);
}

fn draw_results(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let results = app.active_results();
    let terminal_width = area.width.saturating_sub(4) as usize; // subtract borders + padding

    if !app.active_tab.is_enabled() {
        // Disabled tab: show placeholder message
        let paragraph = Paragraph::new("AI Conversations search coming in a future update.")
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center);
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = results
        .iter()
        .map(|result| {
            let line = format_result(result, terminal_width);
            ListItem::new(line)
        })
        .collect();

    let block = Block::default().borders(Borders::ALL);
    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    if !results.is_empty() {
        list_state.select(Some(app.selected_index.min(results.len().saturating_sub(1))));
    }

    frame.render_stateful_widget(list, area, &mut list_state);
}

fn format_result(result: &SearchResult, max_width: usize) -> String {
    match result {
        SearchResult::File { name, path, .. } | SearchResult::Folder { name, path, .. } => {
            // "name  dimmed_full_path" — truncate path if too wide
            let prefix = format!("{}  ", name);
            let remaining = max_width.saturating_sub(prefix.len());
            let path_display = if path.len() > remaining {
                format!("...{}", &path[path.len().saturating_sub(remaining.saturating_sub(3))..])
            } else {
                path.clone()
            };
            format!("{}{}", prefix, path_display)
        }
        SearchResult::ContentMatch { path, line_number, line } => {
            // "path:line_number  snippet" — truncate to terminal width
            let prefix = format!("{}:{}  ", path, line_number);
            let remaining = max_width.saturating_sub(prefix.len());
            let snippet = line.trim();
            let snippet_display = if snippet.len() > remaining {
                &snippet[..remaining]
            } else {
                snippet
            };
            format!("{}{}", prefix, snippet_display)
        }
    }
}

fn draw_status_bar(frame: &mut Frame, app: &AppState, area: ratatui::layout::Rect) {
    let result_count = app.active_results().len();

    let text = if let Some(ref msg) = app.status_message {
        msg.clone()
    } else {
        format!(
            "{}  ↑↓ navigate  Tab switch tab  Enter open  Ctrl+C copy  Esc quit",
            if result_count == 1 {
                "1 result".to_string()
            } else {
                format!("{} results", result_count)
            }
        )
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(paragraph, area);
}
