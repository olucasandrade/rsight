---
phase: 01-search-core
plan: 03
subsystem: search
tags: [rust, rayon, ignore, mpsc, content-search, binary-detection, filesystem]

# Dependency graph
requires:
  - phase: 01-search-core
    plan: 01
    provides: SearchResult enum with ContentMatch variant, Cargo workspace with ignore/rayon/tokio deps

provides:
  - search_contents(root, query, tx) exported from src/lib.rs
  - Binary file detection via null-byte heuristic on first 8KB
  - File size guard (1MB max)
  - Hidden directory traversal (dotfiles included, SRCH-04)
  - Excluded dirs: node_modules, .git, target, vendor, build
  - Parallel file reads via rayon

affects: [01-search-core-04]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Two-pass collect-then-parallel: WalkBuilder collects paths, rayon par_iter searches contents"
    - "blocking_send from rayon threads into tokio mpsc channel"
    - "Null-byte heuristic on first 8KB for binary detection"

key-files:
  created:
    - src/content_search.rs
  modified:
    - src/lib.rs

key-decisions:
  - "Two-pass strategy (collect paths, then parallel search) avoids WalkBuilder lifetime issues with rayon"
  - "blocking_send used from rayon threads since they are not inside a tokio runtime"
  - "filter_entry used for directory exclusion (prunes entire subtree, more efficient than per-file check)"

patterns-established:
  - "TDD RED/GREEN: write failing tests first, commit, then implement to green"
  - "rayon par_iter + tokio mpsc::blocking_send for CPU-parallel streaming results"

requirements-completed: [SRCH-02, SRCH-04]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 1 Plan 03: Content Search Summary

**Full-text content search over filesystem using rayon-parallel reads, null-byte binary detection, and 1MB size guard — streaming ContentMatch results through mpsc channel**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-03-18T18:38:39Z
- **Completed:** 2026-03-18T18:40:22Z
- **Tasks:** 1 (TDD with RED + GREEN commits)
- **Files modified:** 2

## Accomplishments
- `search_contents(root, query, tx)` walks entire directory tree including hidden dirs
- Binary files detected and skipped using null-byte heuristic on first 8KB
- Files over 1MB skipped entirely via size guard applied at walk time
- Excluded dirs (node_modules, .git, target, vendor, build) pruned at directory level
- Parallel content reads via `rayon::par_iter` with results streamed through `mpsc::blocking_send`
- All 7 tests pass covering: match, binary skip, size guard, hidden dirs, node_modules exclusion, empty query, multi-line line numbers

## Task Commits

Each task was committed atomically:

1. **Task 1 RED (tests): Content search with binary detection and size guard** - `5a57d29` (test)
2. **Task 1 GREEN (impl): Content search with binary detection and size guard** - `a30f1e8` (feat)

_TDD task with RED commit then GREEN commit._

## Test Results

```
running 7 tests
test content_search::tests::empty_query_returns_nothing ... ok
test content_search::tests::skips_binary_files ... ok
test content_search::tests::finds_matching_line ... ok
test content_search::tests::multi_line_correct_numbers ... ok
test content_search::tests::skips_node_modules ... ok
test content_search::tests::traverses_hidden_dirs ... ok
test content_search::tests::skips_files_over_1mb ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.01s
```

## Function Signature

```rust
pub fn search_contents(root: &str, query: &str, tx: mpsc::Sender<SearchResult>)
```

Exported from `src/lib.rs` as `pub use content_search::search_contents`.

## Files Created/Modified

- `src/content_search.rs` — Full-text content search implementation with 7 inline tests
- `src/lib.rs` — Added `pub mod content_search` and `pub use content_search::search_contents`

## Decisions Made

- Two-pass strategy (collect paths first, then `rayon::par_iter`) chosen over streaming parallel walk to avoid WalkBuilder lifetime issues with rayon threads
- `blocking_send` used from rayon threads (not inside tokio runtime) to push results into mpsc channel
- `filter_entry` used for excluded directory pruning — prunes entire subtree at walk time, more efficient than per-file checks

## Deviations from Plan

None — plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness

- `search_contents` is ready to be called by the unified search API in Plan 04
- Exports: `pub fn search_contents(root: &str, query: &str, tx: mpsc::Sender<SearchResult>)` from `src/lib.rs`
- ContentMatch results include path, 1-based line_number, and trimmed line text

---
*Phase: 01-search-core*
*Completed: 2026-03-18*

## Self-Check: PASSED

- FOUND: src/content_search.rs
- FOUND: src/lib.rs
- FOUND: 01-search-core-03-SUMMARY.md
- FOUND commit: 5a57d29 (RED)
- FOUND commit: a30f1e8 (GREEN)
