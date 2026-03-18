---
phase: 02-tui-shell
plan: 04
type: standard
wave: 3
depends_on:
  - 02-tui-shell-02-PLAN.md
  - 02-tui-shell-03-PLAN.md
files_modified:
  - src/ui/highlight.rs
  - src/ui/render.rs
  - src/ui/mod.rs
  - src/actions.rs
  - src/event_loop.rs
  - src/lib.rs
autonomous: true
must_haves:
  - src/ui/highlight.rs exists with highlight_spans(text, query) -> Vec<Span> function
  - File/Folder result rows show matched characters in yellow/bold in the rendered list
  - ContentMatch snippet shows matched substring in yellow/bold
  - Enter on a File or Folder result calls `open` (macOS system default) on the path
  - Enter on a ContentMatch result opens $EDITOR at the matching line if $EDITOR is set, otherwise falls back to `open`
  - Ctrl+C copies the selected result's full path to the system clipboard
  - Pressing Enter on AiConversations tab sets AppState.status_message to "AI search coming in a future update"
  - cargo build passes
requirements:
  - TUI-03
  - TUI-04
---

# Phase 2 Plan 04: Match Highlighting and Open Actions

**Objective:** Add match highlighting (yellow bold spans for matched characters in result rows) and implement the open/copy actions triggered by Enter and Ctrl+C.

## Context

@src/ui/render.rs — format_result and draw_results (Plan 02) — to be updated for highlighted spans
@src/event_loop.rs — handle_key (Plan 03) — to be updated with Enter/Ctrl+C branches
@src/app.rs — AppState, active_results(), status_message field

Decisions from CONTEXT.md:
- Match highlighting: Claude's discretion — highlight matched characters bold+yellow in filenames and snippets
- Enter: opens result with system default (`open` on macOS); for ContentMatch with $EDITOR set, uses `$EDITOR +LINE path`; fall back to `open` if $EDITOR not set
- Ctrl+C: copies full path to clipboard
- Pressing Enter on disabled AI Conversations tab: sets status_message = "AI search coming in a future update"
- Clipboard: use `pbcopy` on macOS (pipe path to stdin)

## Tasks

<task type="auto">
  <name>Task 1: Implement match highlighting spans</name>
  <files>src/ui/highlight.rs, src/ui/render.rs, src/ui/mod.rs</files>
  <action>
**Step 1: Create src/ui/highlight.rs**

Match highlighting strategy: find all occurrences of query characters in text using a simple greedy substring match (find the query as a substring, bold+yellow that portion; if no substring match found, return the text as-is with no highlight). This is lightweight and covers the most common case.

```rust
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
```

**Step 2: Update src/ui/mod.rs** — add `pub mod highlight;` and `pub use highlight::highlight_spans;`.

**Step 3: Update src/ui/render.rs** — modify `draw_results` and `format_result` to use highlighted `Line` items instead of plain strings.

Replace the `format_result` function and the `items` construction in `draw_results` with a `make_list_item` function that returns `ListItem` with styled `Line`:

```rust
use super::highlight::highlight_spans;
use ratatui::text::Line;

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
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        path.to_string()
    } else {
        format!("...{}", &path[path.len().saturating_sub(max_len.saturating_sub(3))..])
    }
}
```

Update the items construction in `draw_results`:
```rust
let terminal_width = area.width.saturating_sub(4) as usize;
let items: Vec<ListItem> = results
    .iter()
    .map(|result| make_list_item(result, &app.query, terminal_width))
    .collect();
```

Remove the old `format_result` function entirely.

Run `cargo check` after all three file updates.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo check 2>&1 | tail -10</automated>
  </verify>
  <done>cargo check passes. highlight_spans is exported from src/ui/. Result rows use Line with Span instead of plain strings.</done>
</task>

<task type="auto">
  <name>Task 2: Implement open and copy actions (Enter and Ctrl+C)</name>
  <files>src/actions.rs, src/event_loop.rs, src/lib.rs</files>
  <action>
**Step 1: Create src/actions.rs**

