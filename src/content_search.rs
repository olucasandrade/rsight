use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use tokio::sync::mpsc;
use crate::types::SearchResult;

const EXCLUDED_DIRS: &[&str] = &[
    // Development build artifacts and dependency caches
    "node_modules", ".git", "target", "vendor", "build",
    // Rust toolchain caches (can be gigabytes)
    ".cargo", ".rustup",
    // macOS system and media directories (large, rarely contain searchable text)
    "Library", "Applications", ".Trash",
    "Movies", "Music", "Pictures",
    // AI tool data — handled by AI conversation search, not content search
    ".claude", ".cursor", ".codex",
];
/// Maximum file size to search in bytes (1 MB).
const MAX_FILE_SIZE: u64 = 1_024 * 1_024;
/// Maximum number of content matches to collect before stopping.
const MAX_CONTENT_RESULTS: usize = 100;

/// Returns true if the byte slice looks like binary (contains a null byte).
fn is_binary(buf: &[u8]) -> bool {
    buf.contains(&0u8)
}

/// Search the contents of all text files under `root` for lines containing `query`.
/// Results are sent to `tx` as matches are found.
/// Stops after collecting MAX_CONTENT_RESULTS matches.
///
/// Memory strategy: paths stream from the walker into rayon via par_bridge() —
/// no Vec<PathBuf> is collected. Files are read line-by-line via BufReader so only
/// one 8 KB buffer per rayon thread is live at a time, not a full 1 MB per file.
pub fn search_contents(root: &str, query: &str, tx: mpsc::Sender<SearchResult>) {
    if query.is_empty() {
        return;
    }

    // Walker sends paths as found; rayon consumes them via par_bridge().
    // No collection into a Vec — paths are processed and discarded as they arrive.
    let (path_tx, path_rx) = std::sync::mpsc::channel::<std::path::PathBuf>();

    let root_owned = root.to_string();
    thread::spawn(move || {
        WalkBuilder::new(&root_owned)
            .hidden(false)
            .git_ignore(false)
            .max_depth(Some(10))
            .filter_entry(|e| {
                if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let name = e.file_name().to_string_lossy();
                    !EXCLUDED_DIRS.contains(&name.as_ref())
                } else {
                    true
                }
            })
            .build_parallel()
            .run(|| {
                let path_tx = path_tx.clone();
                Box::new(move |entry_result| {
                    use ignore::WalkState;
                    let Ok(entry) = entry_result else { return WalkState::Continue };
                    if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        if let Ok(meta) = entry.metadata() {
                            if meta.len() <= MAX_FILE_SIZE {
                                if path_tx.send(entry.into_path()).is_err() {
                                    return WalkState::Quit;
                                }
                            }
                        }
                    }
                    WalkState::Continue
                })
            });
    });

    let match_count = Arc::new(AtomicUsize::new(0));

    // par_bridge() pulls from the channel iterator in parallel without buffering all paths.
    path_rx.into_iter().par_bridge().for_each(|path| {
        if tx.is_closed() {
            return;
        }
        if match_count.load(Ordering::Relaxed) >= MAX_CONTENT_RESULTS {
            return;
        }

        let Ok(file) = File::open(&path) else { return };
        let mut reader = BufReader::new(file);

        // Peek at the first bytes for binary detection without consuming them.
        // fill_buf() fills the internal 8 KB buffer and returns a slice;
        // the bytes remain buffered for the subsequent line reads.
        {
            let header = reader.fill_buf().unwrap_or(&[]);
            if is_binary(header) {
                return;
            }
        }

        let path_str = path.to_string_lossy().into_owned();
        let mut line_buf = String::new();
        let mut line_number: u64 = 0;

        loop {
            if tx.is_closed() { return; }
            if match_count.load(Ordering::Relaxed) >= MAX_CONTENT_RESULTS { return; }

            line_buf.clear();
            match reader.read_line(&mut line_buf) {
                Ok(0) => break,
                Ok(_) => {}
                Err(_) => break,
            }
            line_number += 1;

            if line_buf.contains(query) {
                let result = SearchResult::ContentMatch {
                    path: path_str.clone(),
                    line_number,
                    line: line_buf.trim_end().to_string(),
                };
                if tx.blocking_send(result).is_ok() {
                    match_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::sync::mpsc;

    fn collect_sync(root: &str, query: &str) -> Vec<SearchResult> {
        let (tx, mut rx) = mpsc::channel(4096);
        search_contents(root, query, tx);
        let mut results = Vec::new();
        while let Ok(r) = rx.try_recv() {
            results.push(r);
        }
        results
    }

    #[test]
    fn finds_matching_line() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("notes.txt"), "hello world\nno match here\n").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        assert_eq!(results.len(), 1);
        match &results[0] {
            SearchResult::ContentMatch { line_number, line, .. } => {
                assert_eq!(*line_number, 1);
                assert!(line.contains("hello world"));
            }
            _ => panic!("expected ContentMatch"),
        }
    }

    #[test]
    fn skips_binary_files() {
        let dir = TempDir::new().unwrap();
            let mut data = b"hello world".to_vec();
        data.push(0u8);
        fs::write(dir.path().join("binary.bin"), data).unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        assert!(results.is_empty(), "binary file should be skipped");
    }

    #[test]
    fn skips_files_over_1mb() {
        let dir = TempDir::new().unwrap();
        let big = vec![b'a'; MAX_FILE_SIZE as usize + 1];
        fs::write(dir.path().join("big.txt"), big).unwrap();
        fs::write(dir.path().join("small.txt"), "hello world\n").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        assert_eq!(results.len(), 1);
        match &results[0] {
            SearchResult::ContentMatch { path, .. } => {
                assert!(path.ends_with("small.txt"));
            }
            _ => panic!("expected ContentMatch"),
        }
    }

    #[test]
    fn traverses_hidden_dirs() {
        let dir = TempDir::new().unwrap();
        let hidden = dir.path().join(".hidden");
        fs::create_dir(&hidden).unwrap();
        fs::write(hidden.join("notes.txt"), "hello world\n").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        assert!(!results.is_empty(), "hidden dir file should be found");
    }

    #[test]
    fn skips_node_modules() {
        let dir = TempDir::new().unwrap();
        let nm = dir.path().join("node_modules").join("pkg");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("index.js"), "hello world\n").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        assert!(results.is_empty(), "node_modules should be excluded");
    }

    #[test]
    fn empty_query_returns_nothing() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("file.txt"), "hello\n").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "");
        assert!(results.is_empty());
    }

    #[test]
    fn multi_line_correct_numbers() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("multi.txt"), "skip\nhello\nskip\nhello again\n").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        assert_eq!(results.len(), 2);
        let line_numbers: Vec<u64> = results.iter().map(|r| match r {
            SearchResult::ContentMatch { line_number, .. } => *line_number,
            _ => 0,
        }).collect();
        assert!(line_numbers.contains(&2));
        assert!(line_numbers.contains(&4));
    }
}
