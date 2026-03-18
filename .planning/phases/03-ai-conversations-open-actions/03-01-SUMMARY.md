---
phase: 03-ai-conversations-open-actions
plan: 01
subsystem: ai-search
tags: [rust, serde_json, rusqlite, sqlite, jsonl, mpsc, streaming]

# Dependency graph
requires:
  - phase: 01-search-core
    provides: SearchResult enum and mpsc channel streaming pattern
  - phase: 02-tui-shell
    provides: AppState.push_result dispatch and ui/render.rs make_list_item
provides:
  - AiSource enum (ClaudeCode, Cursor) in src/types.rs
  - AiConversation variant in SearchResult with path, conversation_id, title, date, source fields
  - search_claude_conversations — walks ~/.claude/projects/**/*.jsonl, streams matching results
  - search_cursor_conversations — walks ~/.cursor/chats/**/store.db via SQLite, streams matching results
  - src/ai_search module exported from src/lib.rs
affects: [03-02, 03-03, 03-04, phase-04]

# Tech tracking
tech-stack:
  added: [serde_json = "1", rusqlite = "0.31" (bundled)]
  patterns:
    - "search_*_conversations(query, tx: mpsc::Sender<SearchResult>) — blocking sync pattern matching content_search.rs"
    - "Silently skip missing dirs and malformed files — return early without panic"
    - "Stop sending on Err from tx.send (channel closed) — consistent with existing parsers"
    - "Manual ISO 8601 date parsing (no chrono) — month number to abbreviation via match"
    - "Gregorian calendar algorithm for Unix ms timestamp to Mon DD (no chrono)"
    - "Bundled rusqlite to avoid system SQLite dependency"
    - "Read-only SQLite open flags to avoid WAL lock conflicts"

key-files:
  created:
    - src/ai_search/mod.rs
    - src/ai_search/claude.rs
    - src/ai_search/cursor.rs
  modified:
    - src/types.rs
    - src/lib.rs
    - src/app.rs
    - src/actions.rs
    - src/ui/render.rs
    - Cargo.toml

key-decisions:
  - "serde_json added as dependency — was absent from Cargo.toml despite plan assuming it existed"
  - "AiConversation match arms added to app.rs (drops to void), actions.rs (open path), ui/render.rs (minimal [AI] label) to satisfy exhaustive pattern checks"
  - "rusqlite bundled feature used — avoids requiring system SQLite, ensures consistent behavior across machines"
  - "Hex-decode via manual step_by(2) iteration — no hex crate dependency needed"
  - "tx.send() used instead of tx.blocking_send() — both parsers run synchronously without tokio context; std::sync::mpsc::Sender::send() is the correct API"

patterns-established:
  - "ai_search parsers use std::sync::mpsc not tokio::sync::mpsc — consistent with name_search.rs pattern"
  - "New SearchResult variant requires exhaustive match updates across app.rs, actions.rs, ui/render.rs"

requirements-completed: [AICV-01, AICV-02, AICV-03]

# Metrics
duration: 5min
completed: 2026-03-18
---

# Phase 3 Plan 01: AI Conversations Data Layer Summary

**AiConversation type + Claude Code JSONL parser + Cursor SQLite parser streamed via std::sync::mpsc, backed by serde_json and bundled rusqlite**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-18T20:00:45Z
- **Completed:** 2026-03-18T20:05:02Z
- **Tasks:** 3
- **Files modified:** 9

## Accomplishments

- `AiSource` enum and `AiConversation` variant added to `SearchResult` with all required fields
- Claude Code parser walks `~/.claude/projects/**/*.jsonl`, extracts title from first real user message, skips command-prefixed content, formats dates manually from ISO 8601
- Cursor parser walks `~/.cursor/chats/**/store.db` read-only via rusqlite, hex-decodes meta JSON, searches title and blob data, converts Unix ms timestamps via Gregorian math

## Task Commits

Each task was committed atomically:

1. **Task 1: Add AiConversation variant and AiSource enum** - `5c176a1` (feat)
2. **Task 2: Implement Claude Code parser** - `0dcb52a` (feat)
3. **Task 3: Implement Cursor parser + rusqlite** - `3164790` (feat)

## Files Created/Modified

- `src/types.rs` - Added AiSource enum and AiConversation variant to SearchResult
- `src/ai_search/mod.rs` - Module declaration; re-exports both search functions
- `src/ai_search/claude.rs` - Claude Code JSONL walker and parser
- `src/ai_search/cursor.rs` - Cursor SQLite walker and parser with hex-decode and date math
- `src/lib.rs` - Registered ai_search module
- `src/app.rs` - Added AiConversation arm to push_result match (drops silently)
- `src/actions.rs` - Added AiConversation arms to open_result and result_path
- `src/ui/render.rs` - Added AiConversation arm to make_list_item (minimal [AI] label)
- `Cargo.toml` - Added serde_json = "1" and rusqlite = "0.31" (bundled)

## Decisions Made

- **serde_json not pre-existing:** Plan stated it was "already in Cargo.toml from content search" but it was absent. Added as dependency (Rule 3 auto-fix).
- **rusqlite bundled feature:** Ensures consistent SQLite availability without system dependency.
- **std::sync::mpsc::Sender::send():** Both parsers are synchronous and not inside a tokio runtime, so std::sync::mpsc is the correct channel type (matching name_search.rs pattern, not content_search.rs which uses tokio::sync::mpsc).
- **Placeholder match arms in app/actions/render:** AiConversation must be handled in existing match statements; deferred full routing/rendering to later plans per phase design.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added missing serde_json dependency to Cargo.toml**
- **Found during:** Task 2 (Claude Code parser)
- **Issue:** Plan assumed serde_json was already in Cargo.toml, but it was absent — import failed to compile
- **Fix:** Added `serde_json = "1"` to Cargo.toml [dependencies]
- **Files modified:** Cargo.toml
- **Verification:** cargo build passed
- **Committed in:** 0dcb52a (Task 2 commit)

**2. [Rule 1 - Bug] Added exhaustive match arms for AiConversation in existing files**
- **Found during:** Task 1 (types.rs extension)
- **Issue:** Adding AiConversation to SearchResult enum caused non-exhaustive pattern errors in app.rs, actions.rs, and ui/render.rs
- **Fix:** Added AiConversation arms to push_result (silent drop), open_result (open path), result_path (return path), and make_list_item (minimal label)
- **Files modified:** src/app.rs, src/actions.rs, src/ui/render.rs
- **Verification:** cargo build passed
- **Committed in:** 5c176a1 (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (1 missing dependency, 1 exhaustive pattern)
**Impact on plan:** Both auto-fixes required for compilation. No scope creep.

## Issues Encountered

None beyond the two auto-fixed deviations above.

## Next Phase Readiness

- AiConversation data type and both parsers are complete and compile cleanly
- `search_claude_conversations` and `search_cursor_conversations` are exported from `src/ai_search/mod.rs`
- Phase 3 Plan 02 can wire these parsers into the TUI search pipeline and add the AI Conversations tab

---
*Phase: 03-ai-conversations-open-actions*
*Completed: 2026-03-18*
