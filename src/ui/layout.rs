use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Split the full terminal area into the four vertical regions.
/// Returns [search_bar, tab_bar, results, status_bar].
pub fn build_layout(area: Rect) -> [Rect; 4] {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // search bar (border + 1 line content)
            Constraint::Length(3), // tab bar (border + 1 line content)
            Constraint::Min(0),    // results list (fills remaining space)
            Constraint::Length(1), // status bar (single line, no border)
        ])
        .split(area);
    [chunks[0], chunks[1], chunks[2], chunks[3]]
}
