---
phase: 02-tui-shell
plan: 01
subsystem: ui
tags: [ratatui, crossterm, rust, tui, state-management]

# Dependency graph
requires:
  - phase: 01-search-core
    provides: SearchResult enum, SearchHandle type, debounced_search function
provides:
  - AppState struct with all TUI runtime state
  - TabKind enum (Files, Folders, Contents, AiConversations)
  - ratatui and crossterm in Cargo.toml for Plans 02 and 03
affects: [02-tui-shell-02, 02-tui-shell-03]

# Tech tracking
tech-stack:
  added: [ratatui 0.29, crossterm 0.28 with event-stream]
  patterns: [central AppState struct pattern, mpsc channel lifecycle owned by AppState]

key-files:
  created: [src/app.rs]
  modified: [Cargo.toml, src/lib.rs]

key-decisions:
  - "crate-local imports (crate::types::SearchResult, crate::search::SearchHandle) used in app.rs instead of rsight:: — app.rs is inside the crate, not a dependent"
  - "Default trait implemented for AppState delegating to new() for ergonomic construction"
  - "mpsc channel with capacity 4096 created inside AppState::new() — state owns both ends"

patterns-established:
  - "AppState: central TUI state struct owns all result vecs, selection index, query string, and channel endpoints"
  - "push_result: routes SearchResult to correct tab vec by variant match, enforces 100-cap, sorts by score"
  - "active_results: returns &[SearchResult] slice for the currently selected tab"

requirements-completed: [TUI-01, TUI-02, TUI-03, TUI-04]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 2 Plan 01: TUI State Types and Dependency Setup Summary

**ratatui 0.29 + crossterm 0.28 added, AppState/TabKind defined with mpsc channel ownership and per-tab result routing**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-18T18:55:56Z
- **Completed:** 2026-03-18T18:57:50Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- ratatui 0.29 and crossterm 0.28 (event-stream) added to Cargo.toml — dependency graph verified via cargo check
- AppState struct defines all TUI runtime state: query, active_tab, files, folders, contents, selected_index, search_handle, result_tx, result_rx, status_message, should_quit
- TabKind enum with 4 variants, display labels, is_enabled guard (AiConversations disabled until Phase 3)
- AppState::new() creates fresh state with mpsc channel at 4096 capacity
- push_result routes results to correct tab with 100-item cap and score-descending sort for Files/Folders

## Task Commits

Each task was committed atomically:

1. **Task 1: Add ratatui and crossterm to Cargo.toml** - `c7c9b54` (chore)
2. **Task 2: Define AppState and TabKind in src/app.rs** - `170e016` (feat)

## Files Created/Modified
- `src/app.rs` - AppState struct, TabKind enum, impl blocks with new/active_results/clear_results/push_result/Default
- `Cargo.toml` - ratatui 0.29 and crossterm 0.28 with event-stream feature added
- `src/lib.rs` - pub mod app added, AppState and TabKind re-exported

## Decisions Made
- Used `crate::types::SearchResult` and `crate::search::SearchHandle` in app.rs — the file is part of the rsight crate so external crate path doesn't apply
- Implemented `Default` for `AppState` delegating to `new()` — follows Rust idiom and makes struct construction ergonomic
- mpsc channel with 4096 capacity owned inside AppState — no external channel management needed by callers

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed unresolved import `rsight` inside the crate itself**
- **Found during:** Task 2 (Define AppState and TabKind)
- **Issue:** Plan template used `use rsight::{...}` but app.rs is a module inside the rsight crate — that import path causes E0432 at compile time
- **Fix:** Changed to `use crate::types::SearchResult` and `use crate::search::SearchHandle` — correct intra-crate import paths
- **Files modified:** src/app.rs
- **Verification:** `cargo check` exits 0 after fix
- **Committed in:** `170e016` (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - compile error from incorrect import path in plan template)
**Impact on plan:** Fix required for compilation. No scope creep. Public API unchanged.

## Issues Encountered
- Plan template used `use rsight::{...}` import style, but app.rs is compiled as part of the rsight crate itself — intra-crate imports must use `crate::` prefix. Fixed automatically via Rule 1.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- AppState and TabKind types are stable contracts for Plans 02 (rendering) and 03 (event loop)
- ratatui and crossterm available to all crate modules
- Plan 02 can import `use rsight::{AppState, TabKind}` directly (re-exported from lib.rs)
- AiConversations tab is stubbed as disabled — Phase 3 enables it

---
*Phase: 02-tui-shell*
*Completed: 2026-03-18*
