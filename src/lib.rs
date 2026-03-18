pub mod types;
pub mod name_search;
pub mod content_search;
pub mod search;
pub mod app;
pub mod ui;

pub use types::SearchResult;
pub use name_search::search_names;
pub use content_search::search_contents;
pub use search::search;
pub use search::debounced_search;
pub use search::SearchHandle;
pub use app::AppState;
pub use app::TabKind;
pub use ui::draw_ui;
