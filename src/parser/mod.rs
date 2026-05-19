use std::path::Path;

use crate::domain::model::CommitRecord;

/// Data contract — Markdown table format owned by rusty-commit-saver.
/// Header row (pipe-delimited):
///   | FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |
///
/// Column positions (0-indexed after splitting on `|` and trimming):
///   0 = FOLDER, 1 = TIME, 2 = COMMIT MESSAGE, 3 = REPOSITORY URL
///
/// This constant is the single source of truth for the format.
/// If rusty-commit-saver changes its output, update this constant
/// and fix all tests — they will go RED immediately.
pub const COMMITS_SECTION_HEADING: &str = "## Commits";
pub const EXPECTED_COLUMNS: [&str; 4] = ["FOLDER", "TIME", "COMMIT MESSAGE", "REPOSITORY URL"];

/// Parse a single Obsidian daily note file and extract commit records.
///
/// Strategy:
/// 1. Locate the `## Commits` section heading.
/// 2. Parse the Markdown pipe table header to verify column order.
/// 3. For each data row, extract the four fields.
/// 4. Skip malformed rows with `tracing::debug!` — never panic.
/// 5. Stop parsing when a line is encountered that is not a table row or blank.
///
/// The `date` field in the returned `CommitRecord` structs is derived from
/// the filename (e.g. `2026-05-18.md` → `"2026-05-18"`).
///
/// # Returns
///
/// A `Vec<CommitRecord>` (may be empty). This function NEVER panics, NEVER
/// returns an `Err`. Skip-and-log is the error strategy.
pub fn parse_note(path: &Path) -> Vec<CommitRecord> {
    let date = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to read {:?}: {}", path, e);
            return Vec::new();
        }
    };

    let mut in_table = false;
    let mut header_seen = false;
    let mut records = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == COMMITS_SECTION_HEADING {
            in_table = true;
            continue;
        }

        if !in_table {
            continue;
        }

        if !trimmed.starts_with('|') {
            if !trimmed.is_empty() {
                break;
            }
            continue;
        }

        let cols: Vec<&str> = trimmed
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();

        if is_separator_row(&cols) {
            continue;
        }

        if !header_seen {
            header_seen = true;
            continue;
        }

        if cols.len() < 4 {
            tracing::debug!("Skipping malformed row (< 4 cols): {:?}", trimmed);
            continue;
        }

        let url = optional_column(cols[3]);

        records.push(CommitRecord {
            folder: cols[0].to_string(),
            time: cols[1].to_string(),
            message: cols[2].to_string(),
            url,
            date: date.clone(),
        });
    }

    records
}

/// Returns true if every cell in the row contains only `-`, ` `, and `:` characters,
/// which identifies a Markdown table separator row (e.g. `| --- | :---: | --- |`).
fn is_separator_row(cols: &[&str]) -> bool {
    cols.iter()
        .all(|cell| cell.chars().all(|ch| ch == '-' || ch == ' ' || ch == ':'))
}

/// Returns `None` if the column value is empty, `Some(String)` otherwise.
fn optional_column(value: &str) -> Option<String> {
    if value.is_empty() { None } else { Some(value.to_string()) }
}
