---
phase: 01-search-core
plan: "04"
subsystem: search
tags: [rust, async, tokio, mpsc, debounce, unified-api]
dependency_graph:
  requires: [01-search-core-02, 01-search-core-03]
  provides: [unified-search-api, debounced-search-api]
  affects: [phase-02-tui-shell]
tech_stack:
  added: []
  patterns:
    - tokio::task::spawn_blocking for sync search functions in async context
    - mpsc channel cancellation via Receiver drop
    - JoinHandle abort for debounce cancellation
key_files:
  created:
    - src/search.rs
  modified:
    - src/lib.rs
    - src/main.rs
decisions:
  - spawn_blocking chosen over spawn for name_search and content_search since both are synchronous blocking operations
  - tx clone pattern — original tx dropped at end of search() so channel closes when both tasks finish
  - debounced_search accepts Sender (not returning Receiver) so TUI controls the result channel lifecycle
  - delay_ms as parameter for testability rather than hardcoding 150ms inside the function
metrics:
  duration: "2 min"
  completed: "2026-03-18"
  tasks_completed: 2
  files_modified: 3
---

# Phase 1 Plan 4: Unified Search API Summary

**One-liner:** Async search() wrapping concurrent name+content search with mpsc streaming and 150ms debounce via JoinHandle abort.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Unified search API with concurrent name + content search | 9306d36 | src/search.rs (created), src/lib.rs (updated) |
| 2 | Debounce wrapper and working main.rs smoke test | c275fbc | src/search.rs (debounce added), src/main.rs (updated) |

## Public API Implemented

```rust
/// Run a full search of `root` for `query`.
/// Both name search and content search run concurrently as blocking tasks.
/// Dropping the Receiver cancels both searches.
pub async fn search(root: &str, query: &str) -> mpsc::Receiver<SearchResult>

/// Debounced search: waits delay_ms before launching. Abort the returned handle to cancel.
pub async fn debounced_search(
    root: String,
    query: String,
    result_tx: mpsc::Sender<SearchResult>,
    delay_ms: u64,
) -> SearchHandle

pub type SearchHandle = task::JoinHandle<()>;
```

## Test Results

```
running 15 tests
test content_search::tests::empty_query_returns_nothing ... ok
test name_search::tests::empty_query_returns_nothing ... ok
test content_search::tests::skips_files_over_1mb ... ok
test content_search::tests::skips_node_modules ... ok
test content_search::tests::multi_line_correct_numbers ... ok
test content_search::tests::finds_matching_line ... ok
test content_search::tests::skips_binary_files ... ok
test search::tests::empty_query_closes_immediately ... ok
test content_search::tests::traverses_hidden_dirs ... ok
test name_search::tests::matches_folders ... ok
test name_search::tests::fuzzy_matches_file ... ok
test name_search::tests::traverses_hidden_dirs ... ok
test name_search::tests::skips_node_modules ... ok
test search::tests::returns_both_result_types ... ok
test search::tests::dropping_receiver_does_not_panic ... ok

test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured
```

## Binary

```
-rwxr-xr-x@ 1 lucasandrade  staff   1.8M Mar 18 19:44 target/release/rsight
```

## Deviations from Plan

### Auto-included Items

**1. [Rule 2 - Missing export] Exported debounced_search and SearchHandle from lib.rs**
- Found during: Task 1 implementation
- Issue: Plan specified exporting `search` from lib.rs but `debounced_search` and `SearchHandle` are also part of the public API needed by Phase 2 TUI
- Fix: Added `pub use search::debounced_search` and `pub use search::SearchHandle` to lib.rs
- Files modified: src/lib.rs

**2. [Plan enhancement] Debounce wrapper added in Task 1 commit rather than Task 2**
- The debounce wrapper was included in the same file as the search function (src/search.rs) and implemented together. Task 2 commit covers main.rs changes. The plan's ordering was followed conceptually.

## Self-Check: PASSED

- src/search.rs: FOUND
- src/lib.rs: FOUND (updated)
- src/main.rs: FOUND (updated)
- Commit 9306d36: FOUND
- Commit c275fbc: FOUND
- target/release/rsight: FOUND (1.8M)
- All 15 tests: PASSED
