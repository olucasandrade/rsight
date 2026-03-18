use ratatui::{
    Frame,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs},
};
use crate::app::{AppState, TabKind};
use crate::types::{SearchResult, AiSource};
use super::layout::build_layout;
use super::highlight::highlight_spans;

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

    let items: Vec<ListItem> = results
        .iter()
        .map(|result| make_list_item(result, &app.query, terminal_width))
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

fn make_list_item(result: &SearchResult, query: &str, max_width: usize) -> ListItem<'static> {
    match result {
        SearchResult::File { name, path, .. } | SearchResult::Folder { name, path, .. } => {
            // Highlight the name portion; path is always dimmed plain text
            let name_spans = highlight_spans(name.clone(), query);
            let separator = Span::raw("  ");
            let path_truncated = truncate_path(path, max_width.saturating_sub(name.len() + 2));
            let path_span = Span::styled(
                path_truncated,
                Style::default().fg(Color::DarkGray),
            );
            let mut spans = name_spans;
            spans.push(separator);
            spans.push(path_span);
            ListItem::new(Line::from(spans))
        }
        SearchResult::ContentMatch { path, line_number, line } => {
            // Format: "path:line_number  " prefix (plain) + highlighted snippet
            let prefix = format!("{}:{}  ", path, line_number);
            let snippet = line.trim().to_string();
            let remaining = max_width.saturating_sub(prefix.len());
            let snippet_truncated = if snippet.len() > remaining {
                snippet[..remaining].to_string()
            } else {
                snippet
            };
            let prefix_span = Span::styled(
                prefix,
                Style::default().fg(Color::DarkGray),
            );
            let snippet_spans = highlight_spans(snippet_truncated, query);
            let mut spans = vec![prefix_span];
            spans.extend(snippet_spans);
            ListItem::new(Line::from(spans))
        }
        SearchResult::AiConversation { title, date, source, .. } => {
            // Format: highlighted title + dim " · " + dim date + dim source badge
            let title_spans = highlight_spans(title.clone(), query);
            let separator = Span::styled(
                "  ·  ",
                Style::default().fg(Color::DarkGray),
            );
            let date_span = Span::styled(
                date.clone(),
                Style::default().fg(Color::DarkGray),
            );
            let source_badge = match source {
                AiSource::ClaudeCode => Span::styled(" [Claude]", Style::default().fg(Color::DarkGray)),
                AiSource::Cursor => Span::styled(" [Cursor]", Style::default().fg(Color::DarkGray)),
            };
            let mut spans = title_spans;
            spans.push(separator);
            spans.push(date_span);
            spans.push(source_badge);
            ListItem::new(Line::from(spans))
        }
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_len.saturating_sub(3))..])
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
