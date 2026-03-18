# Phase 1: Search Core - Context

**Gathered:** 2026-03-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Build the search engine that scans the entire `$HOME` directory and returns file names, folder names, and file contents matching a query in under 1 second. This phase delivers the search library/module — no TUI yet, just the search logic and data structures that Phase 2 will consume.

</domain>

<decisions>
## Implementation Decisions

### Language
- **Rust** — chosen for raw performance and minimal memory overhead
- Use the `ignore` crate (the library behind ripgrep) for file traversal — battle-tested, parallel with rayon, respects .gitignore by default

### Search Architecture
- **Stream results as found** — start traversal on query, push matches into the TUI as they arrive; user sees results immediately, list grows in real time
- No startup index — scan on demand to keep memory footprint low
- **Debounced re-run**: wait ~150ms after last keypress, cancel previous in-flight search, start fresh traversal — ensures correct results every time
- **Include hidden directories** — `~/.claude`, `~/.cursor`, `~/.config` etc. must be searched; configure the `ignore` crate to traverse dotfiles

### Content Search Scope
- Search **text files only** — use ripgrep's binary-detection heuristic (check for null bytes in first N bytes) to auto-skip binaries
- **1 MB per file limit** — skip files larger than 1MB during content search
- **Always-excluded directories**: `node_modules/`, `.git/`, `target/`, `vendor/`, `build/`

### Claude's Discretion
- Exact debounce implementation (tokio timer vs channel-based cancellation)
- Thread pool sizing for parallel traversal
- How to represent a search result internally (struct fields, lifetime vs owned strings)
- Whether to use `grep` crate for content matching or implement manually

</decisions>

<specifics>
## Specific Ideas

- The `ignore` crate is the right foundation: it handles gitignore rules, hidden file traversal toggle, and parallel walking — no need to reinvent
- ripgrep's approach of checking null bytes for binary detection is proven; reuse that heuristic
- Performance target: < 1s on a typical developer $HOME (tens of thousands of files)

</specifics>

<code_context>
## Existing Code Insights

### Reusable Assets
- None yet — greenfield project

### Established Patterns
- None yet — first phase sets the patterns

### Integration Points
- This phase produces a search API (function or struct) that Phase 2 (TUI Shell) will call to get a stream of results
- Result types must support all four categories: File, Folder, ContentMatch, AiConversation (Phase 3 adds the last)

</code_context>

<deferred>
## Deferred Ideas

- None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-search-core*
*Context gathered: 2026-03-18*
