---
phase: 02-tui-shell
plan: 02
type: standard
wave: 2
depends_on:
  - 02-tui-shell-01-PLAN.md
files_modified:
  - src/ui/mod.rs
  - src/ui/layout.rs
  - src/ui/render.rs
  - src/lib.rs
autonomous: true
must_haves:
  - src/ui/ module exists with mod.rs, layout.rs, render.rs
  - draw_ui(frame, app) function renders search bar, tab bar, results list, status bar top-to-bottom
  - Search bar shows the current query with a cursor indicator
  - Tab bar renders all four tabs; active tab visually distinguished; AI Conversations grayed out
  - Results list renders File/Folder results as "name  dimmed_path" and ContentMatch as "path:line  snippet"
  - Status bar shows result count and key hints
  - cargo check passes
requirements:
  - TUI-01
  - TUI-02
  - TUI-03
  - TUI-04
---

# Phase 2 Plan 02: TUI Layout and Rendering

**Objective:** Build all ratatui rendering code — search bar, tab bar, results list, status bar — as pure functions that take `&AppState` and produce frame draws. No event handling here; rendering only.

## Context

@src/app.rs — AppState, TabKind (from Plan 01)
@src/types.rs — SearchResult enum

Rendering decisions from CONTEXT.md:
- Layout top-to-bottom: search bar → tabs → results list → status bar (full terminal takeover)
- File/Folder display: `filename  ~/full/path/to/file` (name bold, path dimmed)
- ContentMatch display: `path:line_number  snippet` truncated to terminal width
- Tab bar: 4 tabs, AI Conversations grayed out, active tab visually distinct
- Status bar: "N results  ↑↓ navigate  Tab switch tab  Enter open  Ctrl+C copy  Esc quit"
- Style: minimal, mostly text, subtle borders, no heavy colors (match highlights are Plan 04)
- Selected row: reverse video highlight

## Tasks

<task type="auto">
  <name>Task 1: Create src/ui/ module with layout constants and frame structure</name>
  <files>src/ui/mod.rs, src/ui/layout.rs, src/lib.rs</files>
  <action>
Create the `src/ui/` directory and its module files.

**src/ui/mod.rs:**
```rust
pub mod layout;
pub mod render;

pub use render::draw_ui;
```

**src/ui/layout.rs:**
```rust
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
```

**src/lib.rs update:** Add `pub mod ui;` and `pub use ui::draw_ui;`.

Run `cargo check` after creating files.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo check 2>&1 | tail -10</automated>
  </verify>
  <done>cargo check passes. src/ui/mod.rs and src/ui/layout.rs exist. build_layout is callable.</done>
</task>

<task type="auto">
  <name>Task 2: Implement draw_ui — search bar, tab bar, results list, status bar</name>
  <files>src/ui/render.rs</files>
  <action>
Create `src/ui/render.rs` with the full `draw_ui` function. This is the only rendering entry point — called from the event loop in Plan 03.

```rust
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
```

After writing, run `cargo check`. Fix any import path issues (ratatui 0.29 may have minor API differences — adjust imports as needed to pass cargo check, keeping the same visual structure).
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo check 2>&1 | tail -10</automated>
  </verify>
  <done>cargo check passes. src/ui/render.rs exists. draw_ui function is exported from src/ui/mod.rs.</done>
</task>

## Verification

```bash
cd /Users/lucasandrade/rsight && cargo check
```

## Success Criteria

- `cargo check` passes with 0 errors
- `draw_ui(frame, &app_state)` renders 4 regions top-to-bottom
- File/Folder results show name + dimmed path
- ContentMatch results show path:line  snippet
- Active tab highlighted in yellow bold; AI Conversations grayed out
- Selected result row uses reverse-video / blue background highlight
- Status bar shows result count and key hints

## Output

- `src/ui/mod.rs`
- `src/ui/layout.rs` — build_layout() splits terminal into 4 regions
- `src/ui/render.rs` — draw_ui() + sub-functions for each region
- `src/lib.rs` — updated with `pub mod ui`
