pub mod claude;
pub mod cursor;
pub mod codex;

pub use claude::search_claude_conversations;
pub use cursor::search_cursor_conversations;
pub use codex::search_codex_conversations;

use std::sync::mpsc as std_mpsc;
use std::thread;
use tokio::sync::mpsc;
use crate::types::SearchResult;

/// Search Claude Code and Cursor conversation histories.
/// Results are forwarded to the provided tokio mpsc sender.
pub fn search_ai_conversations(query: &str, tx: mpsc::Sender<SearchResult>) {
    let (std_tx, std_rx) = std_mpsc::channel::<SearchResult>();

    let query_claude = query.to_string();
    let std_tx_claude = std_tx.clone();
    let h1 = thread::spawn(move || {
        search_claude_conversations(&query_claude, std_tx_claude);
    });

    let query_cursor = query.to_string();
    let std_tx_cursor = std_tx.clone();
    let h2 = thread::spawn(move || {
        search_cursor_conversations(&query_cursor, std_tx_cursor);
    });

    let query_codex = query.to_string();
    let std_tx_codex = std_tx.clone();
    let h3 = thread::spawn(move || {
        search_codex_conversations(&query_codex, std_tx_codex);
    });

    // Drop the original sender so the channel closes when all threads finish
    drop(std_tx);

    for result in std_rx {
        if tx.blocking_send(result).is_err() {
            break;
        }
    }

    let _ = h1.join();
    let _ = h2.join();
    let _ = h3.join();
}
