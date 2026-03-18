use std::fs;
use std::path::Path;
use std::sync::mpsc;
use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use crate::types::{SearchResult, AiSource};

/// Walk `~/.cursor/chats/` and emit one `AiConversation` per `store.db` whose title or
/// blob data contains the query string (case-insensitive).
pub fn search_cursor_conversations(query: &str, tx: mpsc::Sender<SearchResult>) {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return,
    };
    let chats_dir = format!("{}/.cursor/chats", home);
    let base = Path::new(&chats_dir);
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
        } else if path.file_name().and_then(|n| n.to_str()) == Some("store.db") {
            if process_store_db(&path, query, tx).is_err() {
                // Channel closed — stop walking
                return;
            }
        }
    }
}

fn process_store_db(path: &Path, query: &str, tx: &mpsc::Sender<SearchResult>) -> Result<(), ()> {
    let conn = match Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY) {
        Ok(c) => c,
        Err(_) => return Ok(()), // skip silently
    };

    // Read meta row with key = '0'
    let meta_result: rusqlite::Result<String> = conn
        .query_row("SELECT value FROM meta WHERE key = '0'", [], |row| {
            row.get::<_, String>(0)
        });

    let hex_val = match meta_result {
        Ok(v) => v,
        Err(_) => return Ok(()), // unexpected schema — skip silently
    };

    // Hex-decode the meta value
    let meta_bytes = match hex_decode(&hex_val) {
        Some(b) => b,
        None => return Ok(()),
    };

    let meta_str = match std::str::from_utf8(&meta_bytes) {
        Ok(s) => s.to_string(),
        Err(_) => return Ok(()),
    };

    let meta: Value = match serde_json::from_str(&meta_str) {
        Ok(v) => v,
        Err(_) => return Ok(()),
    };

    let name = meta
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();

    let agent_id = meta
        .get("agentId")
        .and_then(|a| a.as_str())
        .unwrap_or("")
        .to_string();

    let created_at_ms = meta
        .get("createdAt")
        .and_then(|c| c.as_u64())
        .unwrap_or(0);

    let query_lower = query.to_lowercase();

    // Check if query appears in the title
    let mut matches = name.to_lowercase().contains(&query_lower);

    // If not in title, check blob data
    if !matches {
        let blob_result: rusqlite::Result<Vec<Vec<u8>>> = {
            let mut stmt = match conn.prepare("SELECT data FROM blobs") {
                Ok(s) => s,
                Err(_) => return Ok(()),
            };
            stmt.query_map([], |row| row.get::<_, Vec<u8>>(0))
                .map(|rows| rows.flatten().collect())
        };

        if let Ok(blobs) = blob_result {
            for blob in blobs {
                let text = String::from_utf8_lossy(&blob);
                if text.to_lowercase().contains(&query_lower) {
                    matches = true;
                    break;
                }
            }
        }
    }

    if !matches {
        return Ok(());
    }

    let date = format_cursor_date(created_at_ms);
    let path_str = path.to_string_lossy().to_string();

    let result = SearchResult::AiConversation {
        path: path_str,
        conversation_id: agent_id,
        title: name,
        date,
        source: AiSource::Cursor,
    };

    tx.send(result).map_err(|_| ())
}

/// Hex-decode a string like "4a6f6e" into bytes.
fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

/// Convert a Unix timestamp in milliseconds to "Mon DD" format.
/// Uses a Gregorian calendar algorithm — no chrono required.
fn format_cursor_date(ms: u64) -> String {
    let secs = ms / 1000;
    let days = secs / 86400;

    let z = days as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };

    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun",
        "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let month_str = months
        .get((m as usize).saturating_sub(1))
        .copied()
        .unwrap_or("???");

    format!("{} {}", month_str, d)
}
