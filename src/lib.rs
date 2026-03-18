pub mod types;
pub use types::SearchResult;

use tokio::sync::mpsc;

/// Stub search function — implemented in Plans 02-04.
/// Returns an mpsc receiver that will yield SearchResult items.
pub async fn search(_root: &str, _query: &str) -> mpsc::Receiver<SearchResult> {
    let (tx, rx) = mpsc::channel(1024);
    drop(tx); // immediately closed — stub produces no results
    rx
}