```rust
use std::process::Command;
use crate::types::SearchResult;

/// Open a search result with the appropriate application.
/// - File/Folder: uses macOS `open` command (system default app)
/// - ContentMatch: uses $EDITOR at line number if set, else falls back to `open`
pub fn open_result(result: &SearchResult) {
    match result {
        SearchResult::File { path, .. } | SearchResult::Folder { path, .. } => {
            let _ = Command::new("open").arg(path).spawn();
        }
        SearchResult::ContentMatch { path, line_number, .. } => {
            if let Ok(editor) = std::env::var("EDITOR") {
                // Open at line number: e.g. `vim +8 /path/to/file`
                let line_arg = format!("+{}", line_number);
                let _ = Command::new(&editor)
                    .arg(&line_arg)
                    .arg(path)
                    .spawn();
            } else {
                // No $EDITOR set — fall back to system default
                let _ = Command::new("open").arg(path).spawn();
            }
        }
    }
}

/// Copy a path string to the macOS clipboard via pbcopy.
pub fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    if let Ok(mut child) = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Extract the full path from any SearchResult variant.
pub fn result_path(result: &SearchResult) -> &str {
    match result {
        SearchResult::File { path, .. }
        | SearchResult::Folder { path, .. }
        | SearchResult::ContentMatch { path, .. } => path.as_str(),
    }
}
```

**Step 2: Add `pub mod actions;` to src/lib.rs.**

**Step 3: Update src/event_loop.rs — add Enter and Ctrl+C branches to handle_key.**

In the `match (code, modifiers)` block in `handle_key`, add:

```rust
// Open selected result
(KeyCode::Enter, _) => {
    if !app.active_tab.is_enabled() {
        // Disabled AI tab: show status message
        app.status_message = Some("AI search coming in a future update".to_string());
    } else {
        let results = app.active_results();
        if !results.is_empty() {
            let idx = app.selected_index.min(results.len() - 1);
            open_result(&results[idx]);
        }
    }
}

// Copy path to clipboard
(KeyCode::Char('c'), KeyModifiers::CONTROL) => {
    let results = app.active_results();
    if !results.is_empty() {
        let idx = app.selected_index.min(results.len() - 1);
        let path = result_path(&results[idx]);
        copy_to_clipboard(path);
        app.status_message = Some(format!("Copied: {}", path));
    }
}
```

Add imports at the top of `src/event_loop.rs`:
```rust
use crate::actions::{open_result, copy_to_clipboard, result_path};
```

Note: `app.active_results()` returns `&[SearchResult]`. Since you need to call open_result (which borrows the result) and potentially mutate app.status_message afterward, clone the result first to avoid borrow conflicts:
```rust
let result = results[idx].clone();
open_result(&result);
// or
let path = result_path(&results[idx]).to_string();
copy_to_clipboard(&path);
app.status_message = Some(format!("Copied: {}", path));
```

Run `cargo build` to confirm everything links.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo build 2>&1 | tail -10</automated>
  </verify>
  <done>cargo build exits 0. src/actions.rs exists with open_result and copy_to_clipboard. Enter and Ctrl+C are handled in event_loop.rs.</done>
</task>

## Verification

```bash
cd /Users/lucasandrade/rsight && cargo build
```

Exit 0 = success.

## Success Criteria

- `cargo build` passes with 0 errors
- highlight_spans returns yellow+bold spans for matched substrings in result rows
- File/Folder rows: filename has match highlighted, path is dimmed
- ContentMatch rows: path:line prefix is dimmed, snippet has match highlighted
- Enter on File/Folder: spawns `open <path>`
- Enter on ContentMatch: spawns `$EDITOR +<line> <path>` if $EDITOR set, else `open <path>`
- Ctrl+C: pipes path to pbcopy, sets status_message to "Copied: <path>"
- Enter on AI Conversations tab: sets status_message to "AI search coming in a future update"

## Output

- `src/ui/highlight.rs` — highlight_spans(text, query) -> Vec<Span>
- `src/ui/render.rs` — updated: make_list_item with highlighted spans, truncate_path
- `src/ui/mod.rs` — updated: pub mod highlight added
- `src/actions.rs` — open_result, copy_to_clipboard, result_path
- `src/event_loop.rs` — updated: Enter and Ctrl+C branches in handle_key
- `src/lib.rs` — updated: pub mod actions added
