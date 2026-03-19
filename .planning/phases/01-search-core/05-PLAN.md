---
phase: 01-search-core
plan: 05
type: execute
wave: 4
depends_on: [04]
files_modified:
  - benches/search_bench.rs
  - Cargo.toml
autonomous: false
requirements: [SRCH-03]

must_haves:
  truths:
    - Running `target/release/rsight <query>` against the real $HOME completes and prints results in under 10 seconds wall time (smoke test — sub-1s is validated interactively)
    - The benchmark suite runs with `cargo bench` without panicking
    - Human visually confirms that results appear within ~1 second of launch on their machine
  artifacts:
    - benches/search_bench.rs
  key_links:
    - Phase 2 (TUI Shell) depends on Phase 1 shipping a confirmed-fast search API
---

<objective>
Validate the < 1 second performance target (SRCH-03) with a criterion benchmark and an interactive smoke test against the real $HOME directory.

Purpose: Phase 1 goal requires results within 1 second. This plan proves the implementation meets that target before Phase 2 begins.
Output: benches/search_bench.rs with a criterion benchmark; human checkpoint to confirm live performance.
</objective>

<execution_context>
@/Users/lucasandrade/.claude/get-shit-done/workflows/execute-plan.md
@/Users/lucasandrade/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/phases/01-search-core/01-CONTEXT.md
@.planning/phases/01-search-core/01-search-core-04-SUMMARY.md

<interfaces>
<!-- Public API from Plan 04 -->
pub async fn search(root: &str, query: &str) -> tokio::sync::mpsc::Receiver<SearchResult>;
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Add criterion benchmark for name and content search</name>
  <files>benches/search_bench.rs, Cargo.toml</files>
  <action>
Add criterion to Cargo.toml:

```toml
[dev-dependencies]
tempfile = "3"
criterion = { version = "0.5", features = ["async_tokio"] }

[[bench]]
name = "search_bench"
harness = false
```

Create benches/search_bench.rs:

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use rsight::{search_names, search_contents};
use std::fs;
use tempfile::TempDir;
use tokio::sync::mpsc;
use tokio::runtime::Runtime;

/// Build a temp directory tree for benchmarking.
/// Creates 500 text files across 10 subdirectories.
fn make_bench_tree() -> TempDir {
    let dir = TempDir::new().unwrap();
    for i in 0..10 {
        let sub = dir.path().join(format!("subdir{}", i));
        fs::create_dir(&sub).unwrap();
        for j in 0..50 {
            fs::write(
                sub.join(format!("file_{}_{}.txt", i, j)),
                format!("line one\nsearch_target content {}\nline three\n", j),
            ).unwrap();
        }
    }
    dir
}

fn bench_name_search(c: &mut Criterion) {
    let dir = make_bench_tree();
    let root = dir.path().to_str().unwrap().to_string();
    c.bench_function("name_search 500 files", |b| {
        b.iter(|| {
            let (tx, mut rx) = mpsc::channel(4096);
            search_names(&root, "file", tx);
            // drain
            while rx.try_recv().is_ok() {}
        });
    });
}

fn bench_content_search(c: &mut Criterion) {
    let dir = make_bench_tree();
    let root = dir.path().to_str().unwrap().to_string();
    c.bench_function("content_search 500 files", |b| {
        b.iter(|| {
            let (tx, mut rx) = mpsc::channel(4096);
            search_contents(&root, "search_target", tx);
            while rx.try_recv().is_ok() {}
        });
    });
}

criterion_group!(benches, bench_name_search, bench_content_search);
criterion_main!(benches);
```

Run the benchmark to confirm it compiles and executes:
```bash
cargo bench 2>&1 | tail -30
```

Note the mean times in the SUMMARY. The benchmark uses a small 500-file tree — real $HOME performance is validated in the checkpoint below.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && cargo bench 2>&1 | grep -E "time:|error" | head -20</automated>
  </verify>
  <done>cargo bench runs without errors. Benchmark output shows mean times for name_search and content_search on 500-file tree.</done>
</task>

<task type="checkpoint:human-verify">
  <name>Task 2: Interactive smoke test — confirm < 1s on real $HOME</name>
  <files></files>
  <action>
Run the following command and time the first results appearing:

```bash
cd /Users/lucasandrade/rsight
time target/release/rsight "readme"
```

This runs the binary against the real `$HOME`, searching for "readme". Observe:
1. Does the binary print results before the 10-second timeout?
2. Are both File/Folder results AND ContentMatch results in the output?
3. What is the wall time reported by `time`?

**If wall time > 5 seconds**: the parallelism may not be working. Check:
- `rayon` thread pool is active (check content_search.rs uses par_iter)
- `WalkBuilder::build_parallel()` is used in name_search.rs (not build())

**If results look correct and timing is under 5 seconds**: Phase 1 is complete.
The < 1 second requirement applies to first-result latency in the TUI (streaming), not total traversal time. The binary waits for 10 results then exits — total time includes full traversal.
  </action>
  <verify>
    <automated>cd /Users/lucasandrade/rsight && timeout 30 target/release/rsight "readme" 2>&1 | head -5</automated>
  </verify>
  <done>
User confirms:
- Binary runs and produces results
- Results include recognizable file names from $HOME
- Performance feels responsive (first results appear quickly)
- No panics or errors in output
  </done>
</task>

</tasks>

<verification>
```bash
cd /Users/lucasandrade/rsight
cargo bench 2>&1 | grep "time:"           # shows benchmark results
cargo test 2>&1 | grep "test result"      # all tests still pass
target/release/rsight "config" | head -5  # produces real results
```
</verification>

<success_criteria>
- `cargo bench` runs without errors and reports timing for both search types
- `cargo test` continues to pass (no regressions)
- Running `target/release/rsight <query>` against real $HOME produces results within a few seconds (total wall time)
- Human checkpoint confirms first results appear quickly — consistent with < 1s streaming target for TUI
</success_criteria>

<output>
After completion, create `.planning/phases/01-search-core/01-search-core-05-SUMMARY.md` with:
- Benchmark results (mean times for name_search and content_search on 500-file tree)
- Observations from the live $HOME smoke test
- Any performance concerns or optimizations identified for Phase 2
- Confirmation that Phase 1 is complete and ready for Phase 2 planning
</output>
