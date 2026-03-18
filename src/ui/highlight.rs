use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};

/// Return a Vec<Span> representing `text` with the first occurrence of `query`
/// highlighted in yellow bold. If query is empty or not found, returns a single
/// plain Span for the whole text.
pub fn highlight_spans<'a>(text: String, query: &str) -> Vec<Span<'a>> {
    if query.is_empty() {
        return vec![Span::raw(text)];
    }

    let lower_text = text.to_lowercase();
    let lower_query = query.to_lowercase();

    if let Some(pos) = lower_text.find(&lower_query) {
        let before = text[..pos].to_string();
        let matched = text[pos..pos + lower_query.len()].to_string();
        let after = text[pos + lower_query.len()..].to_string();

        let mut spans = Vec::new();
        if !before.is_empty() {
            spans.push(Span::raw(before));
        }
        spans.push(Span::styled(
            matched,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        if !after.is_empty() {
            spans.push(Span::raw(after));
        }
        spans
    } else {
        vec![Span::raw(text)]
    }
}
