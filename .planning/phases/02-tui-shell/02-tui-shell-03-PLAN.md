---
phase: 02-tui-shell
plan: 03
type: standard
wave: 2
depends_on:
  - 02-tui-shell-01-PLAN.md
files_modified:
  - src/main.rs
  - src/event_loop.rs
  - src/lib.rs
autonomous: true
must_haves:
  - src/event_loop.rs contains run_app(app) async fn implementing the ratatui event loop
  - Keyboard input updates AppState.query and triggers debounced_search with 150ms delay
  - Arrow keys move AppState.selected_index up/down, clamped to result list bounds
  - Tab / Shift+Tab cycles between enabled tabs (skips AiConversations)
  - Esc sets AppState.should_quit = true
  - mpsc result channel drains into AppState via push_result on each iteration
  - src/main.rs replaced with tokio::main that initializes terminal, runs run_app, restores terminal
  - cargo check passes
requirements:
  - TUI-01
  - TUI-02
  - TUI-03
---

# Phase 2 Plan 03: Event Loop and Search Wiring

**Objective:** Replace the CLI smoke-test `main.rs` with a full ratatui event loop that reads keyboard input, drives `debounced_search`, drains search results into `AppState`, and re-renders every frame.

## Context

@src/app.rs — AppState, TabKind (Plan 01)
@src/search.rs — debounced_search signature: `async fn debounced_search(root: String, query: String, result_tx: mpsc::Sender<SearchResult>, delay_ms: u64) -> SearchHandle`
@src/main.rs — current CLI stub to replace

Event loop decisions from CONTEXT.md:
- debounced_search called with 150ms delay; previous SearchHandle aborted on each keystroke
- keypress → abort old handle → start new debounced_search(HOME, query, result_tx.clone(), 150)
- result_rx drains into AppState.push_result on each loop tick (non-blocking try_recv loop)
- Tab / Shift+Tab switches tabs; wraps around; skips AiConversations
- Arrow down → selected_index += 1 (clamped to list length - 1)
- Arrow up → selected_index = selected_index.saturating_sub(1)
- active_tab resets to Files when query is cleared (empty) and user retypes

## Tasks

<task type="auto">
  <name>Task 1: Create src/event_loop.rs with the ratatui event loop</name>
  <files>src/event_loop.rs, src/lib.rs</files>
  <action>
Create `src/event_loop.rs`. This is the core TUI runtime — it owns the terminal, drives rendering, processes keyboard events, and wires the search API.

```rust
use std::io;
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::app::{AppState, TabKind};
use crate::ui::draw_ui;
use rsight::debounced_search;

const SEARCH_DELAY_MS: u64 = 150;
const TICK_MS: u64 = 16; // ~60fps polling interval

/// Initialize the terminal for full-screen TUI mode.
pub fn init_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

/// Restore terminal to its original state. Call on exit (including panic cleanup).
pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

/// Main TUI event loop. Runs until app.should_quit is true.
pub async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut AppState,
) -> io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());

    loop {
        // 1. Drain any pending search results into AppState (non-blocking)
        loop {
            match app.result_rx.try_recv() {
                Ok(result) => app.push_result(result),
                Err(_) => break, // no more results right now
            }
        }

        // 2. Render current state
        terminal.draw(|frame| draw_ui(frame, app))?;

        // 3. Poll for keyboard events (non-blocking, 16ms timeout = ~60fps)
        if event::poll(Duration::from_millis(TICK_MS))? {
            if let Event::Key(key) = event::read()? {
                // Ignore key release events (crossterm sends both press and release on some platforms)
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key.code, key.modifiers, &home).await;
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

async fn handle_key(app: &mut AppState, code: KeyCode, modifiers: KeyModifiers, home: &str) {
    match (code, modifiers) {
        // Quit
        (KeyCode::Esc, _) => {
            app.should_quit = true;
        }

        // Tab navigation
        (KeyCode::Tab, KeyModifiers::NONE) => {
            cycle_tab(app, Direction::Forward);
        }
        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
            cycle_tab(app, Direction::Backward);
        }

        // Result navigation
        (KeyCode::Down, _) => {
            let len = app.active_results().len();
            if len > 0 {
                app.selected_index = (app.selected_index + 1).min(len - 1);
            }
        }
        (KeyCode::Up, _) => {
            app.selected_index = app.selected_index.saturating_sub(1);
        }

        // Text input
        (KeyCode::Char(c), KeyModifiers::NONE) | (KeyCode::Char(c), KeyModifiers::SHIFT) => {
            let was_empty = app.query.is_empty();
            app.query.push(c);
            // Reset to Files tab when user starts typing after clearing query
            if was_empty {
                app.active_tab = TabKind::Files;
            }
            app.clear_results();
            trigger_search(app, home).await;
        }
        (KeyCode::Backspace, _) => {
            app.query.pop();
            app.clear_results();
            if !app.query.is_empty() {
                trigger_search(app, home).await;
            } else {
                // Query cleared — abort any running search
                if let Some(handle) = app.search_handle.take() {
                    handle.abort();
                }
            }
        }

        // Ignore everything else (Enter and Ctrl+C handled in Plan 04)
        _ => {}
    }
}

async fn trigger_search(app: &mut AppState, home: &str) {
    // Abort the previous debounced search task
    if let Some(handle) = app.search_handle.take() {
        handle.abort();
    }
    // Start a new debounced search
    let handle = debounced_search(
        home.to_string(),
        app.query.clone(),
        app.result_tx.clone(),
        SEARCH_DELAY_MS,
    )
    .await;
    app.search_handle = Some(handle);
}

enum Direction { Forward, Backward }

fn cycle_tab(app: &mut AppState, dir: Direction) {
    let tabs = TabKind::all();
    // Only cycle among enabled tabs
    let enabled: Vec<TabKind> = tabs.iter().filter(|t| t.is_enabled()).copied().collect();
    if enabled.is_empty() { return; }

    let current_pos = enabled.iter().position(|t| *t == app.active_tab).unwrap_or(0);
    let next_pos = match dir {
        Direction::Forward => (current_pos + 1) % enabled.len(),
        Direction::Backward => (current_pos + enabled.len() - 1) % enabled.len(),
    };
    app.active_tab = enabled[next_pos];
    app.selected_index = 0;
}
```

