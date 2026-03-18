---
phase: 02-tui-shell
plan: 01
type: standard
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - src/app.rs
autonomous: true
must_haves:
  - ratatui and crossterm appear in Cargo.toml dependencies
  - AppState struct exists in src/app.rs with fields: query (String), active_tab (TabKind), files (Vec<SearchResult>), folders (Vec<SearchResult>), contents (Vec<SearchResult>), selected_index (usize), search_handle (Option<SearchHandle>)
  - TabKind enum has four variants: Files, Folders, Contents, AiConversations
  - src/app.rs compiles without errors (cargo check passes)
requirements:
  - TUI-01
  - TUI-02
  - TUI-03
  - TUI-04
---

# Phase 2 Plan 01: TUI State Types and Dependency Setup

**Objective:** Add ratatui and crossterm to Cargo.toml and define the central `AppState` type that drives all TUI rendering and event handling. This is the interface-first step — Plans 02 and 03 build against these contracts.

## Context

@Cargo.toml — add ratatui and crossterm
@src/lib.rs — re-exports to import (SearchResult, SearchHandle, debounced_search)
@src/types.rs — SearchResult enum variants for reference
@src/search.rs — SearchHandle = JoinHandle<()>

Phase 1 established:
- `rsight::SearchResult` enum: `File { path, name, score }`, `Folder { path, name, score }`, `ContentMatch { path, line_number, line }`
- `rsight::debounced_search(root, query, tx, delay_ms) -> SearchHandle`
- `rsight::SearchHandle` = `tokio::task::JoinHandle<()>`

## Tasks

<task type="auto">
  <name>Task 1: Add ratatui and crossterm to Cargo.toml</name>
  <files>Cargo.toml</files>
  <action>
Add to the [dependencies] section of Cargo.toml:

```toml
ratatui = "0.29"
crossterm = { version = "0.28", features = ["event-stream"] }
```

Use ratatui 0.29 (latest stable as of early 2026). crossterm with event-stream feature enables async terminal event reading via EventStream, which integrates cleanly with tokio.

Do NOT add any additional dependencies beyond these two. Do NOT modify existing dependencies.

After editing, run `cargo check` to confirm the crate graph resolves without errors.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>cargo check exits 0. Cargo.toml contains ratatui and crossterm entries under [dependencies].</done>
</task>

<task type="auto">
  <name>Task 2: Define AppState and TabKind in src/app.rs</name>
  <files>src/app.rs</files>
  <action>
Create `src/app.rs` with the following types. This is the single source of truth for all TUI state — Plans 02 (rendering) and 03 (event loop) both import from here.

