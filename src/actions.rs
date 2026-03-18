use std::process::Command;
use crate::types::SearchResult;

/// Open a search result with the appropriate application.
/// - File/Folder: uses macOS `open` command (system default app)
/// - ContentMatch: uses $EDITOR at line number if set, else falls back to `open`
pub fn open_result(result: &SearchResult) {
    match result {
        SearchResult::File { path, .. } | SearchResult::Folder { path, .. } => {
            let _ = Command::new("open").arg(path).spawn();
        }
        SearchResult::ContentMatch { path, line_number, .. } => {
            if let Ok(editor) = std::env::var("EDITOR") {
                // Open at line number: e.g. `vim +8 /path/to/file`
                let line_arg = format!("+{}", line_number);
                let _ = Command::new(&editor)
                    .arg(&line_arg)
                    .arg(path)
                    .spawn();
            } else {
                // No $EDITOR set — fall back to system default
                let _ = Command::new("open").arg(path).spawn();
            }
        }
    }
}

/// Copy a path string to the macOS clipboard via pbcopy.
pub fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    if let Ok(mut child) = Command::new("pbcopy").stdin(std::process::Stdio::piped()).spawn() {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Extract the full path from any SearchResult variant.
pub fn result_path(result: &SearchResult) -> &str {
    match result {
        SearchResult::File { path, .. }
        | SearchResult::Folder { path, .. }
        | SearchResult::ContentMatch { path, .. } => path.as_str(),
    }
}
