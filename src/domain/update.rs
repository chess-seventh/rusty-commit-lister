//! The pure `update` reducer and its key-handling helpers (Elm/MVU).
#![allow(clippy::collapsible_match)]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::domain::events::AppEvent;
use crate::domain::model::{AppMode, AppModel, CommitRecord};

/// Returns the distinct repository names present in `commit_rows`, sorted by count descending.
///
/// The repo name is derived from the last path segment of the URL (e.g. `"dotfiles"` from
/// `"https://github.com/user/dotfiles"`), or the last segment of `folder` when `url` is `None`.
/// Entries with an empty derived name are ignored.
///
/// Ties in count are broken alphabetically (ascending) by repo name for deterministic ordering.
///
/// # Examples
///
/// ```text
/// distinct_repos(&rows) // => [("dotfiles", 2), ("notes", 1)]
/// ```
pub fn distinct_repos(commit_rows: &[CommitRecord]) -> Vec<(String, usize)> {
    use std::collections::HashMap;
    let mut counts: HashMap<String, usize> = HashMap::new();
    for record in commit_rows {
        let name = record
            .url
            .as_deref()
            .and_then(|u| u.rsplit('/').next())
            .map(str::to_string)
            .or_else(|| record.folder.rsplit('/').next().map(str::to_string))
            .unwrap_or_default();
        if !name.is_empty() {
            *counts.entry(name).or_insert(0) += 1;
        }
    }
    let mut pairs: Vec<(String, usize)> = counts.into_iter().collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    pairs
}

/// Pure state machine: given the current model and an event, return the next model.
///
/// This is the Update function in the Elm/MVU architecture.
/// It has zero I/O, zero side effects. Every transition produces a new `AppModel`.
///
/// # Key state machine transitions
///
/// - Browse + Down/Up → cursor moves (wraps)
/// - Browse + char → `search_query` += char, `filtered_rows` recomputed, cursor = 0
/// - Browse + Backspace → `search_query` shortened, `filtered_rows` recomputed
/// - Browse + Enter → mode = Detail
/// - Browse + Ctrl-Y/E/P/U → copy folder / message / note path / URL
/// - Browse + Ctrl-F → toggle `RepoPicker` (or clear an active repo filter)
/// - Browse + Ctrl-R → loading = true (triggers re-scan in event loop)
/// - Browse + Esc / Ctrl-C → signal to quit (returns model with quit flag)
/// - Detail + Esc → mode = Browse, cursor preserved
/// - Detail + `c` → triggers clipboard write (`ClipboardResult` event follows)
/// - `RepoPicker` + Enter → `active_repo_filter` = selected repo
/// - `RepoPicker` + Esc → mode = Browse, `active_repo_filter` unchanged
/// - `LoadComplete` → `commit_rows` set, `filtered_rows` computed, loading = false
/// - `LoadFailed` → `error_message` set, loading = false
/// - ClipboardResult(Ok) → `status_message` = "URL copied to clipboard"
/// - ClipboardResult(Err) → `status_message` = "Copy not available — select text manually"
pub fn update(mut model: AppModel, event: AppEvent) -> AppModel {
    match event {
        AppEvent::LoadComplete(records) => {
            model.commit_rows = records;
            model.loading = false;
            model.error_message = None;
            model.filtered_rows = recompute_filtered(&model);
        }
        AppEvent::LoadFailed(msg) => {
            model.error_message = Some(msg);
            model.loading = false;
        }
        AppEvent::ClipboardResult(Ok(())) => {
            model.clipboard_pending = None;
            model.status_message = Some("URL copied to clipboard".to_string());
        }
        AppEvent::ClipboardResult(Err(msg)) => {
            model.clipboard_pending = None;
            model.status_message = Some(msg);
        }
        AppEvent::Tick => {}
        AppEvent::KeyPress(key) => {
            model = handle_key(model, key);
        }
    }
    model
}

/// Recompute `filtered_rows` from `commit_rows` under the active search + repo filters.
fn recompute_filtered(model: &AppModel) -> Vec<CommitRecord> {
    model
        .commit_rows
        .iter()
        .filter(|record| {
            record_matches_filters(
                record,
                &model.search_query,
                model.active_repo_filter.clone().as_ref(),
            )
        })
        .cloned()
        .collect()
}

/// Returns true when `record` satisfies both the repo filter and the search query.
fn record_matches_filters(
    record: &CommitRecord,
    search_query: &str,
    active_repo_filter: Option<&String>,
) -> bool {
    repo_filter_matches(record, active_repo_filter.cloned().as_ref())
        && search_query_matches(record, search_query)
}

/// Returns true when no repo filter is active, or the record's URL contains it.
fn repo_filter_matches(record: &CommitRecord, active_repo_filter: Option<&String>) -> bool {
    active_repo_filter.as_ref().map_or(true, |filter| {
        record
            .url
            .as_deref()
            .unwrap_or("")
            .contains(filter.as_str())
    })
}

/// Returns true when the query is empty, or matches the record's message or URL
/// (case-insensitive substring).
fn search_query_matches(record: &CommitRecord, search_query: &str) -> bool {
    if search_query.is_empty() {
        return true;
    }
    let query = search_query.to_lowercase();
    record.message.to_lowercase().contains(&query)
        || record
            .url
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains(&query)
}

