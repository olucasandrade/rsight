---
phase: 01-search-core
plan: 02
type: execute
wave: 2
depends_on: [01]
files_modified:
  - src/name_search.rs
  - src/lib.rs
autonomous: true
requirements: [SRCH-01, SRCH-04]

must_haves:
  truths:
    - Fuzzy name matching scores file and folder names against the query using SkimMatcherV2
    - Traversal includes hidden directories (dotfiles enabled on the ignore WalkBuilder)
    - Excluded directories (node_modules, .git, target, vendor, build) are skipped during walk
    - Results are sent through an mpsc channel as they are found (streaming, not batch)
    - Files and folders are both returned as SearchResult::File and SearchResult::Folder variants
  artifacts:
    - src/name_search.rs
  key_links:
    - name_search.rs exports `pub fn search_names(root, query, tx)` — called by the unified search API in Plan 04
    - Uses SearchResult from src/types.rs (defined in Plan 01)
---

<objective>
Implement fuzzy file and folder name search over the entire $HOME directory using the `ignore` crate for parallel traversal and `fuzzy-matcher` for scoring.

Purpose: Satisfies SRCH-01 (fuzzy name matching) and SRCH-04 (entire $HOME, including hidden dirs).
Output: src/name_search.rs with a `search_names` function that walks the filesystem in parallel and streams SearchResult::File and SearchResult::Folder matches into an mpsc sender.
</objective>

<execution_context>
@/Users/lucasandrade/.claude/get-shit-done/workflows/execute-plan.md
@/Users/lucasandrade/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/phases/01-search-core/01-CONTEXT.md
@.planning/phases/01-search-core/01-search-core-01-SUMMARY.md

<interfaces>
<!-- From src/types.rs (Plan 01 output) -->
pub enum SearchResult {
    File   { path: String, name: String, score: Option<i64> },
    Folder { path: String, name: String, score: Option<i64> },
    ContentMatch { path: String, line_number: u64, line: String },
}
</interfaces>
</context>

<tasks>

<task type="auto" tdd="true">
  <name>Task 1: Implement fuzzy name search with parallel traversal</name>
  <files>src/name_search.rs, src/lib.rs</files>
  <behavior>
    - Test 1: search_names on a temp dir containing "foobar.txt" with query "fba" returns a File result (fuzzy match)
    - Test 2: search_names on a temp dir containing a hidden subdir ".hidden/" with a file inside returns the file (hidden dirs traversed)
    - Test 3: search_names on a temp dir containing "node_modules/lodash/index.js" does NOT return that path (excluded dir skipped)
    - Test 4: search_names on a temp dir containing a folder "projects/" with query "proj" returns a Folder result
    - Test 5: search_names with an empty query returns no results (zero-length query guard)
  </behavior>
  <action>
Create src/name_search.rs implementing the following:

```rust
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use ignore::WalkBuilder;
use tokio::sync::mpsc;
use crate::types::SearchResult;

/// Directories always excluded from traversal regardless of .gitignore rules.
const EXCLUDED_DIRS: &[&str] = &["node_modules", ".git", "target", "vendor", "build"];

/// Search file and folder names under `root` for entries matching `query` using fuzzy matching.
/// Results are sent to `tx` as they are found. The function returns when traversal is complete.
///
/// # Arguments
/// - `root`: Absolute path to search root (typically $HOME)
/// - `query`: The search string. Empty query produces no results.
/// - `tx`: mpsc sender; caller drops receiver to cancel (backpressure handled by bounded channel)
pub fn search_names(root: &str, query: &str, tx: mpsc::Sender<SearchResult>) {
    if query.is_empty() {
        return;
    }

    let matcher = SkimMatcherV2::default();
    let query_owned = query.to_string();

    WalkBuilder::new(root)
        .hidden(false)          // traverse hidden directories (SRCH-04 requirement)
        .git_ignore(false)      // do not skip files just because they're gitignored
        .filter_entry(|entry| {
            // Skip excluded directory names at any depth
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = entry.file_name().to_string_lossy();
                !EXCLUDED_DIRS.contains(&name.as_ref())
            } else {
                true
            }
        })
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            let matcher = SkimMatcherV2::default();
            let query = query_owned.clone();
            Box::new(move |result| {
                use ignore::WalkState;
                let entry = match result {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };
                // Skip the root itself
                if entry.depth() == 0 {
                    return WalkState::Continue;
                }
                let path = entry.path();
                let name = path.file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default();
                if name.is_empty() {
                    return WalkState::Continue;
                }
                if let Some(score) = matcher.fuzzy_match(&name, &query) {
                    let path_str = path.to_string_lossy().into_owned();
                    let result = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        SearchResult::Folder { path: path_str, name, score: Some(score) }
                    } else {
                        SearchResult::File { path: path_str, name, score: Some(score) }
                    };
                    // If receiver dropped (cancelled), stop walking
                    if tx.blocking_send(result).is_err() {
                        return WalkState::Quit;
                    }
                }
                WalkState::Continue
            })
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::sync::mpsc;

    fn collect_sync(root: &str, query: &str) -> Vec<SearchResult> {
        let (tx, mut rx) = mpsc::channel(1024);
        search_names(root, query, tx);
        let mut results = Vec::new();
        while let Ok(r) = rx.try_recv() {
            results.push(r);
        }
        results
    }

    #[test]
    fn fuzzy_matches_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("foobar.txt"), "").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "fba");
        assert!(results.iter().any(|r| matches!(r, SearchResult::File { name, .. } if name == "foobar.txt")));
    }

    #[test]
    fn traverses_hidden_dirs() {
        let dir = TempDir::new().unwrap();
        let hidden = dir.path().join(".hidden");
        fs::create_dir(&hidden).unwrap();
        fs::write(hidden.join("secret.txt"), "").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "secret");
        assert!(results.iter().any(|r| matches!(r, SearchResult::File { name, .. } if name == "secret.txt")));
    }

    #[test]
    fn skips_node_modules() {
        let dir = TempDir::new().unwrap();
        let nm = dir.path().join("node_modules").join("lodash");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("index.js"), "").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "index");
        assert!(results.is_empty(), "node_modules should be excluded");
    }

    #[test]
    fn matches_folders() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("projects")).unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "proj");
        assert!(results.iter().any(|r| matches!(r, SearchResult::Folder { name, .. } if name == "projects")));
    }

    #[test]
    fn empty_query_returns_nothing() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("anything.txt"), "").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "");
        assert!(results.is_empty());
    }
}
```

Add `tempfile` to Cargo.toml under `[dev-dependencies]`:
```toml
[dev-dependencies]
tempfile = "3"
```

Register the module in src/lib.rs:
```rust
pub mod name_search;
pub use name_search::search_names;
```
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo test name_search -- --nocapture 2>&1 | tail -20</automated>
  </verify>
  <done>All 5 tests in name_search module pass. `cargo test name_search` exits 0.</done>
</task>

</tasks>

<verification>
```bash
cd /Users/lucasandrade/rsight
cargo test name_search 2>&1 | grep -E "test result|FAILED"
# Expected: "test result: ok. 5 passed"
```
</verification>

<success_criteria>
- All 5 name_search tests pass
- search_names function is exported from src/lib.rs
- Hidden directories are traversed (confirmed by test)
- node_modules and other excluded dirs are skipped (confirmed by test)
- Fuzzy matching returns File and Folder variants with scores
</success_criteria>

<output>
After completion, create `.planning/phases/01-search-core/01-search-core-02-SUMMARY.md` with:
- Test results output
- Any crate version adjustments made
- search_names function signature as implemented
- Performance observations (optional: time a quick walk over a real directory)
</output>
