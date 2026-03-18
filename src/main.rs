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