/// Dispatch a key event to the handler for the model's current `AppMode`.
fn handle_key(model: AppModel, key: KeyEvent) -> AppModel {
    match model.mode.clone() {
        AppMode::Browse => handle_browse_key(model, key),
        AppMode::Detail => handle_detail_key(model, key),
        AppMode::RepoPicker => handle_repo_picker_key(model, key),
    }
}

/// Queue `text` for the clipboard when it is available, otherwise surface a
/// graceful status message (US-08: copy never panics, degrades to a hint).
fn queue_copy(model: &mut AppModel, text: String) {
    if model.config.clipboard_available {
        model.clipboard_pending = Some(text);
        model.status_message = None;
    } else {
        model.status_message = Some("Copy not available — select text manually".to_string());
    }
}

/// Browse-mode keys (fzf-style): any printable char filters live; arrows and
/// page keys navigate; Ctrl-modified keys copy/reload/open the picker; Esc and
/// Ctrl-C quit. Nothing here mutates data — this is a read-only browser.
fn handle_browse_key(model: AppModel, key: KeyEvent) -> AppModel {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return handle_browse_command(model, key);
    }

    let mut model = model;
    let row_count = model.filtered_rows.len();
    match key.code {
        KeyCode::Down => {
            if row_count > 0 {
                model.cursor = (model.cursor + 1) % row_count;
            }
        }
        KeyCode::Up => {
            if row_count > 0 {
                model.cursor = model.cursor.checked_sub(1).unwrap_or(row_count - 1);
            }
        }
        KeyCode::PageDown => {
            if row_count > 0 {
                model.cursor = (model.cursor + model.page_size).min(row_count.saturating_sub(1));
            }
        }
        KeyCode::PageUp => {
            model.cursor = model.cursor.saturating_sub(model.page_size);
        }
        KeyCode::Enter => {
            if !model.filtered_rows.is_empty() {
                model.mode = AppMode::Detail;
            }
        }
        KeyCode::Esc => {
            model.quit = true;
        }
        KeyCode::Backspace => {
            model.search_query.pop();
            model = apply_filter_change(model);
        }
        KeyCode::Char(c) if !c.is_control() => {
            model.search_query.push(c);
            model = apply_filter_change(model);
        }
        _ => {}
    }
    model
}

/// Ctrl-modified Browse commands: copy the selected fields, reload, open the
/// repo picker, or quit. Copies degrade gracefully when the clipboard is absent.
fn handle_browse_command(mut model: AppModel, key: KeyEvent) -> AppModel {
    let selected = model.filtered_rows.get(model.cursor).cloned();
    match key.code {
        KeyCode::Char('c') => model.quit = true,
        KeyCode::Char('y') => {
            if let Some(r) = selected {
                queue_copy(&mut model, r.folder);
            }
        }
        KeyCode::Char('e') => {
            if let Some(r) = selected {
                queue_copy(&mut model, r.message);
            }
        }
        KeyCode::Char('p') => {
            if let Some(r) = selected {
                queue_copy(&mut model, r.note_path);
            }
        }
        KeyCode::Char('u') => {
            if let Some(r) = selected {
                match r.url {
                    Some(url) => queue_copy(&mut model, url),
                    None => model.status_message = Some("Copy not available — no URL".to_string()),
                }
            }
        }
        KeyCode::Char('f') => {
            if model.active_repo_filter.is_some() {
                model.active_repo_filter = None;
                model.filtered_rows = recompute_filtered(&model);
            } else {
                model.mode = AppMode::RepoPicker;
                model.picker_cursor = 0;
            }
        }
        KeyCode::Char('r') => model.loading = true,
        _ => {}
    }
    model
}

/// Recompute `filtered_rows` after a filter-query change and clamp the cursor to
/// the top of the new result set (fzf semantics: selection jumps to first match).
fn apply_filter_change(mut model: AppModel) -> AppModel {
    model.filtered_rows = recompute_filtered(&model);
    model.cursor = 0;
    model
}

/// Detail-mode keys: copy the URL to the clipboard, or return to Browse.
fn handle_detail_key(mut model: AppModel, key: KeyEvent) -> AppModel {
    match key.code {
        KeyCode::Esc => {
            model.mode = AppMode::Browse;
        }
        KeyCode::Char('c') => {
            if !model.filtered_rows.is_empty() {
                let url = model.filtered_rows[model.cursor].url.clone();
                if let Some(url_str) = url {
                    if model.config.clipboard_available {
                        model.clipboard_pending = Some(url_str);
                    } else {
                        model.status_message =
                            Some("Copy not available — select text manually".to_string());
                    }
                } else {
                    model.status_message = Some("Copy not available — no URL".to_string());
                }
            }
        }
        _ => {}
    }
    model
}

/// Repo-picker keys: move the picker cursor, confirm a repo filter, or cancel.
fn handle_repo_picker_key(mut model: AppModel, key: KeyEvent) -> AppModel {
    let repos = distinct_repos(&model.commit_rows);
    let len = repos.len();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if len > 0 {
                model.picker_cursor = (model.picker_cursor + 1) % len;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if len > 0 {
                model.picker_cursor = model.picker_cursor.checked_sub(1).unwrap_or(len - 1);
            }
        }
        KeyCode::Enter => {
            if len > 0 {
                model.active_repo_filter = Some(repos[model.picker_cursor].0.clone());
                model.filtered_rows = recompute_filtered(&model);
                model.mode = AppMode::Browse;
                model.cursor = 0;
            }
        }
        KeyCode::Esc => {
            model.mode = AppMode::Browse;
        }
        _ => {}
    }
    model
}
