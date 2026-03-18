use std::fs;
use std::path::Path;
use std::sync::mpsc;
use serde_json::Value;
use crate::types::{SearchResult, AiSource};

/// Walk `~/.claude/projects/` recursively and emit one `AiConversation` result for each
/// JSONL conversation file whose contents contain the query string (case-insensitive).
pub fn search_claude_conversations(query: &str, tx: mpsc::Sender<SearchResult>) {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let projects_dir = format!("{}/.claude/projects", home);
    let base = Path::new(&projects_dir);
    if !base.exists() {
        return;
    }

    walk_dir(base, query, &tx);
}

fn walk_dir(dir: &Path, query: &str, tx: &mpsc::Sender<SearchResult>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_dir(&path, query, tx);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            process_jsonl_file(&path, query, tx);
        }
    }
}

fn process_jsonl_file(path: &Path, query: &str, tx: &mpsc::Sender<SearchResult>) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let query_lower = query.to_lowercase();

    // Case-insensitive substring match across full file text
    if !content.to_lowercase().contains(&query_lower) {
        return;
    }

    let path_str = path.to_string_lossy().to_string();

    // Filename stem as fallback conversation_id and title
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let mut session_id = stem.clone();
    let mut title = stem.clone();
    let mut date = String::new();
    let mut found_title = false;
    let mut found_session = false;

    for line in content.lines() {
        let v: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // Extract sessionId from any parsed line (use first found)
        if !found_session {
            if let Some(sid) = v.get("sessionId").and_then(|s| s.as_str()) {
                session_id = sid.to_string();
                found_session = true;
            }
        }

        // Find first real human message for title
        if !found_title {
            let type_val = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let is_meta = v.get("isMeta").and_then(|m| m.as_bool()).unwrap_or(false);

            if type_val == "user" && !is_meta {
                let content_val = v.get("message").and_then(|m| m.get("content"));

                // Extract raw text: content may be a plain string or an array of blocks
                let raw_text: Option<String> = content_val.and_then(|c| {
                    // Try plain string first
                    if let Some(s) = c.as_str() {
                        return Some(s.to_string());
                    }
                    // Fall back to array of content blocks (Claude Code JSONL format)
                    if let Some(arr) = c.as_array() {
                        for block in arr {
                            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                                if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                    return Some(text.to_string());
                                }
                            }
                        }
                    }
                    None
                });

                if let Some(raw) = raw_text {
                    // Sanitize: strip newlines and control chars, then trim
                    let sanitized = raw.replace('\n', " ").replace('\r', "").replace('\t', " ");
                    let trimmed = sanitized.trim().to_string();
                    if !trimmed.starts_with('<') && !trimmed.is_empty() {
                        // Extract date from timestamp
                        if let Some(ts) = v.get("timestamp").and_then(|t| t.as_str()) {
                            date = parse_iso_date(ts);
                        }

                        // Truncate title to 60 chars (byte-safe: find char boundary)
                        let t = if trimmed.len() > 60 {
                            let mut end = 60;
                            while !trimmed.is_char_boundary(end) {
                                end -= 1;
                            }
                            trimmed[..end].to_string()
                        } else {
                            trimmed
                        };
                        title = t;
                        found_title = true;
                    }
                }
            }
        }

        if found_title && found_session {
            break;
        }
    }

    let result = SearchResult::AiConversation {
        path: path_str,
        conversation_id: session_id,
        title,
        date,
        source: AiSource::ClaudeCode,
    };

    if tx.send(result).is_err() {
        // Channel closed — stop sending
        return;
    }
}

/// Parse an ISO 8601 timestamp like "2026-03-18T18:27:38.841Z" into "Mar 18".
fn parse_iso_date(ts: &str) -> String {
    // Format: YYYY-MM-DDTHH:MM:SS...
    // Chars 5..7 = month, chars 8..10 = day
    if ts.len() < 10 {
        return String::new();
    }
    let month_str = &ts[5..7];
    let day_str = &ts[8..10];

    let month_name = match month_str {
        "01" => "Jan",
        "02" => "Feb",
        "03" => "Mar",
        "04" => "Apr",
        "05" => "May",
        "06" => "Jun",
        "07" => "Jul",
        "08" => "Aug",
        "09" => "Sep",
        "10" => "Oct",
        "11" => "Nov",
        "12" => "Dec",
        _ => return String::new(),
    };

    // Strip leading zero from day
    let day: u32 = day_str.parse().unwrap_or(0);
    format!("{} {}", month_name, day)
}
