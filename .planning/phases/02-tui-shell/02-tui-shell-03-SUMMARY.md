---
phase: 02-tui-shell
plan: 03
subsystem: ui
tags: [ratatui, crossterm, tokio, event-loop, tui, keyboard]

# Dependency graph
requires:
  - phase: 02-tui-shell-01
    provides: AppState, TabKind, SearchHandle types with mpsc channel ownership
  - phase: 02-tui-shell-02
    provides: draw_ui rendering function (search bar, tab bar, results list, status bar)
  - phase: 01-search-core
    provides: debounced_search async fn, SearchHandle abort pattern
provides:
  - event_loop::run_app — async TUI loop that reads keyboard, renders, drains results
  - event_loop::init_terminal — crossterm raw mode + alternate screen setup
  - event_loop::restore_terminal — terminal cleanup on exit
  - main.rs TUI entry point with panic hook for terminal restore
affects: [02-tui-shell-04, 02-tui-shell-05]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Poll-render loop at 60fps (16ms TICK_MS) with non-blocking try_recv to drain mpsc results
    - Debounce pattern: abort previous JoinHandle, spawn new debounced_search on each keystroke
    - NavDir local enum avoids ratatui Direction name conflict
    - Panic hook restores terminal before re-panicking for clean crash recovery

key-files:
  created:
    - src/event_loop.rs
  modified:
    - src/main.rs
    - src/lib.rs

key-decisions:
  - "crate::search::debounced_search used instead of rsight:: — event_loop.rs is inside the crate"
  - "NavDir enum named to avoid collision with ratatui::layout::Direction"
  - "KeyEventKind::Press filter prevents duplicate events on platforms that send press+release"

patterns-established:
  - "Poll-then-render loop: drain mpsc, draw frame, poll keyboard — all non-blocking"
  - "handle_key is async to allow .await on trigger_search without blocking the loop"

requirements-completed: [TUI-01, TUI-02, TUI-03]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 2 Plan 03: Event Loop and Search Wiring Summary

**Full ratatui TUI event loop wired to debounced_search: keyboard input drives 150ms debounced file/folder/content search with mpsc result streaming into AppState at 60fps**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-18T19:20:30Z
- **Completed:** 2026-03-18T19:22:30Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `src/event_loop.rs` implements the full ratatui event loop: renders at 60fps, drains mpsc result channel each frame, processes keyboard events non-blocking
- Keyboard handling: char/backspace update query and trigger debounced search, arrows navigate selected_index (clamped), Tab/BackTab cycle enabled tabs, Esc sets should_quit
- `src/main.rs` replaced with tokio entry point: init_terminal, panic hook for terminal restore, run_app, restore_terminal on exit
- Binary builds successfully at `target/debug/rsight`

## Task Commits

Each task was committed atomically:

1. **Task 1: Create src/event_loop.rs with the ratatui event loop** - `2cf246c` (feat)
2. **Task 2: Replace src/main.rs with the TUI entry point** - `c0413d4` (feat)

## Files Created/Modified
- `src/event_loop.rs` - init_terminal, restore_terminal, run_app, handle_key, trigger_search, cycle_tab
- `src/main.rs` - tokio::main entry point with panic hook, AppState init, run_app call, terminal restore
- `src/lib.rs` - added `pub mod event_loop`

## Decisions Made
- Used `crate::search::debounced_search` instead of `rsight::` since event_loop.rs is inside the crate, not an external consumer
- Named the direction enum `NavDir` to avoid name collision with `ratatui::layout::Direction`
- Filtered `KeyEventKind::Press` to avoid double-processing key events on platforms that emit press+release pairs

## Deviations from Plan

None - plan executed exactly as written.

Note: The `src/ui/` module (mod.rs, layout.rs, render.rs) was already present from Plan 02 execution. No additional work needed to satisfy the `use crate::ui::draw_ui` import in event_loop.rs.

## Issues Encountered
None - cargo check and cargo build passed on first attempt.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- TUI binary launches and runs the full event loop
- Keyboard input, search wiring, and rendering all complete
- Plan 04 can now add Enter (open file) and Ctrl+C (copy path) actions on top of this event loop

## Self-Check: PASSED

- src/event_loop.rs: FOUND
- src/main.rs: FOUND
- Commit 2cf246c: FOUND
- Commit c0413d4: FOUND

---
*Phase: 02-tui-shell*
*Completed: 2026-03-18*
