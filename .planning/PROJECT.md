# rsight

## What This Is

rsight is a TUI search engine for developers that searches the entire home directory — files, folders, file contents, and AI agent conversations — from a single terminal interface. It presents results in Google-style tabs (Files, Folders, Contents, AI Conversations) and opens results directly in the appropriate application: files in `$EDITOR`, folders in the system file manager, and AI conversations via their native resume commands.

## Core Value

Find anything in your home directory — including AI conversation history — in under a second, without leaving the terminal.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Search the entire $HOME directory for files, folders, and file contents
- [ ] Search and surface AI conversations from Claude Code (~/.claude/) and Cursor (~/.cursor/)
- [ ] TUI with search bar and tabbed results (Files, Folders, Contents, AI Conversations)
- [ ] Results appear in < 1 second
- [ ] Open files in $EDITOR, folders in system file manager, AI convos via native resume (claude --resume, cursor)
- [ ] Low memory footprint — no heavyweight persistent daemon required

### Out of Scope

- OAuth/external search APIs — fully local, offline-only
- Full-text indexing with persistent daemon — scan-on-demand preferred for memory
- Support for ChatGPT exports or other AI formats in v1 — Claude Code + Cursor only
- Mobile or GUI app — TUI only

## Context

- The tool is named `rsight`
- Language: Rust or Go (decided during planning based on TUI ecosystem maturity)
- AI conversation sources: Claude Code stores sessions as JSONL in `~/.claude/projects/`; Cursor stores conversations in `~/.cursor/` (format TBD during research)
- Open behavior per result type:
  - Files → `$EDITOR <path>`
  - Folders → `xdg-open <path>` (or `open` on macOS)
  - AI Conversations → `claude --resume <id>` or Cursor equivalent
- Search scope: entire `$HOME`, recursively
- Performance target: results visible in < 1 second on a typical developer machine

## Constraints

- **Performance**: Results in < 1s — drives choice of language, indexing strategy, and result streaming
- **Memory**: No persistent background daemon; scan-on-demand or lightweight on-launch indexing
- **Platform**: macOS primary (user's environment), Linux compatibility desirable

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Rust or Go for implementation | Both have strong TUI ecosystems; final choice depends on research | — Pending |
| Scan-on-demand vs persistent index | User prefers low memory over instant speed; < 1s is acceptable | Scan-on-demand preferred |
| Claude Code + Cursor only (v1) | Most common AI tools for target user; keep scope tight for v1 | — Pending |

---
*Last updated: 2026-03-18 after initialization*
