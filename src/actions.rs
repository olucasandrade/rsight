use std::process::Command;
use crate::types::{SearchResult, AiSource};

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
                let _ = Command::new("open").arg(path).spawn();
            }
        }
        SearchResult::AiConversation { .. } => {
            // AI conversations are handled by open_conversation() — not this function.
        }
    }
}

/// Open a conversation in its native application.
/// For Claude Code: opens a new Terminal.app window running `claude --resume <id>`.
/// For Cursor: opens Cursor via CLI, or falls back to open -a Cursor.
/// rsight stays alive — uses spawn (non-blocking).
/// Sets status_message on error (CLI not found).
pub fn open_conversation(result: &SearchResult, status_message: &mut Option<String>) {
    if let SearchResult::AiConversation { conversation_id, source, .. } = result {
        match source {
            AiSource::ClaudeCode => {
                let claude_check = Command::new("which").arg("claude").output();
                if claude_check.map(|o| o.status.success()).unwrap_or(false) {
                    let project_dir = std::env::current_dir()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_else(|_| "~".to_string());
                    // cd into the project directory before resuming so Claude Code
                    // opens with the correct workspace context
                    let script = format!(
                        "tell application \"Terminal\" to do script \"cd '{}' && claude --resume {}\"",
                        project_dir, conversation_id
                    );
                    let _ = Command::new("osascript")
                        .arg("-e")
                        .arg(&script)
                        .spawn();
                } else {
                    *status_message = Some(
                        "claude CLI not found — install at claude.ai/code".to_string()
                    );
                }
            }
            // AiSource::Cursor => { ... } // not yet supported
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
        | SearchResult::ContentMatch { path, .. }
        | SearchResult::AiConversation { path, .. } => path.as_str(),
    }
}
