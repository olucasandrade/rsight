pub mod claude;
pub mod cursor;

pub use claude::search_claude_conversations;
pub use cursor::search_cursor_conversations;

use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc;
use crate::types::SearchResult;

/// Search both Claude Code and Cursor conversation histories.
/// Results are forwarded to the provided tokio mpsc sender.
/// Both parsers share an internal std::sync::mpsc channel since they are synchronous.
pub fn search_ai_conversations(query: &str, tx: mpsc::Sender<SearchResult>) {
    let (std_tx, std_rx) = std_mpsc::channel::<SearchResult>();
    search_claude_conversations(query, std_tx.clone());
    search_cursor_conversations(query, std_tx);
    // Forward all results to the tokio sender
    for result in std_rx {
        if tx.blocking_send(result).is_err() {
            break; // receiver dropped — cancel search
        }
    }
}
