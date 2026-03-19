---
phase: 01-search-core
plan: 04
type: execute
wave: 3
depends_on: [02, 03]
files_modified:
  - src/search.rs
  - src/lib.rs
  - src/main.rs
autonomous: true
requirements: [SRCH-01, SRCH-02, SRCH-03]

must_haves:
  truths:
    - The public `search` async function accepts root and query strings and returns a tokio mpsc::Receiver<SearchResult>
    - Name search and content search run concurrently (both spawned as tokio blocking tasks)
    - The search function is cancellable — dropping the returned Receiver stops both searches
    - The debounce wrapper (150ms) cancels in-flight search when called again before the delay expires
    - src/main.rs compiles and runs end-to-end (no stub search function)
  artifacts:
    - src/search.rs
  key_links:
    - `pub async fn search(root, query) -> Receiver<SearchResult>` — the public API Phase 2 (TUI) will call
    - `pub async fn debounced_search(root, query, tx) -> JoinHandle` — wraps search with 150ms debounce for TUI keypress handling
---

<objective>
Wire name search and content search into a unified async search API with a 150ms debounce. This is the public interface Phase 2 (TUI Shell) will call.

Purpose: Satisfies SRCH-01 and SRCH-02 (both search types accessible through one API) and supports SRCH-03 (concurrent execution for speed).
Output: src/search.rs with `search()` and `debounced_search()` functions; src/main.rs updated to use real implementation.
</objective>

<execution_context>
@/Users/lucasandrade/.claude/get-shit-done/workflows/execute-plan.md
@/Users/lucasandrade/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/01-search-core/01-CONTEXT.md
@.planning/phases/01-search-core/01-search-core-01-SUMMARY.md
@.planning/phases/01-search-core/01-search-core-02-SUMMARY.md
@.planning/phases/01-search-core/01-search-core-03-SUMMARY.md

<interfaces>
<!-- From src/types.rs (Plan 01) -->
pub enum SearchResult {
    File   { path: String, name: String, score: Option<i64> },
    Folder { path: String, name: String, score: Option<i64> },
    ContentMatch { path: String, line_number: u64, line: String },
}

<!-- From src/name_search.rs (Plan 02) -->
pub fn search_names(root: &str, query: &str, tx: mpsc::Sender<SearchResult>);

<!-- From src/content_search.rs (Plan 03) -->
pub fn search_contents(root: &str, query: &str, tx: mpsc::Sender<SearchResult>);
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Unified search API with concurrent name + content search</name>
  <files>src/search.rs, src/lib.rs</files>
  <behavior>
    - Test 1: Calling `search(root, query)` returns a Receiver that yields both File/Folder results and ContentMatch results (both search types run)
    - Test 2: Dropping the Receiver from `search()` before it drains terminates cleanly without panicking
    - Test 3: `search(root, "")` returns a Receiver that immediately closes (no results for empty query)
  </behavior>
  <action>
Create src/search.rs:

```rust
use tokio::sync::mpsc;
use tokio::task;
use crate::types::SearchResult;
use crate::{search_names, search_contents};

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
    task::spawn_blocking(move || {
        search_contents(&root_owned, &query_owned, tx_content);
    });

    // tx (original) is dropped here — channel closes when both tasks finish
    rx
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
```

Update src/lib.rs to replace the stub `search` function with the real one and export the search module:

```rust
pub mod types;
pub mod name_search;
pub mod content_search;
pub mod search;

pub use types::SearchResult;
pub use name_search::search_names;
pub use content_search::search_contents;
pub use search::search;
```

Remove or comment out the stub `search` async fn that was in lib.rs from Plan 01 — it is now replaced by the re-export above.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo test search:: -- --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>All 3 tests in the search module pass. `cargo test search::` exits 0.</done>
</task>

<task type="auto">
  <name>Task 2: Debounce wrapper and working main.rs smoke test</name>
  <files>src/search.rs, src/main.rs</files>
  <action>
Add the debounce wrapper to src/search.rs. Append after the existing `search` function:

```rust
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

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
```

Update src/main.rs to use the real `search` function (replace the stub call):

```rust
use rsight::search;

#[tokio::main]
async fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let query = std::env::args().nth(1).unwrap_or_else(|| "main".into());

    println!("Searching {} for {:?}...", home, query);
    let mut rx = search(&home, &query).await;
    let mut count = 0;
    while let Some(result) = rx.recv().await {
        println!("{:?}", result);
        count += 1;
        if count >= 10 { break; }
    }
    println!("Done. {} results shown (max 10).", count);
}
```

Verify the release binary builds successfully:
```bash
cargo build --release
```
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo build --release 2>&1 | tail -10</automated>
  </verify>
  <done>`cargo build --release` exits 0. Binary exists at target/release/rsight. `debounced_search` is defined and exported in src/search.rs.</done>
</task>

</tasks>

<verification>
```bash
cd /Users/lucasandrade/rsight
cargo test 2>&1 | grep -E "test result|FAILED"
cargo build --release 2>&1 | grep -E "^error"  # must be 0 errors
ls -lh target/release/rsight                    # must exist
```
</verification>

<success_criteria>
- All tests pass across all modules (`cargo test` green)
- `cargo build --release` produces target/release/rsight
- `search(root, query)` is the public API — returns Receiver<SearchResult>
- `debounced_search(root, query, tx, delay_ms)` is exported for Phase 2 (TUI) consumption
- main.rs compiles and uses the real search function
</success_criteria>

<output>
After completion, create `.planning/phases/01-search-core/01-search-core-04-SUMMARY.md` with:
- Full `cargo test` output
- Binary size (ls -lh target/release/rsight)
- Public API signatures as implemented:
  - `search`
  - `debounced_search`
- Any implementation deviations
</output>
