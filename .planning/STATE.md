---
gsd_state_version: 1.0
milestone: v1.0
milestone_name: milestone
status: unknown
last_updated: "2026-03-18T19:41:50.424Z"
progress:
  total_phases: 2
  completed_phases: 2
  total_plans: 10
  completed_plans: 10
---

# Project State

## Project Reference

See: .planning/PROJECT.md (updated 2026-03-18)

**Core value:** Find anything in your home directory — including AI conversation history — in under a second, without leaving the terminal.
**Current focus:** Phase 2 — TUI Shell

## Current Position

Phase: 2 of 3 (TUI Shell)
Plan: 5 of 5 in current phase — Plan 05 complete (with gaps)
Status: Phase 2 plans complete — gap closure required before Phase 3
Last activity: 2026-03-18 — Completed 02-tui-shell Plan 05 (Visual verification — 2 gaps found)

Progress: [██████████] 100% (Phase 1) | [██████████] 100% plans done (Phase 2, gap closure pending)

## Performance Metrics

**Velocity:**
- Total plans completed: 1
- Average duration: 2 min
- Total execution time: 0.03 hours

**By Phase:**

| Phase | Plans | Total | Avg/Plan |
|-------|-------|-------|----------|
| 01-search-core | 4 / 5 | 8 min | 2 min |

**Recent Trend:**
- Last 5 plans: 01-01 (2 min), 01-02 (2 min), 01-03 (2 min), 01-04 (2 min)
- Trend: stable

*Updated after each plan completion*
| Phase 01-search-core P03 | 2min | 1 tasks | 2 files |
| Phase 01-search-core P04 | 2min | 2 tasks | 3 files |
| Phase 01-search-core P05 | 10min | 2 tasks | 2 files |
| Phase 02-tui-shell P01 | 2min | 2 tasks | 3 files |
| Phase 02-tui-shell P02 | 2min | 2 tasks | 4 files |
| Phase 02-tui-shell P03 | 2min | 2 tasks | 3 files |
| Phase 02-tui-shell P04 | 2min | 2 tasks | 6 files |
| Phase 02-tui-shell P05 | 10min | 1 task (checkpoint) | 0 files |

## Accumulated Context

### Decisions

Decisions are logged in PROJECT.md Key Decisions table.
Recent decisions affecting current work:

- **Rust chosen** — raw performance and minimal memory overhead (confirmed in 01-01)
- **ignore crate** — ripgrep's parallel traversal engine, respects .gitignore (confirmed in 01-01)
- **mpsc channel streaming API** — search() returns Receiver<SearchResult>, callers consume async (established in 01-01)
- **SearchResult with owned Strings** — no lifetimes for safe channel transport (established in 01-01)
- Scan-on-demand preferred over persistent daemon for memory reasons
- [Phase 01-search-core]: Two-pass collect-then-parallel strategy chosen for search_contents to avoid WalkBuilder lifetime issues with rayon
- [Phase 01-search-core]: blocking_send used from rayon threads for mpsc streaming since rayon threads are not inside tokio runtime
- [01-02]: hidden(false) and git_ignore(false) on WalkBuilder to traverse all files including dotfiles (SRCH-04)
- [01-02]: filter_entry prunes entire subtrees at excluded directories for efficiency
- [01-02]: search_names takes mpsc::Sender (not returning Receiver) — caller controls channel lifecycle; Plan 04 must spawn in tokio::task::spawn_blocking
- [01-04]: spawn_blocking used for both name and content search since both are synchronous; tx clone pattern drops original tx to close channel when both tasks finish
- [01-04]: debounced_search accepts Sender (not returning Receiver) so TUI controls the result channel lifecycle; delay_ms is a parameter for testability
- [Phase 01-search-core]: criterion async_tokio feature added but bench loops call synchronous functions directly — avoids runtime overhead
- [Phase 01-search-core]: < 1s requirement scoped to TUI first-result streaming latency, not total CLI wall time
- [Phase 02-tui-shell]: crate-local imports (crate::) used in app.rs instead of rsight:: — app.rs is inside the crate not a dependent
- [Phase 02-tui-shell]: AppState owns both ends of the mpsc channel (4096 cap) — no external channel management by callers
- [Phase 02-tui-shell]: draw_ui is the sole rendering entry point; format_result produces plain String — styled spans with match highlights deferred to Plan 04
- [Phase 02-tui-shell]: ListState constructed locally each frame from app.selected_index — no persistent widget state
- [Phase 02-tui-shell]: crate::search::debounced_search used in event_loop.rs (inside crate, not rsight:: external consumer)
- [Phase 02-tui-shell]: NavDir enum avoids collision with ratatui::layout::Direction; KeyEventKind::Press filter prevents duplicate key events
- [Phase 02-tui-shell]: highlight_spans uses case-insensitive greedy substring match (first occurrence) — lightweight, no fuzzy overhead
- [Phase 02-tui-shell]: make_list_item returns ListItem<'static> via owned Strings to avoid lifetime propagation into ratatui widgets
- [Phase 02-tui-shell]: clone result before open_result to resolve immutable/mutable borrow conflict on AppState
- [02-tui-shell-05 gap]: TUI-02 partial — content search takes >30s across $HOME; file/folder name search is fast; gap closure needed
- [02-tui-shell-05 gap]: Tab reset-on-query-change decision REVERSED by user testing — active tab must persist on query change
- [02-gap closure]: EXCLUDED_DIRS expanded to include .cargo, .rustup, Library, Applications, .Trash, Movies, Music, Pictures — eliminates the dominant $HOME cost centres for content search
- [02-gap closure]: WalkBuilder switched from build() to build_parallel() — directory traversal is now multi-threaded, matching rayon parallelism already used for file reads
- [02-gap closure]: MAX_CONTENT_RESULTS=100 cap enforced atomically in search_contents — consistent with AppState 100-result-per-tab cap; prevents unbounded streaming on large trees
- [02-gap closure]: active_tab is no longer reset on query-change in event_loop.rs; tab transitions are now exclusively explicit (Tab/Shift+Tab)

### Pending Todos

None yet.

### Blockers/Concerns

- Cursor conversation format (~/.cursor/) is TBD — needs investigation before Phase 3
- ~~**[GAP-01] Content search performance**~~: CLOSED — exclusion list expanded, build_parallel() adopted, 100-result cap added (commit 1cc9ec9)
- ~~**[GAP-02] Tab persistence regression**~~: CLOSED — active_tab reset removed from query-change handler (commit 7f60d7d)

## Session Continuity

Last session: 2026-03-18
Stopped at: Completed 02-gap closure — GAP-01 (content search perf) and GAP-02 (tab persistence) both fixed and committed
Resume file: None
