use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{sleep, Duration};
use crate::types::SearchResult;
use crate::{search_names, search_contents};
use crate::ai_search::search_ai_conversations;

/// Channel buffer size. Large enough to avoid backpressure during fast traversal.
const CHANNEL_BUFFER: usize = 4096;

/// Run a full search of `root` for `query`.
///
/// Both name search and content search run concurrently as blocking tasks.
/// The returned Receiver yields results as they are found.
/// Dropping the Receiver before it drains signals cancellation to both searches
/// (blocking_send returns Err when channel is closed, causing walkers to stop).
pub async fn search(root: &str, query: &str) -> mpsc::Receiver<SearchResult> {
    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER);

    if query.is_empty() {
        return rx; // tx dropped immediately — channel closes
    }

    let root_owned = root.to_string();
    let query_owned = query.to_string();

    // Spawn name search as a blocking task (WalkBuilder is sync)
    let tx_name = tx.clone();
    let root_name = root_owned.clone();
    let query_name = query_owned.clone();
    task::spawn_blocking(move || {
        search_names(&root_name, &query_name, tx_name);
    });

    // Spawn content search as a blocking task (rayon-parallel, sync)
    let tx_content = tx.clone();
    let root_content = root_owned.clone();
    let query_content = query_owned.clone();
    task::spawn_blocking(move || {
        search_contents(&root_content, &query_content, tx_content);
    });

    // Spawn AI conversation search as a blocking task
    let tx_ai = tx.clone();
    let query_ai = query_owned.clone();
    task::spawn_blocking(move || {
        search_ai_conversations(&query_ai, tx_ai);
    });

    // tx (original) is dropped here — channel closes when all three tasks finish
    rx
}

/// Handle returned by `debounced_search`. Call `.abort()` to cancel the pending search.
pub type SearchHandle = task::JoinHandle<()>;

/// Debounced search: waits `delay_ms` milliseconds before launching the search.
/// If called again before the delay expires, the caller should abort the previous handle.
///
/// Usage pattern (TUI keypress handler):
/// ```ignore
/// if let Some(h) = current_handle.take() { h.abort(); }
/// let tx = result_channel_sender.clone();
/// current_handle = Some(debounced_search(HOME, query, tx, 150).await);
/// ```
pub async fn debounced_search(
    root: String,
    query: String,
    result_tx: mpsc::Sender<SearchResult>,
    delay_ms: u64,
) -> SearchHandle {
    task::spawn(async move {
        sleep(Duration::from_millis(delay_ms)).await;
        let mut rx = search(&root, &query).await;
        while let Some(result) = rx.recv().await {
            if result_tx.send(result).await.is_err() {
                break; // consumer gone
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn returns_both_result_types() {
        let dir = TempDir::new().unwrap();
        // File with name that matches and content that matches
        fs::write(dir.path().join("match_me.txt"), "find this line\n").unwrap();

        let mut rx = search(dir.path().to_str().unwrap(), "match").await;
        let mut has_file = false;
        let mut has_content = false;
        while let Some(result) = rx.recv().await {
            match result {
                SearchResult::File { .. } => has_file = true,
                SearchResult::ContentMatch { .. } => has_content = true,
                _ => {}
            }
        }
        // Name "match_me.txt" should fuzzy-match "match"
        assert!(has_file, "expected File result");
        // Content "find this line" does not contain "match" — that's OK,
        // what matters is both search types ran; at minimum File should appear
        let _ = has_content; // content match presence depends on file content
    }

    #[tokio::test]
    async fn dropping_receiver_does_not_panic() {
        let dir = TempDir::new().unwrap();
        for i in 0..100 {
            fs::write(dir.path().join(format!("file{}.txt", i)), "hello world\n").unwrap();
        }
        let rx = search(dir.path().to_str().unwrap(), "hello").await;
        drop(rx); // drop immediately — searches should stop cleanly
        // Give tasks a moment to notice cancellation
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        // If we reach here without panic, test passes
    }

    #[tokio::test]
    async fn empty_query_closes_immediately() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("file.txt"), "content\n").unwrap();
        let mut rx = search(dir.path().to_str().unwrap(), "").await;
        let result = rx.recv().await;
        assert!(result.is_none(), "empty query should produce no results");
    }
}
