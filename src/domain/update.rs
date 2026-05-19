use crossterm::event::{KeyCode, KeyEvent};

use crate::domain::events::AppEvent;
use crate::domain::model::{AppMode, AppModel, CommitRecord};

/// Pure state machine: given the current model and an event, return the next model.
///
/// This is the Update function in the Elm/MVU architecture.
/// It has zero I/O, zero side effects. Every transition produces a new AppModel.
///
/// # Key state machine transitions
///
/// - Browse + j → cursor += 1 (wraps at bottom)
/// - Browse + k → cursor -= 1 (wraps at top)
/// - Browse + `/` → mode = Search, search_query = ""
/// - Browse + Enter → mode = Detail
/// - Browse + `f` → mode = RepoPicker
/// - Browse + `r` → loading = true (triggers re-scan in event loop)
/// - Browse + `q` / Esc → signal to quit (returns model with quit flag)
/// - Search + char → search_query += char, filtered_rows recalculated
/// - Search + Esc → mode = Browse, search_query = "", filtered_rows = commit_rows
/// - Detail + Esc → mode = Browse, cursor preserved
/// - Detail + `c` → triggers clipboard write (ClipboardResult event follows)
/// - RepoPicker + Enter → active_repo_filter = selected repo
/// - RepoPicker + Esc → mode = Browse, active_repo_filter unchanged
/// - LoadComplete → commit_rows set, filtered_rows computed, loading = false
/// - LoadFailed → error_message set, loading = false
/// - ClipboardResult(Ok) → status_message = "URL copied to clipboard"
/// - ClipboardResult(Err) → status_message = "Copy not available — select text manually"
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
            model.status_message = Some("URL copied to clipboard".to_string());
        }
        AppEvent::ClipboardResult(Err(msg)) => {
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
        .filter(|record| record_matches_filters(record, &model.search_query, &model.active_repo_filter))
        .cloned()
        .collect()
}

fn record_matches_filters(
    record: &CommitRecord,
    search_query: &str,
    active_repo_filter: &Option<String>,
) -> bool {
    repo_filter_matches(record, active_repo_filter) && search_query_matches(record, search_query)
}

fn repo_filter_matches(record: &CommitRecord, active_repo_filter: &Option<String>) -> bool {
    active_repo_filter.as_ref().map_or(true, |filter| {
        record.url.as_deref().unwrap_or("").contains(filter.as_str())
    })
}

fn search_query_matches(record: &CommitRecord, search_query: &str) -> bool {
    if search_query.is_empty() {
        return true;
    }
    let query = search_query.to_lowercase();
    record.message.to_lowercase().contains(&query)
        || record.url.as_deref().unwrap_or("").to_lowercase().contains(&query)
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
            model.mode = AppMode::RepoPicker;
            model.picker_cursor = 0;
        }
        KeyCode::Char('r') => {
            model.loading = true;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
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
    if key.code == KeyCode::Esc {
        model.mode = AppMode::Browse;
    }
    model
}

fn handle_repo_picker_key(mut model: AppModel, key: KeyEvent) -> AppModel {
    if key.code == KeyCode::Esc {
        model.mode = AppMode::Browse;
    }
    model
}
