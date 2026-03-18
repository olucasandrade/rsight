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
use crate::search::debounced_search;
use crate::actions::{open_result, copy_to_clipboard, result_path};

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
            cycle_tab(app, NavDir::Forward);
        }
        (KeyCode::BackTab, _) | (KeyCode::Tab, KeyModifiers::SHIFT) => {
            cycle_tab(app, NavDir::Backward);
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
            app.query.push(c);
            // Active tab is intentionally NOT reset here — the user's current tab
            // (Files, Contents, Folders) is preserved when the query changes.
            // Tab switching is only driven by explicit Tab/Shift+Tab keypresses.
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

        // Open selected result
        (KeyCode::Enter, _) => {
            if !app.active_tab.is_enabled() {
                // Disabled AI tab: show status message
                app.status_message = Some("AI search coming in a future update".to_string());
            } else {
                let results = app.active_results();
                if !results.is_empty() {
                    let idx = app.selected_index.min(results.len() - 1);
                    let result = results[idx].clone();
                    open_result(&result);
                }
            }
        }

        // Copy path to clipboard
        (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            let results = app.active_results();
            if !results.is_empty() {
                let idx = app.selected_index.min(results.len() - 1);
                let path = result_path(&results[idx]).to_string();
                copy_to_clipboard(&path);
                app.status_message = Some(format!("Copied: {}", path));
            }
        }

        // Ignore everything else
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

enum NavDir { Forward, Backward }

fn cycle_tab(app: &mut AppState, dir: NavDir) {
    let tabs = TabKind::all();
    // Only cycle among enabled tabs
    let enabled: Vec<TabKind> = tabs.iter().filter(|t| t.is_enabled()).copied().collect();
    if enabled.is_empty() { return; }

    let current_pos = enabled.iter().position(|t| *t == app.active_tab).unwrap_or(0);
    let next_pos = match dir {
        NavDir::Forward => (current_pos + 1) % enabled.len(),
        NavDir::Backward => (current_pos + enabled.len() - 1) % enabled.len(),
    };
    app.active_tab = enabled[next_pos];
    app.selected_index = 0;
}
