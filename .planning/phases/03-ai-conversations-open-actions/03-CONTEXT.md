# Phase 3: AI Conversations + Open Actions - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Search Claude Code conversation history (`~/.claude/projects/`) and Cursor conversation history (`~/.cursor/`), surface results in the existing AI Conversations tab, and let the user resume conversations or open files/folders in native apps. The tab is already defined in AppState — this phase activates it.

</domain>

<decisions>
## Implementation Decisions

### Conversation display
- Format: `Title · Date` on a single line (e.g. `Debug WalkBuilder dotfile traversal  · Mar 15`)
- Title derived from: first human message in the conversation, truncated to ~60 chars
- If first human message is unavailable (malformed/empty): fall back to conversation ID or filename
- Query matches highlighted in yellow/bold within the title text (consistent with file/content highlighting)
- Date shown as abbreviated format (e.g. `Mar 15`, `Mar 17`) — not full ISO timestamp

### Resume behavior (Enter on AI conversation result)
- **Claude Code conversations:** Open a **new Terminal.app window** running `claude --resume <conversation_id>`. rsight stays open in the original terminal.
  - Use `open -a Terminal` with the command, or `osascript` to launch a new Terminal window
  - If `claude` CLI is not found: show error in status bar — "claude CLI not found — install at claude.ai/code"
- **Cursor conversations:** Open the associated workspace/project folder in Cursor via `cursor <workspace_path>`. rsight stays open.
  - If `cursor` CLI is not found: show error in status bar
- rsight does NOT exit when a conversation is resumed (new window approach keeps rsight alive)

### Search scope & matching
- Search full message text — both human and assistant messages in each conversation
- Search all conversations (no time limit) — entire `~/.claude/projects/` and `~/.cursor/` history
- Unified query: same search string triggers AI conversation search alongside file/content search
- AI Conversations tab results stream in alongside other tabs when the user types
- If `~/.cursor/` does not exist: silently skip Cursor search, no error shown
- Cap at 100 results per tab (consistent with other tabs)

### Open actions for files & folders (Claude's discretion — not discussed)
- Current behavior (`open` for files/folders, `$EDITOR +line` for content matches) is already in `actions.rs`
- Folder: use `open` (opens in Finder) — success criteria says "system file manager", `open` satisfies this on macOS
- No change needed to existing open_result logic

### Claude's Discretion
- How to parse `~/.claude/projects/` file format (JSON structure, conversation ID extraction)
- How to parse `~/.cursor/` storage format (may differ significantly from Claude Code)
- Whether to add `AiConversation` as a new `SearchResult` variant or use a parallel struct
- How to launch a new Terminal.app window with a command on macOS (osascript vs open -a Terminal)
- Exact date formatting logic
- How to run AI conversation search concurrently with name/content search (extend `search()` or separate channel)

</decisions>

<specifics>
## Specific Ideas

- The AI Conversations tab was already wired as `TabKind::AiConversations` in Phase 2 — it just needs the search backend and display rendering activated
- The "title · date" display mirrors common app patterns (like iMessage, Slack history) — clean and scannable
- Opening a new Terminal.app window (not current terminal handoff) is preferred so rsight stays available for further searches

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SearchResult` enum in `src/types.rs` — needs a new `AiConversation` variant (or separate type) with fields: `path` (conversation file), `conversation_id` (for resume), `title` (derived first message), `date` (file modification time or parsed from content), `source` (ClaudeCode | Cursor)
- `TabKind::AiConversations` already defined in `src/app.rs` — `AppState` has fields for the tab; needs `ai_conversations: Vec<AiConversation>` results field added
- `content_search.rs` pattern (WalkBuilder + rayon) — can be adapted for walking `~/.claude/projects/` and reading JSON conversation files
- `actions.rs` `open_result` — extend to handle `AiConversation` variant (new Terminal window for Claude, `cursor` for Cursor)
- `src/ui/render.rs` `draw_results` — extend to render `AiConversation` items with `title · date` format and match highlighting
- `src/search.rs` `search()` — extend to spawn a third concurrent task for AI conversation search alongside name/content tasks

### Established Patterns
- Results streamed via `mpsc::Sender<SearchResult>` into `AppState` per-tab — AI conversation search follows the same pattern
- 100-result cap enforced per tab — same for AI conversations
- Match highlighting via `highlight_spans()` in `src/ui/highlight.rs` — reuse for title highlighting
- Debounced search with `SearchHandle` abort — AI search participates in the same debounce cycle

### Integration Points
- `src/search.rs` `search()` function — spawn third `task::spawn_blocking` for AI conversation search
- `src/app.rs` `AppState` — add `ai_conversations: Vec<AiConversation>` field, update result routing in `drain_results()`
- `src/event_loop.rs` Enter handler — add `AiConversation` arm calling new `open_conversation()` in actions.rs
- `src/ui/render.rs` — add `AiConversation` arm to result rendering with `title · date` format

</code_context>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 03-ai-conversations-open-actions*
*Context gathered: 2026-03-18*
