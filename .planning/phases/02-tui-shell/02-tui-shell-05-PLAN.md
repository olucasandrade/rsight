---
phase: 02-tui-shell
plan: 05
type: standard
wave: 4
depends_on:
  - 02-tui-shell-04-PLAN.md
files_modified: []
autonomous: false
must_haves:
  - Launching rsight shows a search bar at the top of the terminal
  - Typing a query updates results in real-time (within ~150ms of last keystroke)
  - Results appear under correct tabs (Files, Folders, Contents)
  - AI Conversations tab is visible but grayed out and shows placeholder on Enter
  - Arrow keys move the selection highlight up and down
  - Tab / Shift+Tab switches between enabled tabs
  - Matched query text is visually highlighted (yellow/bold) in result rows
  - Esc exits the TUI and restores the terminal
requirements:
  - TUI-01
  - TUI-02
  - TUI-03
  - TUI-04
---

# Phase 2 Plan 05: Visual Verification Checkpoint

**Objective:** Human verification that the TUI looks and behaves correctly end-to-end. This checkpoint ensures all four TUI requirements are satisfied before Phase 2 is marked complete.

## Context

This plan has no implementation tasks. Plans 01-04 built and wired the complete TUI. This plan pauses for the user to run the binary and verify the experience matches the Phase 2 goal.

## Tasks

<task type="checkpoint:human-verify">
  <name>Checkpoint: Verify TUI functionality end-to-end</name>
  <action>
Run the rsight binary and verify the following:

```bash
cd /Users/lucasandrade/rsight && cargo run
```

**What to check:**

1. **TUI-01 — Search bar visible on launch**
   - [ ] Terminal goes full-screen (alternate buffer)
   - [ ] Search bar appears at the top with a cursor
   - [ ] Typing immediately shows characters in the search bar

2. **TUI-02 — Tabbed result panes update as you type**
   - [ ] Type a short query (e.g. "main") — results appear under Files and Contents tabs
   - [ ] Files tab shows filename results with full path dimmed
   - [ ] Contents tab shows `path:line  snippet` format
   - [ ] AI Conversations tab is visible but grayed out
   - [ ] Switching tabs shows the appropriate result list
   - [ ] Status bar shows result count and key hints

3. **TUI-03 — Keyboard navigation works**
   - [ ] Arrow Down / Up moves the selection highlight
   - [ ] Tab switches to the next enabled tab (Files → Folders → Contents → Files)
   - [ ] Shift+Tab switches backwards
   - [ ] Enter on a file result opens it (system default)
   - [ ] Ctrl+C copies the path and shows "Copied: ..." in the status bar
   - [ ] Pressing Enter on AI Conversations tab shows "AI search coming in a future update"
   - [ ] Esc exits cleanly and restores the terminal (normal shell prompt visible)

4. **TUI-04 — Match highlighting visible**
   - [ ] The query text (or its substring) appears in yellow/bold within file names and content snippets
   - [ ] Non-matching text is rendered in normal style
   - [ ] Path portions are dimmed gray

**If any item fails:** Note which requirement (TUI-0X) and what the actual behavior was. The executor will create a gap closure plan to address it.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo build 2>&1 | tail -3</automated>
  </verify>
  <done>All four TUI requirements (TUI-01, TUI-02, TUI-03, TUI-04) are visually confirmed working. Terminal is restored cleanly on Esc.</done>
</task>

## Verification

User runs `cargo run` and confirms the checklist above. Phase 2 is complete when all items are checked.

## Success Criteria

- TUI-01: Search bar visible and responsive on launch
- TUI-02: Results appear in tabbed panes and update live as user types
- TUI-03: All keyboard shortcuts work (arrows, Tab, Enter, Ctrl+C, Esc)
- TUI-04: Query matches are visually highlighted in yellow bold in result rows
- Terminal state is clean after exit (no artifacts, cursor restored)

## Output

No files modified. This is a verification-only checkpoint.