Note: `Direction` is a local enum — do NOT import ratatui's Direction here (different type). If there's a name conflict, rename to `TabDirection` or `NavDir`.

Add `pub mod event_loop;` to `src/lib.rs`. Do NOT re-export run_app from lib.rs (it's called directly from main).

Run `cargo check` after writing.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo check 2>&1 | tail -10</automated>
  </verify>
  <done>cargo check passes. src/event_loop.rs exists with run_app, init_terminal, restore_terminal functions.</done>
</task>

<task type="auto">
  <name>Task 2: Replace src/main.rs with the TUI entry point</name>
  <files>src/main.rs</files>
  <action>
Replace the current CLI smoke-test `src/main.rs` with the TUI entry point:

```rust
use rsight::event_loop::{init_terminal, restore_terminal, run_app};
use rsight::app::AppState;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize ratatui terminal (raw mode, alternate screen)
    let mut terminal = init_terminal()?;

    // Set up a panic hook to restore the terminal before printing panic info.
    // Without this, a panic leaves the terminal in raw mode.
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Best-effort terminal restore (ignore errors)
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::LeaveAlternateScreen
        );
        original_hook(panic_info);
    }));

    let mut app = AppState::new();
    let result = run_app(&mut terminal, &mut app).await;

    // Always restore terminal, even on error
    restore_terminal(&mut terminal)?;

    result
}
```

After writing, run `cargo check` and then `cargo build` to confirm the binary links correctly.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo build 2>&1 | tail -10</automated>
  </verify>
  <done>cargo build exits 0. The rsight binary is built. src/main.rs no longer contains the CLI smoke test.</done>
</task>

## Verification

```bash
cd /Users/lucasandrade/rsight && cargo build
```

Exit 0 = success. The built binary is at `target/debug/rsight`.

## Success Criteria

- `cargo build` passes with 0 errors
- `run_app` processes keyboard events and updates AppState correctly
- Typing characters appends to query, clears results, triggers debounced_search with 150ms
- Backspace removes the last character from the query
- Arrow keys update selected_index, clamped to result list bounds
- Tab / Shift+Tab cycles through Files → Folders → Contents (skips AI Conversations)
- Esc sets should_quit = true
- mpsc result_rx is drained into AppState via push_result each iteration
- Terminal is properly restored on exit (alternate screen dismissed)

## Output

- `src/event_loop.rs` — init_terminal, restore_terminal, run_app, handle_key, trigger_search, cycle_tab
- `src/main.rs` — tokio entry point, panic hook, AppState init, run_app call, terminal restore
- `src/lib.rs` — updated with `pub mod event_loop`
