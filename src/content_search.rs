use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
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
];
/// Maximum file size to search in bytes (1 MB).
const MAX_FILE_SIZE: u64 = 1_024 * 1_024;
/// Number of bytes to inspect for binary detection.
const BINARY_CHECK_BYTES: usize = 8_192;
/// Maximum number of content matches to collect before stopping.
const MAX_CONTENT_RESULTS: usize = 100;

/// Returns true if the byte slice looks like binary (contains a null byte).
fn is_binary(buf: &[u8]) -> bool {
    buf.contains(&0u8)
}

/// Search the contents of all text files under `root` for lines containing `query`.
/// Results are sent to `tx` as matches are found.
/// Stops after collecting MAX_CONTENT_RESULTS matches.
pub fn search_contents(root: &str, query: &str, tx: mpsc::Sender<SearchResult>) {
    if query.is_empty() {
        return;
    }

    // Collect file paths using parallel WalkBuilder traversal.
    // build_parallel() distributes directory traversal across threads, making
    // the walk itself faster on large trees (e.g. $HOME with many subdirs).
    let mut file_paths: Vec<std::path::PathBuf> = Vec::new();
    let (path_tx, path_rx) = std::sync::mpsc::channel();

    WalkBuilder::new(root)
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
                            let _ = path_tx.send(entry.into_path());
                        }
                    }
                }
                WalkState::Continue
            })
        });
    // Drop the original sender so the receiver closes when all walk threads finish
    drop(path_tx);
    while let Ok(p) = path_rx.recv() {
        file_paths.push(p);
    }

    // Shared counter for result cap. Checked atomically across rayon threads.
    let match_count = Arc::new(AtomicUsize::new(0));

    file_paths.par_iter().for_each(|path| {
        if tx.is_closed() {
            return;
        }
        if match_count.load(Ordering::Relaxed) >= MAX_CONTENT_RESULTS {
            return;
        }
        let Ok(bytes) = fs::read(path) else { return };
        let check_len = bytes.len().min(BINARY_CHECK_BYTES);
        if is_binary(&bytes[..check_len]) {
            return;
        }
        let path_str = path.to_string_lossy().into_owned();
        for (idx, line) in bytes.split(|&b| b == b'\n').enumerate() {
            if tx.is_closed() { return; }
            if match_count.load(Ordering::Relaxed) >= MAX_CONTENT_RESULTS {
                return;
            }
            let Ok(line_str) = std::str::from_utf8(line) else { continue };
            if line_str.contains(query) {
                let result = SearchResult::ContentMatch {
                    path: path_str.clone(),
                    line_number: (idx + 1) as u64,
                    line: line_str.trim_end().to_string(),
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
