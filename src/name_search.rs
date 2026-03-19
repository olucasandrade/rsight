use ignore::WalkBuilder;
use tokio::sync::mpsc;
use crate::types::SearchResult;

/// Directories always excluded from traversal regardless of .gitignore rules.
const EXCLUDED_DIRS: &[&str] = &["node_modules", ".git", "target", "vendor", "build"];

/// Search file and folder names under `root` for entries matching `query` using exact
/// case-insensitive substring matching. Results are sent to `tx` as they are found.
///
/// # Arguments
/// - `root`: Absolute path to search root (typically $HOME)
/// - `query`: The search string. Empty query produces no results.
/// - `tx`: mpsc sender; caller drops receiver to cancel (backpressure handled by bounded channel)
pub fn search_names(root: &str, query: &str, tx: mpsc::Sender<SearchResult>) {
    if query.is_empty() {
        return;
    }

    let query_lower = query.to_lowercase();

    WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(false)
        .max_depth(Some(10))
        .filter_entry(|entry| {
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
            let query_lower = query_lower.clone();
            Box::new(move |result| {
                use ignore::WalkState;
                let entry = match result {
                    Ok(e) => e,
                    Err(_) => return WalkState::Continue,
                };
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
                if name.to_lowercase().contains(&query_lower) {
                    let path_str = path.to_string_lossy().into_owned();
                    let result = if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                        SearchResult::Folder { path: path_str, name, score: None }
                    } else {
                        SearchResult::File { path: path_str, name, score: None }
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
    fn exact_matches_file() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("foobar.txt"), "").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "ooba");
        assert!(results.iter().any(|r| matches!(r, SearchResult::File { name, .. } if name == "foobar.txt")));
    }

    #[test]
    fn exact_match_is_case_insensitive() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("FooBar.txt"), "").unwrap();
        let results = collect_sync(dir.path().to_str().unwrap(), "foobar");
        assert!(results.iter().any(|r| matches!(r, SearchResult::File { name, .. } if name == "FooBar.txt")));
    }

    #[test]
    fn fuzzy_pattern_no_longer_matches() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("foobar.txt"), "").unwrap();
        // "fba" is a fuzzy pattern — should NOT match with exact search
        let results = collect_sync(dir.path().to_str().unwrap(), "fba");
        assert!(results.is_empty(), "fuzzy pattern should not match with exact search");
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
