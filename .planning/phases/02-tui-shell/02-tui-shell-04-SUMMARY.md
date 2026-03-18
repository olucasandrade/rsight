---
phase: 02-tui-shell
plan: 04
subsystem: ui
tags: [ratatui, crossterm, highlight, spans, clipboard, pbcopy, open]

# Dependency graph
requires:
  - phase: 02-tui-shell
    provides: render.rs draw_results with plain string items, event_loop.rs handle_key with Enter/Ctrl+C stubs

provides:
  - highlight_spans(text, query) -> Vec<Span> with yellow+bold match highlighting
  - make_list_item using styled Line<Span> for result rows (filename highlighted, path dimmed)
  - open_result() using macOS `open` or $EDITOR at line number for ContentMatch
  - copy_to_clipboard() piping to pbcopy
  - result_path() extracting path from any SearchResult variant
  - Enter and Ctrl+C fully wired in event_loop handle_key

affects: [02-tui-shell-05, 03-ai-conversations]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - highlight_spans returns Vec<Span<'static>> via owned Strings — no lifetime issues
    - make_list_item produces ListItem<'static> for ratatui compatibility
    - clone result before open_result to avoid borrow conflict with status_message mutation

key-files:
  created:
    - src/ui/highlight.rs
    - src/actions.rs
  modified:
    - src/ui/mod.rs
    - src/ui/render.rs
    - src/event_loop.rs
    - src/lib.rs

key-decisions:
  - "highlight_spans uses case-insensitive greedy substring match (first occurrence) — covers common case with no fuzzy overhead"
  - "open_result checks $EDITOR env var for ContentMatch; falls back to `open` if unset — plan decision"
  - "copy_to_clipboard uses pbcopy via stdin pipe on macOS"
  - "clone result before calling open_result to avoid immutable/mutable borrow conflict on AppState"

patterns-established:
  - "highlight_spans: owned String in, Vec<Span<'static>> out — avoids lifetime propagation into ListItem"
  - "actions.rs: pure functions operating on SearchResult — no AppState dependency, easily testable"

requirements-completed: [TUI-03, TUI-04]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 2 Plan 04: Match Highlighting and Open Actions Summary

**Yellow+bold match highlighting via ratatui Span, plus Enter (open with system default or $EDITOR) and Ctrl+C (pbcopy) actions fully wired in the TUI event loop**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-18T19:25:47Z
- **Completed:** 2026-03-18T19:27:29Z
- **Tasks:** 2
- **Files modified:** 6

## Accomplishments

- Created `src/ui/highlight.rs` with `highlight_spans(text, query)` returning Vec<Span> with yellow+bold for matched substring
- Updated `render.rs` with `make_list_item` producing styled `Line<Span>`: filename highlighted, path dimmed; ContentMatch prefix dimmed, snippet highlighted
- Created `src/actions.rs` with `open_result`, `copy_to_clipboard`, `result_path` pure functions
- Wired Enter (open) and Ctrl+C (copy path to clipboard) in `event_loop.rs`; Enter on disabled AI tab sets "AI search coming in a future update"

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement match highlighting spans** - `f9fa146` (feat)
2. **Task 2: Implement open and copy actions (Enter and Ctrl+C)** - `3621e9e` (feat)

**Plan metadata:** (docs commit pending)

## Files Created/Modified

- `src/ui/highlight.rs` - highlight_spans(text, query) -> Vec<Span>, case-insensitive greedy match
- `src/ui/mod.rs` - added `pub mod highlight` and `pub use highlight::highlight_spans`
- `src/ui/render.rs` - replaced format_result with make_list_item using styled Line/Span, added truncate_path
- `src/actions.rs` - open_result (macOS `open` / $EDITOR), copy_to_clipboard (pbcopy), result_path
- `src/event_loop.rs` - added Enter and Ctrl+C branches in handle_key, import actions
- `src/lib.rs` - added `pub mod actions`

## Decisions Made

- `highlight_spans` uses case-insensitive greedy substring find (first occurrence) — lightweight approach adequate for this use case
- `make_list_item` returns `ListItem<'static>` by using owned Strings throughout — avoids lifetime issues
- `result` is cloned before calling `open_result` to avoid immutable/mutable borrow conflict when subsequently mutating `app.status_message`

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- All Plan 04 must-haves satisfied: highlight_spans exists, result rows show yellow+bold matches, Enter/Ctrl+C actions are wired, AI tab Enter shows "coming soon" message, cargo build passes
- Ready for Plan 05 (final TUI shell plan)

## Self-Check: PASSED

- src/ui/highlight.rs: FOUND
- src/actions.rs: FOUND
- 02-tui-shell-04-SUMMARY.md: FOUND
- Commit f9fa146: FOUND
- Commit 3621e9e: FOUND

---
*Phase: 02-tui-shell*
*Completed: 2026-03-18*
