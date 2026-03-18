use criterion::{criterion_group, criterion_main, Criterion};
use rsight::{search_names, search_contents};
use std::fs;
use tempfile::TempDir;
use tokio::sync::mpsc;

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
