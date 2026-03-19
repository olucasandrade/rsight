pub mod claude;
// pub mod cursor; // not yet supported

pub use claude::search_claude_conversations;

use std::sync::mpsc as std_mpsc;
use tokio::sync::mpsc;
use crate::types::SearchResult;

/// Search Claude Code conversation histories scoped to the current working directory.
/// Results are forwarded to the provided tokio mpsc sender.
pub fn search_ai_conversations(query: &str, tx: mpsc::Sender<SearchResult>) {
    let (std_tx, std_rx) = std_mpsc::channel::<SearchResult>();
    search_claude_conversations(query, std_tx);
    for result in std_rx {
        if tx.blocking_send(result).is_err() {
            break;
        }
    }
}