```rust
use tokio::sync::mpsc;
use rsight::{SearchResult, SearchHandle, debounced_search};

/// Which tab is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    Files,
    Folders,
    Contents,
    AiConversations,
}

impl TabKind {
    /// All tabs in display order.
    pub fn all() -> &'static [TabKind] {
        &[
            TabKind::Files,
            TabKind::Folders,
            TabKind::Contents,
            TabKind::AiConversations,
        ]
    }

    /// Display label for the tab bar.
    pub fn label(&self) -> &'static str {
        match self {
            TabKind::Files => "Files",
            TabKind::Folders => "Folders",
            TabKind::Contents => "Contents",
            TabKind::AiConversations => "AI Conversations",
        }
    }

    /// AI Conversations tab is disabled until Phase 3.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, TabKind::AiConversations)
    }
}

/// All runtime state for the TUI application.
pub struct AppState {
    /// Current search query string (mirrors what the user typed).
    pub query: String,
    /// Active tab. Resets to Files when query is cleared and retyped.
    pub active_tab: TabKind,
    /// File name search results (capped at 100, sorted by fuzzy score desc).
    pub files: Vec<SearchResult>,
    /// Folder name search results (capped at 100, sorted by fuzzy score desc).
    pub folders: Vec<SearchResult>,
    /// Content match results (capped at 100, in discovery order).
    pub contents: Vec<SearchResult>,
    /// Selected row index within the active tab's result list.
    pub selected_index: usize,
    /// Handle for the running debounced search task. Abort on new keystroke.
    pub search_handle: Option<SearchHandle>,
    /// mpsc sender for search results. TUI drains the paired receiver.
    pub result_tx: mpsc::Sender<SearchResult>,
    /// Receiver for incoming search results from debounced_search.
    pub result_rx: mpsc::Receiver<SearchResult>,
    /// Status message shown in the status bar (e.g. "AI search coming in a future update").
    pub status_message: Option<String>,
    /// Whether the application should exit on the next loop iteration.
    pub should_quit: bool,
}

impl AppState {
    /// Create a fresh AppState. Call once at startup.
    pub fn new() -> Self {
        let (result_tx, result_rx) = mpsc::channel(4096);
        AppState {
            query: String::new(),
            active_tab: TabKind::Files,
            files: Vec::new(),
            folders: Vec::new(),
            contents: Vec::new(),
            selected_index: 0,
            search_handle: None,
            result_tx,
            result_rx,
            status_message: None,
            should_quit: false,
        }
    }

    /// Returns the result list for the currently active tab.
    pub fn active_results(&self) -> &[SearchResult] {
        match self.active_tab {
            TabKind::Files => &self.files,
            TabKind::Folders => &self.folders,
            TabKind::Contents => &self.contents,
            TabKind::AiConversations => &[], // disabled in Phase 2
        }
    }

    /// Clear all search results and reset selection. Call when query changes.
    pub fn clear_results(&mut self) {
        self.files.clear();
        self.folders.clear();
        self.contents.clear();
        self.selected_index = 0;
        self.status_message = None;
    }

    /// Push a SearchResult into the correct tab list.
    /// Enforces the 100-result cap per tab.
    pub fn push_result(&mut self, result: SearchResult) {
        const CAP: usize = 100;
        match &result {
            SearchResult::File { .. } => {
                if self.files.len() < CAP {
                    self.files.push(result);
                    // Sort by score descending (higher score = better match)
                    self.files.sort_by(|a, b| {
                        let score_a = if let SearchResult::File { score, .. } = a { score.unwrap_or(0) } else { 0 };
                        let score_b = if let SearchResult::File { score, .. } = b { score.unwrap_or(0) } else { 0 };
                        score_b.cmp(&score_a)
                    });
                }
            }
            SearchResult::Folder { .. } => {
                if self.folders.len() < CAP {
                    self.folders.push(result);
                    self.folders.sort_by(|a, b| {
                        let score_a = if let SearchResult::Folder { score, .. } = a { score.unwrap_or(0) } else { 0 };
                        let score_b = if let SearchResult::Folder { score, .. } = b { score.unwrap_or(0) } else { 0 };
                        score_b.cmp(&score_a)
                    });
                }
            }
            SearchResult::ContentMatch { .. } => {
                if self.contents.len() < CAP {
                    self.contents.push(result);
                }
            }
        }
    }
}
```

Also add `pub mod app;` to `src/lib.rs` and `pub use app::AppState; pub use app::TabKind;` re-exports so Plans 02/03 can import cleanly.

After writing src/app.rs and updating src/lib.rs, run `cargo check` to confirm compilation.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo check 2>&1 | tail -10</automated>
  </verify>
  <done>cargo check exits 0. src/app.rs exists. AppState and TabKind are importable via `use rsight::{AppState, TabKind};`.</done>
</task>

## Verification

```bash
cd /Users/lucasandrade/rsight && cargo check
```

Exit 0 = success. Both ratatui and crossterm appear in Cargo.lock.

## Success Criteria

- `cargo check` passes with no errors
- `src/app.rs` defines `AppState` and `TabKind` as described
- `TabKind::all()` returns 4 variants in display order
- `AppState::new()` creates a usable initial state
- Plans 02 and 03 can import `AppState` and `TabKind` without modification to this file

## Output

- `Cargo.toml` — ratatui 0.29 + crossterm 0.28 with event-stream feature added
- `src/app.rs` — AppState struct, TabKind enum, impl blocks with new/active_results/clear_results/push_result
- `src/lib.rs` — updated with `pub mod app` and re-exports
