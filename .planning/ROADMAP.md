# Roadmap: rsight

## Overview

rsight ships in three phases. Phase 1 builds the search engine core — file/folder/content scanning of $HOME with sub-second results. Phase 2 wraps that core in a TUI with a search bar, tabbed results, keyboard navigation, and match highlighting. Phase 3 adds AI conversation search (Claude Code, Cursor) and wires all result types to their native open actions.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Search Core** - Scan $HOME for files, folders, and file contents with results under 1 second (completed 2026-03-18)
- [ ] **Phase 2: TUI Shell** - Interactive terminal interface with search bar, tabbed results, and keyboard navigation
- [ ] **Phase 3: AI Conversations + Open Actions** - AI conversation search and result-opening behaviors for all result types

## Phase Details

### Phase 1: Search Core
**Goal**: Users can search their entire $HOME and get file, folder, and content results in under 1 second
**Depends on**: Nothing (first phase)
**Requirements**: SRCH-01, SRCH-02, SRCH-03, SRCH-04
**Success Criteria** (what must be TRUE):
  1. User can search file and folder names using fuzzy matching and see matching results
  2. User can search file contents and see lines that contain the query
  3. Results for a typical $HOME directory appear in under 1 second
  4. Search recurses into the entire $HOME directory, including hidden directories
**Plans**: TBD

### Phase 2: TUI Shell
**Goal**: Users interact with search through a live terminal UI with a search bar, tabbed result panes, and keyboard controls
**Depends on**: Phase 1
**Requirements**: TUI-01, TUI-02, TUI-03, TUI-04
**Success Criteria** (what must be TRUE):
  1. Launching rsight shows a search bar that accepts typing immediately
  2. Results appear under four tabs (Files, Folders, Contents, AI Conversations) and update as the user types
  3. User can move between results with arrow keys, switch tabs, confirm with Enter, and quit with Esc
  4. Query matches are visually highlighted in filenames, content snippets, and conversation excerpts
**Plans**: 5 plans

Plans:
- [ ] 02-tui-shell-01-PLAN.md — ratatui/crossterm dependencies + AppState and TabKind types
- [ ] 02-tui-shell-02-PLAN.md — TUI layout and rendering (search bar, tab bar, results list, status bar)
- [ ] 02-tui-shell-03-PLAN.md — Event loop and search wiring (keyboard input, debounced_search, result drain)
- [ ] 02-tui-shell-04-PLAN.md — Match highlighting and open/copy actions (Enter, Ctrl+C)
- [ ] 02-tui-shell-05-PLAN.md — Visual verification checkpoint

### Phase 3: AI Conversations + Open Actions
**Goal**: Users can search Claude Code and Cursor conversation history and open any result type in its native application
**Depends on**: Phase 2
**Requirements**: AICV-01, AICV-02, AICV-03, AICV-04, OPEN-01, OPEN-02
**Success Criteria** (what must be TRUE):
  1. User can search Claude Code conversations stored in ~/.claude/projects/ and see matching excerpts
  2. User can search Cursor conversations stored in ~/.cursor/ and see matching excerpts
  3. Each AI conversation result displays a title or first-message summary
  4. Selecting an AI conversation resumes it via claude --resume <id> or the Cursor equivalent
  5. Selecting a file result opens it in $EDITOR; selecting a folder opens it in the system file manager
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Search Core | 5/5 | Complete   | 2026-03-18 |
| 2. TUI Shell | 0/5 | Not started | - |
| 3. AI Conversations + Open Actions | 0/TBD | Not started | - |
