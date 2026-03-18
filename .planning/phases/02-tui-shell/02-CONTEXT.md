# Phase 2: TUI Shell - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Build a live terminal UI for rsight: a search bar, four tabbed result panes (Files, Folders, Contents, AI Conversations), keyboard navigation, and match highlighting. The UI calls the existing `search()` / `debounced_search()` API from Phase 1. AI search backend is Phase 3 — the AI tab is visible but disabled in this phase.

</domain>

<decisions>
## Implementation Decisions

### Tab structure
- 4 tabs: Files, Folders, Contents, AI Conversations
- AI Conversations tab is visually present but disabled/grayed out — activated in Phase 3
- Tab labels show names only, no live result count
- Tab/Shift+Tab switches between tabs
- Active tab always resets to Files when the user clears the query and retypes

### Confirm action (Enter)
- Enter opens the selected result with the system default (`open` on macOS)
- For ContentMatch results: if $EDITOR is set, open at the matching line number (e.g. `$EDITOR +LINE_NUMBER path`); fall back to system default if $EDITOR is not set
- Ctrl+C copies the full path to clipboard (secondary action)
- Pressing Enter on the disabled AI Conversations tab shows a brief status bar message: "AI search coming in a future update"

### Layout & chrome
- Full terminal takeover on launch (like vim/htop) — rsight owns the entire terminal
- Layout top-to-bottom: search bar → tabs → results list → status bar
- Status bar shows result count and key hints: e.g. "42 results  ↑↓ navigate  Tab switch tab  Enter open  Ctrl+C copy  Esc quit"
- Minimal visual style: mostly text, subtle borders, no heavy colors except match highlights

### Result display
- File/Folder results: filename displayed bold + full path dimmed, on one line
  - e.g. `README.md  ~/Documents/projects/rsight/README.md`
- ContentMatch results: `path:line_number  snippet` format
  - e.g. `src/lib.rs:8    pub use types::SearchResult;`
- Snippet length: truncated to fit terminal width (no wrapping)
- Cap at 100 results per tab — results sorted by relevance (fuzzy score for Files/Folders, first-found for Contents)

### Claude's Discretion
- Match highlighting implementation (how matched characters are bolded/colored in filenames and snippets)
- Empty state display when no results exist for a query
- Loading/streaming indicator while results are still arriving
- Exact border and color scheme within the minimal style constraint
- How to detect and use $EDITOR for line-number jumps
- Ratatui widget structure and state management internals

</decisions>

<specifics>
## Specific Ideas

- Layout should feel like Raycast or fzf — search bar always at top, immediate responsiveness
- Content match format mirrors ripgrep output (`path:line  snippet`) — familiar to developers
- The selected result row should use a clear highlight (e.g. reverse video or a distinct background)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `rsight::search(root, query) -> mpsc::Receiver<SearchResult>` — the streaming search API; TUI calls this and drains the receiver into tab-specific result lists
- `rsight::debounced_search(root, query, tx, delay_ms) -> SearchHandle` — the TUI's keypress handler should use this with 150ms delay; abort the previous handle on each keystroke
- `SearchResult` enum with `File { path, name, score }`, `Folder { path, name, score }`, `ContentMatch { path, line_number, line }` — maps directly to the 3 active tab types

### Established Patterns
- All search is async (tokio runtime) — TUI needs a tokio runtime; ratatui works with tokio via `EventStream` or a background thread for terminal events
- `spawn_blocking` used for CPU-bound search work — TUI rendering stays on async task; search results arrive via mpsc channel

### Integration Points
- TUI is the new binary entry point in `src/main.rs` — replace the current CLI smoke-test main with the ratatui event loop
- Results flow: keypress → debounced_search → mpsc::Receiver → drain into Vec<SearchResult> per tab → re-render

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-tui-shell*
*Context gathered: 2026-03-18*
