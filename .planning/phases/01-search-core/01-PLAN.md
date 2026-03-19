---
phase: 01-search-core
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - Cargo.toml
  - src/main.rs
  - src/lib.rs
  - src/types.rs
autonomous: true
requirements: [SRCH-03, SRCH-04]

must_haves:
  truths:
    - "cargo build --release" succeeds with zero errors
    - "src/types.rs" defines SearchResult enum with File, Folder, ContentMatch variants
    - The ignore crate is listed in Cargo.toml dependencies
    - The tokio async runtime is listed in Cargo.toml dependencies
    - Building with release profile produces a binary at target/release/rsight
  artifacts:
    - Cargo.toml
    - src/lib.rs
    - src/types.rs
    - src/main.rs
  key_links:
    - src/types.rs exports SearchResult — this is the contract Phase 2 consumes
    - src/lib.rs re-exports types — single import path for downstream plans
---

<objective>
Bootstrap the rsight Rust project: initialize Cargo workspace, define the SearchResult type contract, and wire the dependency stack (ignore + tokio + rayon).

Purpose: Every subsequent plan in Phase 1 depends on the type definitions and crate dependencies established here. This plan creates zero search logic — only the foundation.
Output: A compilable Rust binary skeleton with SearchResult types and all required crate dependencies declared.
</objective>

<execution_context>
@/Users/lucasandrade/.claude/get-shit-done/workflows/execute-plan.md
@/Users/lucasandrade/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/phases/01-search-core/01-CONTEXT.md
</context>

<tasks>

<task type="auto">
  <name>Task 1: Initialize Cargo project and declare dependencies</name>
  <files>Cargo.toml, src/main.rs</files>
  <action>
Run `cargo init --name rsight` in the project root /Users/lucasandrade/rsight to create a new binary crate.

Then edit Cargo.toml to add these dependencies under [dependencies]:

```toml
[package]
name = "rsight"
version = "0.1.0"
edition = "2021"

[dependencies]
ignore = "0.4"          # parallel directory traversal (battle-tested, used by ripgrep)
tokio = { version = "1", features = ["full"] }  # async runtime for debounced search cancellation
rayon = "1"             # data-parallel iterators for content search
fuzzy-matcher = "0.3"   # fuzzy name matching (SkimMatcherV2 algorithm)

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

Edit src/main.rs to be a minimal async entry point:

```rust
use rsight::search;

#[tokio::main]
async fn main() {
    // CLI smoke test: search for "main" in $HOME, print first 5 results
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let mut rx = search(&home, "main").await;
    let mut count = 0;
    while let Some(result) = rx.recv().await {
        println!("{:?}", result);
        count += 1;
        if count >= 5 { break; }
    }
}
```

This main.rs references `rsight::search` which does not exist yet — that is expected. It will compile only after Plans 02-04 are complete. For now, `cargo check` (not `cargo build`) must pass after Plan 01 is done only if we stub the lib — see Task 2.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cat Cargo.toml | grep -E "ignore|tokio|rayon|fuzzy-matcher"</automated>
  </verify>
  <done>Cargo.toml lists ignore, tokio, rayon, and fuzzy-matcher as dependencies. src/main.rs exists.</done>
</task>

<task type="auto">
  <name>Task 2: Define SearchResult types and library root</name>
  <files>src/lib.rs, src/types.rs</files>
  <action>
Create src/types.rs with the canonical SearchResult type that all search functions return and Phase 2 (TUI Shell) will consume:

```rust
/// A single search result from any search category.
#[derive(Debug, Clone)]
pub enum SearchResult {
    /// A file whose name matched the query.
    File {
        /// Absolute path to the file.
        path: String,
        /// The file name component, for display.
        name: String,
        /// Fuzzy match score (higher = better match). None for non-fuzzy results.
        score: Option<i64>,
    },
    /// A directory whose name matched the query.
    Folder {
        /// Absolute path to the directory.
        path: String,
        /// The directory name component, for display.
        name: String,
        /// Fuzzy match score. None for non-fuzzy results.
        score: Option<i64>,
    },
    /// A line within a file whose contents matched the query.
    ContentMatch {
        /// Absolute path to the file containing the match.
        path: String,
        /// 1-based line number of the match.
        line_number: u64,
        /// The full text of the matching line (trimmed to reasonable length).
        line: String,
    },
}
```

Create src/lib.rs that re-exports types and stubs the search function so the project compiles:

```rust
pub mod types;
pub use types::SearchResult;

use tokio::sync::mpsc;

/// Stub search function — implemented in Plans 02-04.
/// Returns an mpsc receiver that will yield SearchResult items.
pub async fn search(_root: &str, _query: &str) -> mpsc::Receiver<SearchResult> {
    let (tx, rx) = mpsc::channel(1024);
    drop(tx); // immediately closed — stub produces no results
    rx
}
```

After creating these files, run `cargo check` to confirm zero compilation errors.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo check 2>&1 | tail -5</automated>
  </verify>
  <done>`cargo check` exits 0 with no errors. src/types.rs defines SearchResult enum with File, Folder, ContentMatch variants. src/lib.rs re-exports SearchResult and exports a stub `search` async function.</done>
</task>

</tasks>

<verification>
Run the following after both tasks complete:

```bash
cd /Users/lucasandrade/rsight
cargo check 2>&1 | grep -E "^error" | wc -l  # must be 0
grep "SearchResult" src/types.rs | wc -l      # must be > 0
grep "ignore" Cargo.toml                       # must appear
```
</verification>

<success_criteria>
- `cargo check` passes with zero errors
- SearchResult enum exists in src/types.rs with File, Folder, ContentMatch variants
- Cargo.toml declares ignore, tokio, rayon, fuzzy-matcher
- src/lib.rs compiles and re-exports SearchResult
</success_criteria>

<output>
After completion, create `.planning/phases/01-search-core/01-search-core-01-SUMMARY.md` with:
- What was created
- Dependency versions locked in Cargo.toml
- SearchResult type signature (paste the enum definition)
- Any deviations from the plan
</output>
