---
phase: 03-ai-conversations-open-actions
plan: "03"
subsystem: ui
tags: [rust, ratatui, tui, osascript, terminal, cursor, claude-code]

# Dependency graph
requires:
  - phase: 03-ai-conversations-open-actions plan 01
    provides: AiConversation SearchResult variant with conversation_id, title, date, source, path
  - phase: 03-ai-conversations-open-actions plan 02
    provides: AI Conversations tab enabled, search pipeline wired
provides:
  - AiConversation rendered as "title · date [Source]" with yellow match highlighting
  - open_conversation() action for Claude Code (Terminal.app osascript) and Cursor (cursor CLI)
  - Enter handler dispatches AiConversation to open_conversation, other variants to open_result
affects:
  - 03-04

# Tech tracking
tech-stack:
  added: []
  patterns:
    - osascript used to open new Terminal.app window (non-blocking spawn)
    - which-check pattern for CLI detection before invoking external tools
    - Variant dispatch in Enter handler via match on SearchResult enum

key-files:
  created: []
  modified:
    - src/ui/render.rs
    - src/actions.rs
    - src/event_loop.rs

key-decisions:
  - "osascript 'tell application Terminal to do script' used to open new Terminal window for Claude Code — rsight stays alive"
  - "Cursor CLI checked via 'which cursor'; falls back to 'open -a Cursor' if CLI absent"
  - "Enter handler dispatches by SearchResult variant, not by tab — avoids tab-identity coupling"
  - "Unused Alignment import removed after dead disabled-tab branch deleted (Rule 1 auto-fix)"

patterns-established:
  - "open_conversation takes &mut Option<String> status_message for in-place error display"
  - "Spawn (non-blocking) used for all external process launches — rsight never waits"

requirements-completed: [AICV-03, AICV-04, OPEN-01, OPEN-02]

# Metrics
duration: 2min
completed: 2026-03-18
---

# Phase 3 Plan 03: AI Conversations Rendering and Open Actions Summary

**AiConversation TUI rendering with title · date [Source] format plus osascript Terminal launch for Claude Code and cursor CLI launch for Cursor, wired to Enter key with graceful CLI-not-found status messages**

## Performance

- **Duration:** 2 min
- **Started:** 2026-03-18T21:11:59Z
- **Completed:** 2026-03-18T21:13:41Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- AiConversation results render as highlighted title + dim " · " + dim date + dim [Claude]/[Cursor] badge
- open_conversation() in actions.rs handles both Claude Code (osascript Terminal.app) and Cursor (cursor CLI or open -a Cursor fallback)
- Enter handler in event_loop.rs dispatches AiConversation to open_conversation, all other variants to open_result (no regression)
- Dead disabled-tab branch removed from draw_results() — all tabs now handled uniformly

## Task Commits

Each task was committed atomically:

1. **Task 1: Render AI conversations in TUI (title · date format + highlighting)** - `50b5e13` (feat)
2. **Task 2: Implement open_conversation() action + wire Enter handler** - `27a4814` (feat)

**Plan metadata:** (docs commit follows)

## Files Created/Modified
- `src/ui/render.rs` - AiConversation arm with highlight_spans on title, dim separator/date/badge; removed dead disabled-tab branch and unused Alignment import
- `src/actions.rs` - open_conversation() with which-based CLI detection, osascript for Claude, cursor CLI + open -a Cursor fallback; AiConversation arm in open_result() left as no-op
- `src/event_loop.rs` - Enter handler dispatches by SearchResult variant; open_conversation imported; SearchResult imported for match

## Decisions Made
- osascript `tell application "Terminal" to do script` chosen for Claude Code — opens a new Terminal window, rsight remains running
- Cursor CLI `which cursor` check with `open -a Cursor` fallback — handles both CLI-installed and app-only users
- Enter handler dispatches by variant (not by tab) for cleaner separation of concerns

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed unused `Alignment` import after deleting disabled-tab branch**
- **Found during:** Task 1 (render.rs changes)
- **Issue:** Removing the disabled-tab branch that used `Alignment::Center` left the import unused; Rust emits a warning
- **Fix:** Removed `layout::Alignment` from the ratatui imports
- **Files modified:** src/ui/render.rs
- **Verification:** cargo build produced zero warnings after fix
- **Committed in:** 50b5e13 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (Rule 1 - unused import cleanup)
**Impact on plan:** Minor cleanup required for dead code from prior plan; no scope creep.

## Issues Encountered
None - build was clean after each task.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All three sources (File, Folder/ContentMatch, AiConversation) have wired Enter handlers
- Ctrl+C path copy works for AiConversation via existing result_path() which already handled the variant
- Ready for Phase 3 Plan 04 (final polish / packaging)

---
*Phase: 03-ai-conversations-open-actions*
*Completed: 2026-03-18*
