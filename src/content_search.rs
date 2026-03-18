use ignore::WalkBuilder;
use rayon::prelude::*;
use std::fs;
use tokio::sync::mpsc;
use crate::types::SearchResult;

const EXCLUDED_DIRS: &[&str] = &["node_modules", ".git", "target", "vendor", "build"];
/// Maximum file size to search in bytes (1 MB).
const MAX_FILE_SIZE: u64 = 1_024 * 1_024;
/// Number of bytes to inspect for binary detection.
const BINARY_CHECK_BYTES: usize = 8_192;

/// Returns true if the byte slice looks like binary (contains a null byte).
fn is_binary(buf: &[u8]) -> bool {
    buf.contains(&0u8)
}

/// Search the contents of all text files under `root` for lines containing `query`.
/// Results are sent to `tx` as matches are found.
pub fn search_contents(root: &str, query: &str, tx: mpsc::Sender<SearchResult>) {
    if query.is_empty() {
        return;
    }

    // Collect file paths first, then search in parallel with rayon.
    // WalkBuilder handles traversal; rayon handles parallel file reads.
    let mut file_paths: Vec<std::path::PathBuf> = Vec::new();

    for entry in WalkBuilder::new(root)
        .hidden(false)       // include hidden dirs (SRCH-04)
        .git_ignore(false)   // do not skip gitignored files
        .filter_entry(|e| {
            if e.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let name = e.file_name().to_string_lossy();
                !EXCLUDED_DIRS.contains(&name.as_ref())
            } else {
                true
            }
        })
        .build()
        .flatten()
    {
        if entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            // Apply size filter eagerly
            if let Ok(meta) = entry.metadata() {
                if meta.len() <= MAX_FILE_SIZE {
                    file_paths.push(entry.into_path());
                }
            }
        }
    }

    // Parallel content search with rayon
    file_paths.par_iter().for_each(|path| {
        if tx.is_closed() {
            return; // receiver dropped — cancelled
        }
        let Ok(bytes) = fs::read(path) else { return };
        // Binary detection: check first BINARY_CHECK_BYTES
        let check_len = bytes.len().min(BINARY_CHECK_BYTES);
        if is_binary(&bytes[..check_len]) {
            return;
        }
        // Line-by-line search
        let path_str = path.to_string_lossy().into_owned();
        for (idx, line) in bytes.split(|&b| b == b'\n').enumerate() {
            if tx.is_closed() { return; }
            // Try to decode as UTF-8; skip lines that aren't valid UTF-8
            let Ok(line_str) = std::str::from_utf8(line) else { continue };
            if line_str.contains(query) {
                let result = SearchResult::ContentMatch {
                    path: path_str.clone(),
                    line_number: (idx + 1) as u64,
                    line: line_str.trim_end().to_string(),
                };
                let _ = tx.blocking_send(result);
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
        // Write a file with null bytes (binary)
        let mut data = b"hello world".to_vec();
        data.push(0u8); // null byte = binary
        fs::write(dir.path().join("binary.bin"), data).unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        assert!(results.is_empty(), "binary file should be skipped");
    }

    #[test]
    fn skips_files_over_1mb() {
        let dir = TempDir::new().unwrap();
        // Create file slightly over 1MB
        let big = vec![b'a'; MAX_FILE_SIZE as usize + 1];
        fs::write(dir.path().join("big.txt"), big).unwrap();
        // Create small file with same content marker
        fs::write(dir.path().join("small.txt"), "hello world\n").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "hello");
        // Only small.txt should appear
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
