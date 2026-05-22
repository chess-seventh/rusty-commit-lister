#![allow(clippy::collapsible_match)]
use crossterm::event::{KeyCode, KeyEvent};

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
/// ```
/// // [("dotfiles", 2), ("notes", 1)]
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
/// - Browse + j → cursor += 1 (wraps at bottom)
/// - Browse + k → cursor -= 1 (wraps at top)
/// - Browse + `/` → mode = Search, `search_query` = ""
/// - Browse + Enter → mode = Detail
/// - Browse + `f` → mode = `RepoPicker`
/// - Browse + `r` → loading = true (triggers re-scan in event loop)
/// - Browse + `q` / Esc → signal to quit (returns model with quit flag)
/// - Search + char → `search_query` += char, `filtered_rows` recalculated
/// - Search + Esc → mode = Browse, `search_query` = "", `filtered_rows` = `commit_rows`
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

fn record_matches_filters(
    record: &CommitRecord,
    search_query: &str,
    active_repo_filter: Option<&String>,
) -> bool {
    repo_filter_matches(record, active_repo_filter.cloned().as_ref())
        && search_query_matches(record, search_query)
}

fn repo_filter_matches(record: &CommitRecord, active_repo_filter: Option<&String>) -> bool {
    active_repo_filter.as_ref().map_or(true, |filter| {
        record
            .url
            .as_deref()
            .unwrap_or("")
            .contains(filter.as_str())
    })
}

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

fn handle_key(model: AppModel, key: KeyEvent) -> AppModel {
    match model.mode.clone() {
        AppMode::Browse => handle_browse_key(model, key),
        AppMode::Search => handle_search_key(model, key),
        AppMode::Detail => handle_detail_key(model, key),
        AppMode::RepoPicker => handle_repo_picker_key(model, key),
    }
}

fn handle_browse_key(mut model: AppModel, key: KeyEvent) -> AppModel {
    let row_count = model.filtered_rows.len();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if row_count > 0 {
                model.cursor = (model.cursor + 1) % row_count;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
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
        KeyCode::Char('/') => {
            model.mode = AppMode::Search;
            model.search_query = String::new();
        }
        KeyCode::Enter => {
            if !model.filtered_rows.is_empty() {
                model.mode = AppMode::Detail;
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
        KeyCode::Char('r') => {
            model.loading = true;
        }
        KeyCode::Char('q' | 'Q') | KeyCode::Esc => {
            model.quit = true;
        }
        _ => {}
    }
    model
}

fn handle_search_key(mut model: AppModel, key: KeyEvent) -> AppModel {
    match key.code {
        KeyCode::Esc => {
            model.mode = AppMode::Browse;
            model.search_query = String::new();
            model.filtered_rows = model.commit_rows.clone();
        }
        KeyCode::Enter => {
            model.mode = AppMode::Browse;
            model.cursor = 0;
        }
        KeyCode::Backspace => {
            model.search_query.pop();
            model.filtered_rows = recompute_filtered(&model);
        }
        KeyCode::Char(c) if !c.is_control() => {
            model.search_query.push(c);
            model.filtered_rows = recompute_filtered(&model);
        }
        _ => {}
    }
    model
}

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
