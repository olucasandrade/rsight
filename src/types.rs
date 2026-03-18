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
