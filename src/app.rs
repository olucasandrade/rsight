use tokio::sync::mpsc;
use crate::types::SearchResult;
use crate::search::SearchHandle;

/// Which tab is currently active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabKind {
    Files,
    Folders,
    Contents,
    AiConversations,
}

impl TabKind {
    /// All tabs in display order.
    pub fn all() -> &'static [TabKind] {
        &[
            TabKind::Files,
            TabKind::Folders,
            TabKind::Contents,
            TabKind::AiConversations,
        ]
    }

    /// Display label for the tab bar.
    pub fn label(&self) -> &'static str {
        match self {
            TabKind::Files => "Files",
            TabKind::Folders => "Folders",
            TabKind::Contents => "Contents",
            TabKind::AiConversations => "AI Conversations",
        }
    }

    /// AI Conversations tab is disabled until Phase 3.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, TabKind::AiConversations)
    }
}

/// All runtime state for the TUI application.
pub struct AppState {
    /// Current search query string (mirrors what the user typed).
    pub query: String,
    /// Active tab. Resets to Files when query is cleared and retyped.
    pub active_tab: TabKind,
    /// File name search results (capped at 100, sorted by fuzzy score desc).
    pub files: Vec<SearchResult>,
    /// Folder name search results (capped at 100, sorted by fuzzy score desc).
    pub folders: Vec<SearchResult>,
    /// Content match results (capped at 100, in discovery order).
    pub contents: Vec<SearchResult>,
    /// Selected row index within the active tab's result list.
    pub selected_index: usize,
    /// Handle for the running debounced search task. Abort on new keystroke.
    pub search_handle: Option<SearchHandle>,
    /// mpsc sender for search results. TUI drains the paired receiver.
    pub result_tx: mpsc::Sender<SearchResult>,
    /// Receiver for incoming search results from debounced_search.
    pub result_rx: mpsc::Receiver<SearchResult>,
    /// Status message shown in the status bar (e.g. "AI search coming in a future update").
    pub status_message: Option<String>,
    /// Whether the application should exit on the next loop iteration.
    pub should_quit: bool,
}

impl AppState {
    /// Create a fresh AppState. Call once at startup.
    pub fn new() -> Self {
        let (result_tx, result_rx) = mpsc::channel(4096);
        AppState {
            query: String::new(),
            active_tab: TabKind::Files,
            files: Vec::new(),
            folders: Vec::new(),
            contents: Vec::new(),
            selected_index: 0,
            search_handle: None,
            result_tx,
            result_rx,
            status_message: None,
            should_quit: false,
        }
    }

    /// Returns the result list for the currently active tab.
    pub fn active_results(&self) -> &[SearchResult] {
        match self.active_tab {
            TabKind::Files => &self.files,
            TabKind::Folders => &self.folders,
            TabKind::Contents => &self.contents,
            TabKind::AiConversations => &[], // disabled in Phase 2
        }
    }

    /// Clear all search results and reset selection. Call when query changes.
    pub fn clear_results(&mut self) {
        self.files.clear();
        self.folders.clear();
        self.contents.clear();
        self.selected_index = 0;
        self.status_message = None;
    }

    /// Push a SearchResult into the correct tab list.
    /// Enforces the 100-result cap per tab.
    pub fn push_result(&mut self, result: SearchResult) {
        const CAP: usize = 100;
        match &result {
            SearchResult::File { .. } => {
                if self.files.len() < CAP {
                    self.files.push(result);
                    // Sort by score descending (higher score = better match)
                    self.files.sort_by(|a, b| {
                        let score_a = if let SearchResult::File { score, .. } = a { score.unwrap_or(0) } else { 0 };
                        let score_b = if let SearchResult::File { score, .. } = b { score.unwrap_or(0) } else { 0 };
                        score_b.cmp(&score_a)
                    });
                }
            }
            SearchResult::Folder { .. } => {
                if self.folders.len() < CAP {
                    self.folders.push(result);
                    self.folders.sort_by(|a, b| {
                        let score_a = if let SearchResult::Folder { score, .. } = a { score.unwrap_or(0) } else { 0 };
                        let score_b = if let SearchResult::Folder { score, .. } = b { score.unwrap_or(0) } else { 0 };
                        score_b.cmp(&score_a)
                    });
                }
            }
            SearchResult::ContentMatch { .. } => {
                if self.contents.len() < CAP {
                    self.contents.push(result);
                }
            }
            SearchResult::AiConversation { .. } => {
                // AiConversation results will be routed to a dedicated tab in a later plan.
                // For now, silently drop them to keep existing tabs intact.
            }
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
