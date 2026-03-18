use crate::types::SearchResult;
use tokio::sync::mpsc;

const EXCLUDED_DIRS: &[&str] = &["node_modules", ".git", "target", "vendor", "build"];
/// Maximum file size to search in bytes (1 MB).
const MAX_FILE_SIZE: u64 = 1_024 * 1_024;
/// Number of bytes to inspect for binary detection.
const BINARY_CHECK_BYTES: usize = 8_192;

/// Returns true if the byte slice looks like binary (contains a null byte).
fn is_binary(_buf: &[u8]) -> bool {
    unimplemented!("RED phase")
}

/// Search the contents of all text files under `root` for lines containing `query`.
/// Results are sent to `tx` as matches are found.
pub fn search_contents(_root: &str, _query: &str, _tx: mpsc::Sender<SearchResult>) {
    unimplemented!("RED phase")
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
