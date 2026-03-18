---
phase: 01-search-core
plan: 01
subsystem: infra
tags: [rust, cargo, tokio, rayon, ignore, fuzzy-matcher, async]

# Dependency graph
requires: []
provides:
  - Cargo.toml with ignore, tokio, rayon, fuzzy-matcher declared
  - SearchResult enum (File, Folder, ContentMatch variants) in src/types.rs
  - stub search() async fn returning mpsc::Receiver<SearchResult>
  - compilable Rust binary skeleton
affects: [02-search-core, 03-search-core, 04-search-core, 05-search-core, 02-tui-shell]

# Tech tracking
tech-stack:
  added:
    - ignore 0.4 (parallel directory traversal, ripgrep-based)
    - tokio 1 with full features (async runtime)
    - rayon 1 (data-parallel iterators)
    - fuzzy-matcher 0.3 (SkimMatcherV2 algorithm)
  patterns:
    - mpsc channel as streaming search result transport
    - lib.rs re-exports SearchResult as single import path for downstream

key-files:
  created:
    - Cargo.toml
    - src/main.rs
    - src/lib.rs
    - src/types.rs
  modified: []

key-decisions:
  - "Rust chosen as implementation language for raw performance and minimal memory overhead"
  - "ignore crate (ripgrep's traversal engine) selected for parallel file walking with gitignore support"
  - "mpsc channel pattern established as streaming API — search() returns Receiver<SearchResult>"
  - "SearchResult as owned-string enum (no lifetimes) for ergonomic channel transport"

patterns-established:
  - "Stream pattern: search functions push to mpsc::Sender, callers consume from Receiver"
  - "Type re-export pattern: lib.rs re-exports from submodules as single import surface"

requirements-completed: [SRCH-03, SRCH-04]

# Metrics
duration: 1min
completed: 2026-03-18
---

# Phase 1 Plan 01: Bootstrap — Cargo Workspace, SearchResult Types, and Dependency Stack

**Rust binary crate with SearchResult enum (File/Folder/ContentMatch), tokio mpsc streaming API stub, and ignore+rayon+fuzzy-matcher dependency stack wired for Plans 02-04.**

## Performance

- **Duration:** ~2 min
- **Started:** 2026-03-18T18:34:32Z
- **Completed:** 2026-03-18T18:36:10Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Initialized Rust binary crate with `cargo init --name rsight` (edition 2021)
- Declared all required dependencies: ignore 0.4, tokio 1 (full), rayon 1, fuzzy-matcher 0.3
- Defined canonical SearchResult enum with File, Folder, ContentMatch variants in src/types.rs
- Wired src/lib.rs with re-export of SearchResult and stub search() async function via mpsc channel
- `cargo check` passes with zero errors

## Task Commits

Each task was committed atomically:

1. **Task 1: Initialize Cargo project and declare dependencies** - `a42f6ed` (chore)
2. **Task 2: Define SearchResult types and library root** - `bcae510` (feat)

## Files Created/Modified
- `Cargo.toml` - Package metadata, all four dependencies, release profile (opt-level=3, lto=true, codegen-units=1)
- `src/main.rs` - Async entry point with tokio::main, consumes rsight::search stub
- `src/types.rs` - SearchResult enum: File {path, name, score}, Folder {path, name, score}, ContentMatch {path, line_number, line}
- `src/lib.rs` - pub mod types, re-exports SearchResult, stubs search() returning mpsc::Receiver

## Decisions Made
- Used edition 2021 (not 2024 which cargo init defaulted to) for stability across toolchain versions
- SearchResult uses owned Strings (no lifetimes) to allow safe transport across mpsc channel boundaries
- stub search() drops the sender immediately — produces no results, channel closes on first poll

## Deviations from Plan

None — plan executed exactly as written. The only minor note: `cargo init` defaulted to edition "2024"; corrected to "2021" per the plan spec (this is within task scope, not a deviation).

## Issues Encountered

None.

## User Setup Required

None — no external service configuration required.

## Next Phase Readiness
- SearchResult type contract is published; Plans 02-04 can implement File, Folder, and ContentMatch search by adding to lib.rs
- Dependency stack is locked in Cargo.lock after first `cargo check` — reproducible builds guaranteed
- src/types.rs exports are the stable API surface Phase 2 (TUI Shell) will consume

---
*Phase: 01-search-core*
*Completed: 2026-03-18*

## Self-Check: PASSED

- Cargo.toml: FOUND
- src/lib.rs: FOUND
- src/types.rs: FOUND
- src/main.rs: FOUND
- SUMMARY.md: FOUND
- Commit a42f6ed: FOUND
- Commit bcae510: FOUND
