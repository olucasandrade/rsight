use std::fs;
use std::path::Path;
use std::sync::mpsc;
use serde_json::Value;
use crate::types::{SearchResult, AiSource};

/// Walk `~/.codex/sessions/` and emit one `AiConversation` per JSONL file
/// whose contents contain the query string (case-insensitive).
pub fn search_codex_conversations(query: &str, tx: mpsc::Sender<SearchResult>) {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let sessions_dir = format!("{}/.codex/sessions", home);
    let base = Path::new(&sessions_dir);
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
    if !content.to_lowercase().contains(&query_lower) {
        return;
    }

    let path_str = path.to_string_lossy().to_string();
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let mut session_id = stem.clone();
    let mut title = stem.clone();
    let mut date = String::new();
    let mut found_session = false;
    let mut found_title = false;

    for line in content.lines() {
        let v: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let type_val = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

        // Extract session metadata from the first line
        if type_val == "session_meta" && !found_session {
            if let Some(payload) = v.get("payload") {
                if let Some(id) = payload.get("id").and_then(|i| i.as_str()) {
                    session_id = id.to_string();
                }
                if let Some(ts) = payload.get("timestamp").and_then(|t| t.as_str()) {
                    date = parse_iso_date(ts);
                }
            }
            found_session = true;
        }

        // Find the first real user message for the title.
        // Codex injects system context as role="user" lines starting with '#' or '<';
        // skip those and take the first plain user request.
        if type_val == "response_item" && !found_title {
            let payload = match v.get("payload") {
                Some(p) => p,
                None => continue,
            };
            if payload.get("role").and_then(|r| r.as_str()) != Some("user") {
                continue;
            }
            let content_arr = match payload.get("content").and_then(|c| c.as_array()) {
                Some(a) => a,
                None => continue,
            };
            for block in content_arr {
                let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if !matches!(block_type, "input_text" | "text") {
                    continue;
                }
                let text = block.get("text").and_then(|t| t.as_str()).unwrap_or("").trim();
                // Skip system injections (AGENTS.md, permissions, etc.)
                if text.starts_with('#') || text.starts_with('<') || text.is_empty() {
                    continue;
                }
                let sanitized = text.replace('\n', " ").replace('\r', "").replace('\t', " ");
                let trimmed = sanitized.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                // Truncate to 60 chars at a char boundary
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
                break;
            }
        }

        if found_session && found_title {
            break;
        }
    }

    let result = SearchResult::AiConversation {
        path: path_str,
        conversation_id: session_id,
        title,
        date,
        source: AiSource::Codex,
    };

    let _ = tx.send(result);
}

/// Parse an ISO 8601 timestamp like "2026-03-03T20:26:27.385Z" into "Mar 3".
fn parse_iso_date(ts: &str) -> String {
    if ts.len() < 10 {
        return String::new();
    }
    let month_str = &ts[5..7];
    let day_str = &ts[8..10];

    let month_name = match month_str {
        "01" => "Jan", "02" => "Feb", "03" => "Mar", "04" => "Apr",
        "05" => "May", "06" => "Jun", "07" => "Jul", "08" => "Aug",
        "09" => "Sep", "10" => "Oct", "11" => "Nov", "12" => "Dec",
        _ => return String::new(),
    };

    let day: u32 = day_str.parse().unwrap_or(0);
    format!("{} {}", month_name, day)
}
