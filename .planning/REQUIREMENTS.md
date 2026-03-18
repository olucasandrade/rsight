# Requirements: rsight

**Defined:** 2026-03-18
**Core Value:** Find anything in your home directory — including AI conversation history — in under a second, without leaving the terminal.

## v1 Requirements

### TUI

- [x] **TUI-01**: User sees a search bar at the top of the screen that accepts input on launch
- [ ] **TUI-02**: Results are displayed in tabbed panes: Files, Folders, Contents, AI Conversations — PARTIAL: file/folder name search works; content search takes >30s (gap closure required)
- [x] **TUI-03**: User can navigate results with arrow keys, open with Enter, and clear/quit with Esc
- [x] **TUI-04**: Search query matches are highlighted in result text (filenames, content snippets, conversation excerpts)

### Search

- [x] **SRCH-01**: User can search file and folder names using fuzzy matching
- [x] **SRCH-02**: User can search file contents using exact full-text matching
- [x] **SRCH-03**: Results appear in under 1 second on a typical developer machine
- [x] **SRCH-04**: Search scope is the user's entire $HOME directory recursively

### AI Conversations

- [x] **AICV-01**: User can search Claude Code conversation content (~/.claude/projects/)
- [x] **AICV-02**: User can search Cursor conversation content (~/.cursor/)
- [x] **AICV-03**: Each AI conversation result shows a title or first-message summary
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
| TUI-01 | Phase 2 | Complete |
| TUI-02 | Phase 2 | Partial — gap closure required |
| TUI-03 | Phase 2 | Complete |
| TUI-04 | Phase 2 | Complete |
| SRCH-01 | Phase 1 | Complete |
| SRCH-02 | Phase 1 | Complete |
| SRCH-03 | Phase 1 | Complete |
| SRCH-04 | Phase 1 | Complete |
| AICV-01 | Phase 3 | Complete |
| AICV-02 | Phase 3 | Complete |
| AICV-03 | Phase 3 | Complete |
| AICV-04 | Phase 3 | Pending |
| OPEN-01 | Phase 3 | Pending |
| OPEN-02 | Phase 3 | Pending |

**Coverage:**
- v1 requirements: 14 total
- Mapped to phases: 14
- Unmapped: 0 ✓

---
*Requirements defined: 2026-03-18*
*Last updated: 2026-03-18 after roadmap creation*
