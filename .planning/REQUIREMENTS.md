# Requirements: rsight

**Defined:** 2026-03-18
**Core Value:** Find anything in your home directory — including AI conversation history — in under a second, without leaving the terminal.

## v1 Requirements

### TUI

- [ ] **TUI-01**: User sees a search bar at the top of the screen that accepts input on launch
- [ ] **TUI-02**: Results are displayed in tabbed panes: Files, Folders, Contents, AI Conversations
- [ ] **TUI-03**: User can navigate results with arrow keys, open with Enter, and clear/quit with Esc
- [ ] **TUI-04**: Search query matches are highlighted in result text (filenames, content snippets, conversation excerpts)

### Search

- [ ] **SRCH-01**: User can search file and folder names using fuzzy matching
- [ ] **SRCH-02**: User can search file contents using exact full-text matching
- [ ] **SRCH-03**: Results appear in under 1 second on a typical developer machine
- [ ] **SRCH-04**: Search scope is the user's entire $HOME directory recursively

### AI Conversations

- [ ] **AICV-01**: User can search Claude Code conversation content (~/.claude/projects/)
- [ ] **AICV-02**: User can search Cursor conversation content (~/.cursor/)
- [ ] **AICV-03**: Each AI conversation result shows a title or first-message summary
- [ ] **AICV-04**: Selecting an AI conversation result resumes it via native command (claude --resume <id> or Cursor equivalent)

### Open Actions

- [ ] **OPEN-01**: Selecting a file result opens it in $EDITOR
- [ ] **OPEN-02**: Selecting a folder result opens it in the system file manager (xdg-open / open)

## v2 Requirements

### Search

- **SRCH-V2-01**: Regex pattern support in search bar
- **SRCH-V2-02**: Respect .gitignore / skip build artifacts automatically
- **SRCH-V2-03**: Result count displayed per tab

### AI Conversations

- **AICV-V2-01**: Filter AI Conversations tab by agent (Claude only / Cursor only)
- **AICV-V2-02**: Support additional AI conversation formats (ChatGPT exports, etc.)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Persistent background index daemon | Memory cost — scan-on-demand preferred for v1 |
| OAuth / external search APIs | Fully local, offline-only tool |
| GUI / web interface | TUI-only by design |
| Mobile support | Developer terminal tool, not applicable |
| Network/remote filesystem search | Local $HOME only |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| TUI-01 | — | Pending |
| TUI-02 | — | Pending |
| TUI-03 | — | Pending |
| TUI-04 | — | Pending |
| SRCH-01 | — | Pending |
| SRCH-02 | — | Pending |
| SRCH-03 | — | Pending |
| SRCH-04 | — | Pending |
| AICV-01 | — | Pending |
| AICV-02 | — | Pending |
| AICV-03 | — | Pending |
| AICV-04 | — | Pending |
| OPEN-01 | — | Pending |
| OPEN-02 | — | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 0
- Unmapped: 14 ⚠️

---
*Requirements defined: 2026-03-18*
*Last updated: 2026-03-18 after initial definition*
