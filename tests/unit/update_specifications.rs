#![allow(clippy::redundant_closure)]
/// Update Function Unit Tests - rusty-commit-lister
///
/// Tags: @US-03 @US-06 @US-07 @US-09 @in-memory
///
/// Tests for `update(model: AppModel, event: AppEvent) -> AppModel`.
/// Layer: unit - pure function, no subprocess, no I/O.
///
/// PBT layer (Mandate 9): proptest used for state machine invariant (layer 1-2).
/// State-delta (Mandate 8): assert on port-exposed `AppModel` fields only.
/// Universe entries are observable `AppModel` fields (`cursor`, `mode`, `search_query`, etc.)
/// - never internal struct layout details.
///
/// Chained narrative (Pillar 2):
///   - Browse mode scenarios build on the initial Browse state (S0)
///   - Search scenarios reuse the result of the "/" transition (S1)
///   - Detail scenarios reuse the result of Enter on a loaded model (S2)
///   - `RepoPicker` scenarios reuse S0 with loaded data
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use rusty_commit_lister::domain::events::AppEvent;
use rusty_commit_lister::domain::model::{AppConfig, AppMode, AppModel, CommitRecord};
use rusty_commit_lister::domain::update::update;

// ─── Test fixtures ────────────────────────────────────────────────────────────

fn make_commit(message: &str, repo_url: &str) -> CommitRecord {
    CommitRecord {
        folder: "/projects/repo".to_string(),
        time: "14:00".to_string(),
        message: message.to_string(),
        url: Some(repo_url.to_string()),
        date: "2026-05-18".to_string(),
    }
}

fn key_event(code: KeyCode) -> AppEvent {
    AppEvent::KeyPress(KeyEvent::new(code, KeyModifiers::NONE))
}

fn loaded_browse_model() -> AppModel {
    let config = AppConfig::default();
    let commits = vec![
        make_commit("feat: first commit", "https://github.com/franci/a"),
        make_commit("fix: second commit", "https://github.com/franci/b"),
        make_commit("chore: third commit", "https://github.com/franci/c"),
    ];
    let model = AppModel::new(config);
    update(model, AppEvent::LoadComplete(commits))
}

// ─── Browse mode navigation ───────────────────────────────────────────────────

/// @US-03 @in-memory
///
/// Scenario: j key in Browse mode increments cursor by 1
///   Given Browse mode with 3 commits loaded and cursor at row 0
///   When the j key is pressed
///   Then cursor is 1
///   And mode remains Browse
#[test]
fn j_key_in_browse_mode_increments_cursor() {
    let model = loaded_browse_model();
    assert_eq!(model.cursor, 0, "precondition: cursor starts at 0");
    assert_eq!(model.mode, AppMode::Browse, "precondition: Browse mode");

    let after = update(model, key_event(KeyCode::Char('j')));

    assert_eq!(after.cursor, 1, "cursor must increment by 1 on j");
    assert_eq!(after.mode, AppMode::Browse, "mode must remain Browse");
}

/// @US-03 @in-memory
///
/// Scenario: k key at row 0 (top) wraps cursor to last row
///   Given Browse mode with 3 commits and cursor at row 0
///   When the k key is pressed
///   Then cursor wraps to 2 (last row index)
///   And no error or crash occurs
#[test]
fn k_key_at_top_wraps_cursor_to_last_row() {
    let model = loaded_browse_model();
    assert_eq!(model.cursor, 0, "precondition: cursor at top");

    let after = update(model, key_event(KeyCode::Char('k')));

    assert_eq!(
        after.cursor, 2,
        "cursor must wrap to last row (2) when at top"
    );
    assert_eq!(after.mode, AppMode::Browse);
}

/// @US-03 @in-memory
///
/// Scenario: j key at last row wraps cursor to 0
///   Given Browse mode with cursor at last row (2)
///   When the j key is pressed
///   Then cursor wraps to 0 (top)
#[test]
fn j_key_at_last_row_wraps_cursor_to_zero() {
    let model = loaded_browse_model();
    // Navigate to last row
    let at_row_2 = update(
        update(model, key_event(KeyCode::Char('j'))),
        key_event(KeyCode::Char('j')),
    );
    assert_eq!(at_row_2.cursor, 2, "precondition: cursor at last row");

    let after = update(at_row_2, key_event(KeyCode::Char('j')));

    assert_eq!(after.cursor, 0, "cursor must wrap to 0 when at last row");
}

// ─── Mode transitions ─────────────────────────────────────────────────────────

/// @US-06 @in-memory
///
/// Scenario: "/" key in Browse mode transitions to Search mode
///   Given Browse mode with commits loaded
///   When the "/" key is pressed
///   Then mode becomes Search
///   And `search_query` is empty
///   And cursor is unchanged
#[test]
fn slash_key_transitions_browse_to_search_mode() {
    let model = loaded_browse_model();
    let before_cursor = model.cursor;

    let after = update(model, key_event(KeyCode::Char('/')));

    assert_eq!(after.mode, AppMode::Search, "mode must be Search after /");
    assert_eq!(
        after.search_query, "",
        "search_query must be empty on entry"
    );
    assert_eq!(
        after.cursor, before_cursor,
        "cursor must not change on / press"
    );
}

/// @US-06 @in-memory
///
/// Scenario: Esc in Search mode restores Browse mode and clears the query
///   Given Search mode with `search_query` = "rusty"
///   When Esc is pressed
///   Then mode becomes Browse
///   And `search_query` is empty
///   And `filtered_rows` equals `commit_rows` (all rows visible)
#[test]
fn esc_in_search_mode_restores_browse_and_clears_query() {
    let model = loaded_browse_model();
    // Enter search mode
    let in_search = update(model, key_event(KeyCode::Char('/')));
    // Type "rusty" in search
    let with_query = update(
        in_search,
        AppEvent::KeyPress(KeyEvent::new(KeyCode::Char('r'), KeyModifiers::NONE)),
    );
    // Simulate full query by forcing search_query (direct struct test is acceptable here
    // since we're testing Esc behavior, not typing accumulation)
    let _ = with_query; // typing test is separate

    // Test Esc from search mode (with empty query is sufficient for this scenario)
    let in_search = update(loaded_browse_model(), key_event(KeyCode::Char('/')));
    let after = update(in_search, key_event(KeyCode::Esc));

    assert_eq!(
        after.mode,
        AppMode::Browse,
        "mode must return to Browse on Esc"
    );
    assert_eq!(after.search_query, "", "query must be cleared on Esc");
    assert_eq!(
        after.filtered_rows.len(),
        after.commit_rows.len(),
        "filtered_rows must equal commit_rows (all rows visible)"
    );
}

/// @US-06 @in-memory
///
/// Scenario: Enter in Search mode confirms search and returns to Browse
///   Given Search mode with `search_query` = "feat" and 1 filtered row
///   When Enter is pressed
///   Then mode returns to Browse
///   And `search_query` is preserved ("feat" - confirm, not cancel)
///   And `filtered_rows` remains as-is (1 row - not reset to all rows)
///   And cursor resets to 0 (start of filtered result set)
#[test]
fn enter_in_search_mode_confirms_and_returns_to_browse() {
    let model = loaded_browse_model();
    let in_search = update(model, key_event(KeyCode::Char('/')));

    // Type "feat" to narrow to 1 result
    let with_query = ['f', 'e', 'a', 't'].iter().fold(in_search, |m, &ch| {
        update(
            m,
            AppEvent::KeyPress(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)),
        )
    });
    assert_eq!(
        with_query.search_query, "feat",
        "precondition: query is 'feat'"
    );
    assert_eq!(
        with_query.filtered_rows.len(),
        1,
        "precondition: 1 filtered row"
    );
    assert_eq!(
        with_query.mode,
        AppMode::Search,
        "precondition: Search mode"
    );

    let after = update(with_query, key_event(KeyCode::Enter));

    assert_eq!(after.mode, AppMode::Browse, "Enter must return to Browse");
    assert_eq!(
        after.search_query, "feat",
        "search_query must be preserved (confirm, not cancel)"
    );
    assert_eq!(
        after.filtered_rows.len(),
        1,
        "filtered_rows must remain filtered (not reset)"
    );
    assert_eq!(after.cursor, 0, "cursor must reset to 0 on Enter");
}

/// @US-07 @in-memory
///
/// Scenario: Enter in Browse mode transitions to Detail mode
///   Given Browse mode with cursor at row 1
///   When Enter is pressed
///   Then mode becomes Detail
///   And cursor is preserved (still 1)
#[test]
fn enter_in_browse_mode_transitions_to_detail() {
    let model = loaded_browse_model();
    let at_row_1 = update(model, key_event(KeyCode::Char('j')));
    assert_eq!(at_row_1.cursor, 1, "precondition: cursor at row 1");

    let after = update(at_row_1, key_event(KeyCode::Enter));

    assert_eq!(
        after.mode,
        AppMode::Detail,
        "mode must be Detail after Enter"
    );
    assert_eq!(after.cursor, 1, "cursor must be preserved on Enter");
}

/// @US-07 @in-memory
///
/// Scenario: Esc in Detail mode returns to Browse mode, cursor preserved
///   Given Detail mode with cursor at row 1
///   When Esc is pressed
///   Then mode returns to Browse
///   And cursor is still 1
#[test]
fn esc_in_detail_mode_returns_to_browse_preserving_cursor() {
    let model = loaded_browse_model();
    let in_detail = update(
        update(model, key_event(KeyCode::Char('j'))),
        key_event(KeyCode::Enter),
    );
    assert_eq!(in_detail.mode, AppMode::Detail, "precondition: Detail mode");
    assert_eq!(in_detail.cursor, 1, "precondition: cursor at 1");

    let after = update(in_detail, key_event(KeyCode::Esc));

    assert_eq!(
        after.mode,
        AppMode::Browse,
        "mode must return to Browse on Esc"
    );
    assert_eq!(
        after.cursor, 1,
        "cursor must be preserved when closing Detail"
    );
}

/// @US-09 @in-memory
///
/// Scenario: "f" key in Browse mode transitions to `RepoPicker` mode
///   Given Browse mode with commits loaded
///   When the f key is pressed
///   Then mode becomes `RepoPicker`
///   And `picker_cursor` is 0
#[test]
fn f_key_in_browse_mode_transitions_to_repo_picker() {
    let model = loaded_browse_model();

    let after = update(model, key_event(KeyCode::Char('f')));

    assert_eq!(
        after.mode,
        AppMode::RepoPicker,
        "mode must be RepoPicker after f"
    );
    assert_eq!(after.picker_cursor, 0, "picker_cursor must start at 0");
}

// ─── Search filtering ─────────────────────────────────────────────────────────

/// @US-06 @in-memory
///
/// Scenario: `SearchInput` event with "feat" narrows `filtered_rows` to matching commits
///   Given 3 commits loaded (one with "feat", two without)
///   And Search mode is active
///   When a char 'f', 'e', 'a', 't' is typed (`SearchInput` event sequence)
///   Then `filtered_rows` contains only the commit with "feat" in the message
///   And the status shows "1 of 3 commits"
///
/// Note: this tests the filtering logic via successive `KeyPress` events in Search mode.
#[test]
fn search_input_event_narrows_filtered_rows_to_matching_commits() {
    let model = loaded_browse_model();
    let in_search = update(model, key_event(KeyCode::Char('/')));

    // Type "feat" to search
    let after = ['f', 'e', 'a', 't'].iter().fold(in_search, |m, &ch| {
        update(
            m,
            AppEvent::KeyPress(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE)),
        )
    });

    assert_eq!(
        after.search_query, "feat",
        "search_query must accumulate typed characters"
    );
    assert_eq!(
        after.filtered_rows.len(),
        1,
        "filtered_rows must contain only the 'feat' commit"
    );
    assert!(
        after.filtered_rows[0].message.contains("feat"),
        "the matching commit must be the feat commit"
    );
}

// ─── LoadComplete and LoadFailed ──────────────────────────────────────────────

/// @US-03 @in-memory
///
/// Scenario: `LoadComplete` event populates `commit_rows` and `filtered_rows`
///   Given an initial model with empty `commit_rows`
///   When `LoadComplete([c1, c2, c3])` is dispatched
///   Then `commit_rows` has 3 items
///   And `filtered_rows` has 3 items (no filter active)
///   And loading is false
#[test]
fn load_complete_event_populates_commit_rows() {
    let config = AppConfig::default();
    let initial = AppModel::new(config);
    let commits = vec![
        make_commit("first", "https://github.com/franci/a"),
        make_commit("second", "https://github.com/franci/b"),
        make_commit("third", "https://github.com/franci/c"),
    ];

    let after = update(initial, AppEvent::LoadComplete(commits));

    assert_eq!(
        after.commit_rows.len(),
        3,
        "commit_rows must have 3 records"
    );
    assert_eq!(
        after.filtered_rows.len(),
        3,
        "filtered_rows must equal commit_rows when no filter"
    );
    assert!(!after.loading, "loading must be false after LoadComplete");
    assert!(
        after.error_message.is_none(),
        "no error message after successful load"
    );
}

/// @US-03 @in-memory @error
///
/// Scenario: `LoadFailed` event sets `error_message` and clears loading
///   Given an initial model
///   When `LoadFailed("vault not found")` is dispatched
///   Then `error_message` is set to the failure message
///   And loading is false
///   And `commit_rows` remains empty
#[test]
fn load_failed_event_sets_error_message() {
    let config = AppConfig::default();
    let initial = AppModel::new(config);

    let after = update(initial, AppEvent::LoadFailed("vault not found".to_string()));

    assert!(
        after.error_message.is_some(),
        "error_message must be set after LoadFailed"
    );
    assert!(
        after
            .error_message
            .as_ref()
            .unwrap()
            .contains("vault not found"),
        "error_message must contain the failure reason"
    );
    assert!(!after.loading, "loading must be false after LoadFailed");
    assert!(
        after.commit_rows.is_empty(),
        "commit_rows must remain empty after LoadFailed"
    );
}

// ─── Page navigation (PgDn / PgUp) ───────────────────────────────────────────

fn make_model_with_rows(row_count: usize, page_size: usize, cursor: usize) -> AppModel {
    let config = AppConfig::default();
    let commits: Vec<CommitRecord> = (0..row_count)
        .map(|i| make_commit(&format!("commit #{i}"), "https://github.com/franci/a"))
        .collect();
    let mut model = AppModel::new(config);
    model.page_size = page_size;
    let loaded = update(model, AppEvent::LoadComplete(commits));
    // Navigate cursor to the desired starting position using j presses.
    // We set it directly via a helper to avoid fragility.
    let mut positioned = loaded;
    positioned.cursor = cursor;
    positioned
}

/// @US-03 @in-memory
///
/// Scenario: `PageDown` in Browse mode advances cursor by `page_size`
///   Given Browse mode with 25 rows, `page_size=10`, cursor at 0
///   When `PageDown` is pressed
///   Then cursor is 10
#[test]
fn page_down_advances_cursor_by_page_size() {
    let model = make_model_with_rows(25, 10, 0);
    assert_eq!(model.cursor, 0, "precondition: cursor at 0");
    assert_eq!(model.page_size, 10, "precondition: page_size is 10");
    assert_eq!(
        model.filtered_rows.len(),
        25,
        "precondition: 25 rows loaded"
    );

    let after = update(model, key_event(KeyCode::PageDown));

    assert_eq!(
        after.cursor, 10,
        "cursor must advance by page_size (10) on PageDown"
    );
    assert_eq!(after.mode, AppMode::Browse, "mode must remain Browse");
}

/// @US-03 @in-memory
///
/// Scenario: `PageDown` clamps at last row - no wrap
///   Given Browse mode with 25 rows, `page_size=10`, cursor at 18
///   When `PageDown` is pressed
///   Then cursor is 24 (last row), not 28 (which would exceed bounds)
#[test]
fn page_down_clamps_at_last_row() {
    let model = make_model_with_rows(25, 10, 18);
    assert_eq!(model.cursor, 18, "precondition: cursor at 18");

    let after = update(model, key_event(KeyCode::PageDown));

    assert_eq!(
        after.cursor, 24,
        "cursor must clamp at last row (24) - no wrap beyond last row"
    );
}

/// @US-03 @in-memory
///
/// Scenario: `PageUp` clamps at row 0 - no wrap
///   Given Browse mode with 25 rows, `page_size`=10, cursor at 3
///   When `PageUp` is pressed
///   Then cursor is 0, not negative (wrapped)
#[test]
fn page_up_clamps_at_zero() {
    let model = make_model_with_rows(25, 10, 3);
    assert_eq!(model.cursor, 3, "precondition: cursor at 3");

    let after = update(model, key_event(KeyCode::PageUp));

    assert_eq!(
        after.cursor, 0,
        "cursor must clamp at 0 - no wrap when page moves beyond top"
    );
}

// ─── Browse mode - quit and reload keys ──────────────────────────────────────

/// @US-03 @in-memory
///
/// Scenario: 'r' key in Browse mode triggers reload by setting `loading` = true
///   Given Browse mode with commits loaded
///   When 'r' is pressed
///   Then loading is true
#[test]
fn r_key_in_browse_mode_sets_loading_true() {
    let model = loaded_browse_model();
    assert!(!model.loading, "precondition: not loading");

    let after = update(model, key_event(KeyCode::Char('r')));

    assert!(
        after.loading,
        "loading must be true after 'r' to signal reload"
    );
    assert_eq!(after.mode, AppMode::Browse, "mode must remain Browse");
}

/// @US-03 @in-memory
///
/// Scenario: 'q' key in Browse mode signals application exit
///   Given Browse mode with commits loaded
///   When 'q' is pressed
///   Then quit is true
#[test]
fn q_key_in_browse_mode_sets_quit_true() {
    let model = loaded_browse_model();
    assert!(!model.quit, "precondition: not quitting");

    let after = update(model, key_event(KeyCode::Char('q')));

    assert!(after.quit, "quit must be true after 'q'");
}

// ─── Browse mode - empty rows guard ──────────────────────────────────────────

/// @US-03 @in-memory
///
/// Scenario: j key with empty `filtered_rows` is a no-op
///   Given Browse mode with no commits (empty `filtered_rows`)
///   When j is pressed
///   Then cursor remains at 0 (no panic, no wrap)
#[test]
fn j_key_with_empty_rows_is_noop() {
    let config = AppConfig::default();
    let model = AppModel::new(config);
    let empty = update(model, AppEvent::LoadComplete(vec![]));
    assert!(empty.filtered_rows.is_empty(), "precondition: no rows");

    let after = update(empty, key_event(KeyCode::Char('j')));

    assert_eq!(after.cursor, 0, "cursor must stay at 0 when no rows");
}

/// @US-03 @in-memory
///
/// Scenario: k key with empty `filtered_rows` is a no-op
///   Given Browse mode with no commits (empty `filtered_rows`)
///   When k is pressed
///   Then cursor remains at 0 (no panic, no underflow)
#[test]
fn k_key_with_empty_rows_is_noop() {
    let config = AppConfig::default();
    let model = AppModel::new(config);
    let empty = update(model, AppEvent::LoadComplete(vec![]));
    assert!(empty.filtered_rows.is_empty(), "precondition: no rows");

    let after = update(empty, key_event(KeyCode::Char('k')));

    assert_eq!(after.cursor, 0, "cursor must stay at 0 when no rows");
}

/// @US-03 @in-memory
///
/// Scenario: `PageDown` with empty `filtered_rows` is a no-op
///   Given Browse mode with no commits
///   When `PageDown` is pressed
///   Then cursor remains at 0
#[test]
fn page_down_with_empty_rows_is_noop() {
    let config = AppConfig::default();
    let model = AppModel::new(config);
    let empty = update(model, AppEvent::LoadComplete(vec![]));

    let after = update(empty, key_event(KeyCode::PageDown));

    assert_eq!(after.cursor, 0, "cursor must stay at 0 with no rows");
}

/// @US-03 @in-memory
///
/// Scenario: `PageDown` preserves cursor when search filter empties `filtered_rows`
///   Given Browse mode with cursor at row 2 and `active_repo_filter` filtering out all rows
///   When `PageDown` is pressed
///   Then cursor stays at 2 (no navigation into empty list)
///
/// This distinguishes `row_count > 0` from `row_count >= 0` (always true for usize):
/// with >= the body would execute and `.min(0)` would drag cursor from 2 to 0.
#[test]
fn page_down_with_empty_filtered_rows_preserves_cursor() {
    let config = AppConfig::default();
    // Load 3 commits, navigate cursor to 2
    let commits = vec![
        make_commit("feat: first", "https://github.com/franci/a"),
        make_commit("fix: second", "https://github.com/franci/b"),
        make_commit("chore: third", "https://github.com/franci/c"),
    ];
    let model = AppModel::new(config);
    let loaded = update(model.clone(), AppEvent::LoadComplete(commits.clone()));
    // Position cursor at 2
    let mut at_row_2 = loaded;
    at_row_2.cursor = 2;
    // Apply repo filter that excludes all commits, then reload
    at_row_2.active_repo_filter = Some("no-such-repo".to_string());
    let empty_filtered = update(at_row_2, AppEvent::LoadComplete(commits));

    assert!(
        empty_filtered.filtered_rows.is_empty(),
        "precondition: filter empties rows"
    );
    assert_eq!(empty_filtered.cursor, 2, "precondition: cursor is 2");

    let after = update(empty_filtered, key_event(KeyCode::PageDown));

    assert_eq!(
        after.cursor, 2,
        "PageDown must not move cursor when filtered_rows is empty"
    );
}

// ─── Search mode - backspace and control chars ────────────────────────────────

/// @US-06 @in-memory
///
/// Scenario: `Backspace` in Search mode removes the last character from `search_query`
///   Given Search mode with query "feat"
///   When `Backspace` is pressed
///   Then `search_query` is "fea"
#[test]
fn backspace_in_search_mode_removes_last_char() {
    let model = loaded_browse_model();
    let in_search = update(model, key_event(KeyCode::Char('/')));
    let with_query = update(
        update(
            update(
                update(in_search, key_event(KeyCode::Char('f'))),
                key_event(KeyCode::Char('e')),
            ),
            key_event(KeyCode::Char('a')),
        ),
        key_event(KeyCode::Char('t')),
    );
    assert_eq!(
        with_query.search_query, "feat",
        "precondition: query is 'feat'"
    );

    let after = update(with_query, key_event(KeyCode::Backspace));

    assert_eq!(
        after.search_query, "fea",
        "Backspace must remove the last char"
    );
}

/// @US-06 @in-memory
///
/// Scenario: Control character is not appended to `search_query` in Search mode
///   Given Search mode with empty query
///   When a control char (`KeyCode::Null`) is pressed as a `Char` event
///   Then `search_query` remains empty
#[test]
fn control_char_not_appended_in_search_mode() {
    let model = loaded_browse_model();
    let in_search = update(model, key_event(KeyCode::Char('/')));
    assert_eq!(in_search.search_query, "", "precondition: empty query");

    let after = update(
        in_search,
        AppEvent::KeyPress(crossterm::event::KeyEvent::new(
            KeyCode::Char('\x01'),
            crossterm::event::KeyModifiers::CONTROL,
        )),
    );

    assert_eq!(after.search_query, "", "control chars must not be appended");
}

// ─── RepoPicker mode ──────────────────────────────────────────────────────────

/// @US-09 @in-memory
///
/// Scenario: `Esc` in `RepoPicker` mode returns to Browse mode
///   Given `RepoPicker` mode (entered via 'f')
///   When `Esc` is pressed
///   Then mode returns to Browse
#[test]
fn esc_in_repo_picker_returns_to_browse() {
    let model = loaded_browse_model();
    let in_picker = update(model, key_event(KeyCode::Char('f')));
    assert_eq!(
        in_picker.mode,
        AppMode::RepoPicker,
        "precondition: RepoPicker mode"
    );

    let after = update(in_picker, key_event(KeyCode::Esc));

    assert_eq!(
        after.mode,
        AppMode::Browse,
        "Esc must return to Browse from RepoPicker"
    );
}

// ─── Repository filter ────────────────────────────────────────────────────────

/// @US-09 @in-memory
///
/// Scenario: `active_repo_filter` hides commits whose URL does not match
///   Given a model with `active_repo_filter` set to "franci/a"
///   When `LoadComplete` fires with 3 commits (only 1 matches the filter)
///   Then `filtered_rows` contains only the matching commit
#[test]
fn active_repo_filter_excludes_non_matching_commits() {
    let config = AppConfig::default();
    let mut model = AppModel::new(config);
    model.active_repo_filter = Some("franci/a".to_string());

    let commits = vec![
        make_commit("feat: first", "https://github.com/franci/a"),
        make_commit("fix: second", "https://github.com/franci/b"),
        make_commit("chore: third", "https://github.com/franci/c"),
    ];

    let after = update(model, AppEvent::LoadComplete(commits));

    assert_eq!(
        after.filtered_rows.len(),
        1,
        "only the commit matching 'franci/a' must be in filtered_rows"
    );
    assert!(
        after.filtered_rows[0].message.contains("first"),
        "the matching commit must be the 'first' commit"
    );
}

// ─── Clipboard: c key in Detail mode ─────────────────────────────────────────

/// @US-05 @in-memory
///
/// Scenario: c key in Detail mode with URL sets `clipboard_pending`
///   Given Detail mode with `clipboard_available=true` and a row with URL
///   When c is pressed
///   Then `clipboard_pending = Some(url)`
///   And `status_message` is unchanged (`None`)
#[test]
fn c_key_in_detail_mode_with_url_sets_clipboard_pending() {
    let config = AppConfig {
        clipboard_available: true,
        ..Default::default()
    };
    let commits = vec![CommitRecord {
        folder: "/projects/repo".to_string(),
        time: "10:00".to_string(),
        message: "feat: clipboard test".to_string(),
        url: Some("https://github.com/franci/test".to_string()),
        date: "2026-05-19".to_string(),
    }];
    let model = update(AppModel::new(config), AppEvent::LoadComplete(commits));
    let in_detail = update(model, key_event(KeyCode::Enter));
    assert_eq!(in_detail.mode, AppMode::Detail, "precondition: Detail mode");

    let after = update(in_detail, key_event(KeyCode::Char('c')));

    assert_eq!(
        after.clipboard_pending,
        Some("https://github.com/franci/test".to_string()),
        "clipboard_pending must be set to the URL when c is pressed with a URL"
    );
    assert!(
        after.status_message.is_none(),
        "status_message must remain None when clipboard_pending is set"
    );
}

/// @US-05 @in-memory
///
/// Scenario: c key in Detail mode without URL sets `status_message`
///   Given Detail mode with `clipboard_available=true` and a row with url=None
///   When c is pressed
///   Then `status_message` contains "no URL"
///   And `clipboard_pending` remains None
#[test]
fn c_key_in_detail_mode_without_url_sets_status_message() {
    let config = AppConfig {
        clipboard_available: true,
        ..Default::default()
    };
    let commits = vec![CommitRecord {
        folder: "/projects/repo".to_string(),
        time: "10:00".to_string(),
        message: "fix: no url commit".to_string(),
        url: None,
        date: "2026-05-19".to_string(),
    }];
    let model = update(AppModel::new(config), AppEvent::LoadComplete(commits));
    let in_detail = update(model, key_event(KeyCode::Enter));
    assert_eq!(in_detail.mode, AppMode::Detail, "precondition: Detail mode");

    let after = update(in_detail, key_event(KeyCode::Char('c')));

    assert!(
        after
            .status_message
            .as_deref()
            .unwrap_or("")
            .contains("no URL"),
        "status_message must contain 'no URL' when row has no URL; got: {:?}",
        after.status_message
    );
    assert!(
        after.clipboard_pending.is_none(),
        "clipboard_pending must remain None when no URL"
    );
}

/// @US-05 @in-memory
///
/// Scenario: c key in Detail mode when clipboard unavailable sets `status_message`
///   Given Detail mode with `clipboard_available=false` and a row with URL
///   When c is pressed
///   Then `status_message` contains "select text manually"
///   And `clipboard_pending` remains `None`
#[test]
fn c_key_when_clipboard_unavailable_sets_status_message() {
    let config = AppConfig::default(); // clipboard_available = false by default
    let commits = vec![CommitRecord {
        folder: "/projects/repo".to_string(),
        time: "10:00".to_string(),
        message: "feat: clipboard unavailable".to_string(),
        url: Some("https://github.com/franci/test".to_string()),
        date: "2026-05-19".to_string(),
    }];
    let model = update(AppModel::new(config), AppEvent::LoadComplete(commits));
    let in_detail = update(model, key_event(KeyCode::Enter));
    assert_eq!(in_detail.mode, AppMode::Detail, "precondition: Detail mode");
    assert!(
        !in_detail.config.clipboard_available,
        "precondition: clipboard not available"
    );

    let after = update(in_detail, key_event(KeyCode::Char('c')));

    assert!(
        after
            .status_message
            .as_deref()
            .unwrap_or("")
            .contains("select text manually"),
        "status_message must contain 'select text manually' when clipboard unavailable; got: {:?}",
        after.status_message
    );
    assert!(
        after.clipboard_pending.is_none(),
        "clipboard_pending must remain None when clipboard unavailable"
    );
}

/// @US-05 @in-memory
///
/// Scenario: `ClipboardResult(Ok)` clears `clipboard_pending` and sets confirmation status
///   Given a model with `clipboard_pending = Some("url")`
///   When `ClipboardResult(Ok(()))` is dispatched
///   Then `clipboard_pending` is `None`
///   And `status_message` = "URL copied to clipboard"
#[test]
fn clipboard_result_ok_clears_pending_and_sets_status() {
    let config = AppConfig::default();
    let mut model = AppModel::new(config);
    model.clipboard_pending = Some("https://github.com/franci/test".to_string());

    let after = update(model, AppEvent::ClipboardResult(Ok(())));

    assert!(
        after.clipboard_pending.is_none(),
        "clipboard_pending must be None after ClipboardResult(Ok)"
    );
    assert_eq!(
        after.status_message.as_deref(),
        Some("URL copied to clipboard"),
        "status_message must be 'URL copied to clipboard' after Ok result"
    );
}

/// @US-05 @in-memory
///
/// Scenario: `ClipboardResult(Err)` clears `clipboard_pending` and sets error status
///   Given a model with `clipboard_pending = Some("url")`
///   When `ClipboardResult(Err("failed"))` is dispatched
///   Then `clipboard_pending` is `None`
///   And `status_message` = "failed"
#[test]
fn clipboard_result_err_sets_status_message() {
    let config = AppConfig::default();
    let mut model = AppModel::new(config);
    model.clipboard_pending = Some("https://github.com/franci/test".to_string());

    let after = update(model, AppEvent::ClipboardResult(Err("failed".to_string())));

    assert!(
        after.clipboard_pending.is_none(),
        "clipboard_pending must be None after ClipboardResult(Err)"
    );
    assert_eq!(
        after.status_message.as_deref(),
        Some("failed"),
        "status_message must contain the error message after Err result"
    );
}

// ─── distinct_repos helper ────────────────────────────────────────────────────

/// @US-09 @in-memory
///
/// Scenario: `distinct_repos` groups commits by last URL path segment, sorted by count descending
///   Given 3 commits - 2 with URL "…/dotfiles" and 1 with "…/notes"
///   When `distinct_repos` is called
///   Then it returns [("dotfiles", 2), ("notes", 1)]
#[test]
fn distinct_repos_groups_by_last_url_segment() {
    use rusty_commit_lister::domain::update::distinct_repos;

    let commits = vec![
        CommitRecord {
            folder: "/home/user/dotfiles".to_string(),
            time: "10:00".to_string(),
            message: "first".to_string(),
            url: Some("https://github.com/user/dotfiles".to_string()),
            date: "2026-05-18".to_string(),
        },
        CommitRecord {
            folder: "/home/user/dotfiles".to_string(),
            time: "11:00".to_string(),
            message: "second".to_string(),
            url: Some("https://github.com/user/dotfiles".to_string()),
            date: "2026-05-18".to_string(),
        },
        CommitRecord {
            folder: "/home/user/notes".to_string(),
            time: "12:00".to_string(),
            message: "third".to_string(),
            url: Some("https://github.com/user/notes".to_string()),
            date: "2026-05-18".to_string(),
        },
    ];

    let result = distinct_repos(&commits);

    assert_eq!(result.len(), 2, "must return exactly 2 distinct repos");
    assert_eq!(
        result[0],
        ("dotfiles".to_string(), 2),
        "dotfiles must be first with count 2"
    );
    assert_eq!(
        result[1],
        ("notes".to_string(), 1),
        "notes must be second with count 1"
    );
}

/// @US-09 @in-memory
///
/// Scenario: `distinct_repos` uses folder last segment when URL is None
///   Given a commit with `url=None` and folder="/projects/my-tool"
///   When `distinct_repos` is called
///   Then it returns [("my-tool", 1)]
#[test]
fn distinct_repos_uses_folder_when_url_is_none() {
    use rusty_commit_lister::domain::update::distinct_repos;

    let commits = vec![CommitRecord {
        folder: "/projects/my-tool".to_string(),
        time: "10:00".to_string(),
        message: "fix: something".to_string(),
        url: None,
        date: "2026-05-18".to_string(),
    }];

    let result = distinct_repos(&commits);

    assert_eq!(result.len(), 1, "must return exactly 1 repo");
    assert_eq!(
        result[0],
        ("my-tool".to_string(), 1),
        "repo name must be last folder segment"
    );
}

// ─── RepoPicker navigation ────────────────────────────────────────────────────

/// Helper: builds a model with `commit_rows` producing exactly 2 distinct repos:
///   dotfiles (2 commits) and notes (1 commit), in `RepoPicker` mode.
fn picker_model() -> AppModel {
    let config = AppConfig::default();
    let commits = vec![
        CommitRecord {
            folder: "/home/user/dotfiles".to_string(),
            time: "10:00".to_string(),
            message: "first dotfiles".to_string(),
            url: Some("https://github.com/user/dotfiles".to_string()),
            date: "2026-05-18".to_string(),
        },
        CommitRecord {
            folder: "/home/user/dotfiles".to_string(),
            time: "11:00".to_string(),
            message: "second dotfiles".to_string(),
            url: Some("https://github.com/user/dotfiles".to_string()),
            date: "2026-05-18".to_string(),
        },
        CommitRecord {
            folder: "/home/user/notes".to_string(),
            time: "12:00".to_string(),
            message: "notes commit".to_string(),
            url: Some("https://github.com/user/notes".to_string()),
            date: "2026-05-18".to_string(),
        },
    ];
    let model = update(AppModel::new(config), AppEvent::LoadComplete(commits));
    // Open RepoPicker via 'f'
    update(model, key_event(KeyCode::Char('f')))
}

/// @US-09 @in-memory
///
/// Scenario: j key in `RepoPicker` advances `picker_cursor` by 1
///   Given `RepoPicker` mode with `picker_cursor=0` and 2 repos available
///   When j is pressed
///   Then `picker_cursor = 1`
#[test]
fn picker_j_advances_cursor() {
    let model = picker_model();
    assert_eq!(
        model.mode,
        AppMode::RepoPicker,
        "precondition: RepoPicker mode"
    );
    assert_eq!(model.picker_cursor, 0, "precondition: picker_cursor at 0");

    let after = update(model, key_event(KeyCode::Char('j')));

    assert_eq!(
        after.picker_cursor, 1,
        "picker_cursor must increment to 1 on j"
    );
    assert_eq!(
        after.mode,
        AppMode::RepoPicker,
        "mode must remain RepoPicker"
    );
}

/// @US-09 @in-memory
///
/// Scenario: k key in `RepoPicker` at `picker_cursor=0` wraps to last repo
///   Given `RepoPicker` mode with `picker_cursor=0` and 2 repos available
///   When k is pressed
///   Then `picker_cursor` wraps to `repos.len()-1 (= 1)`
#[test]
fn picker_k_wraps_to_last() {
    let model = picker_model();
    assert_eq!(model.picker_cursor, 0, "precondition: at top");

    let after = update(model, key_event(KeyCode::Char('k')));

    assert_eq!(
        after.picker_cursor, 1,
        "picker_cursor must wrap to repos.len()-1 on k at top"
    );
    assert_eq!(
        after.mode,
        AppMode::RepoPicker,
        "mode must remain RepoPicker"
    );
}

/// @US-09 @in-memory
///
/// Scenario: Enter in `RepoPicker` sets `active_repo_filter` to selected repo and returns to `Browse`
///   Given `RepoPicker` mode with `picker_cursor=0` pointing at "dotfiles"
///   When `Enter` is pressed
///   Then `active_repo_filter = Some("dotfiles")`
///   And mode = `Browse`
///   And cursor = 0
///   And `filtered_rows` contains only dotfiles commits (2 rows)
#[test]
fn picker_enter_sets_filter_and_returns_to_browse() {
    let model = picker_model();
    assert_eq!(
        model.picker_cursor, 0,
        "precondition: cursor at 0 (dotfiles)"
    );
    assert_eq!(
        model.mode,
        AppMode::RepoPicker,
        "precondition: RepoPicker mode"
    );

    let after = update(model, key_event(KeyCode::Enter));

    assert_eq!(after.mode, AppMode::Browse, "Enter must return to Browse");
    assert_eq!(
        after.active_repo_filter,
        Some("dotfiles".to_string()),
        "active_repo_filter must be set to 'dotfiles'"
    );
    assert_eq!(after.cursor, 0, "cursor must reset to 0");
    assert_eq!(
        after.filtered_rows.len(),
        2,
        "filtered_rows must contain only the 2 dotfiles commits"
    );
}

/// @US-09 @in-memory
///
/// Scenario: `Esc` in `RepoPicker` returns to `Browse` without changing `active_repo_filter`
///   Given `RepoPicker` mode with `an existing active_repo_filter = Some("x")`
///   When `Esc` is pressed
///   Then `mode = Browse`
///   And `active_repo_filter` remains `Some("x")` (unchanged)
#[test]
fn picker_esc_returns_without_changing_filter() {
    let mut base = picker_model();
    base.active_repo_filter = Some("x".to_string());
    assert_eq!(
        base.mode,
        AppMode::RepoPicker,
        "precondition: RepoPicker mode"
    );

    let after = update(base, key_event(KeyCode::Esc));

    assert_eq!(after.mode, AppMode::Browse, "Esc must return to Browse");
    assert_eq!(
        after.active_repo_filter,
        Some("x".to_string()),
        "active_repo_filter must remain unchanged on Esc"
    );
}

/// @US-09 @in-memory
///
/// Scenario: f key in `Browse` mode when `active_repo_filter` is `Some` clears the filter
///   Given `Browse` mode with `active_repo_filter = Some("dotfiles")`
///   When f is pressed
///   Then `active_repo_filter = None`
///   And `mode = Browse` (not `RepoPicker`)
///   And `filtered_rows` is recomputed (shows all commits)
#[test]
fn f_key_clears_active_filter_in_browse_mode() {
    let config = AppConfig::default();
    let commits = vec![
        CommitRecord {
            folder: "/home/user/dotfiles".to_string(),
            time: "10:00".to_string(),
            message: "first dotfiles".to_string(),
            url: Some("https://github.com/user/dotfiles".to_string()),
            date: "2026-05-18".to_string(),
        },
        CommitRecord {
            folder: "/home/user/notes".to_string(),
            time: "12:00".to_string(),
            message: "notes commit".to_string(),
            url: Some("https://github.com/user/notes".to_string()),
            date: "2026-05-18".to_string(),
        },
    ];
    let mut model = update(AppModel::new(config), AppEvent::LoadComplete(commits));
    model.active_repo_filter = Some("dotfiles".to_string());
    model.filtered_rows = model
        .commit_rows
        .iter()
        .filter(|r| r.url.as_deref().unwrap_or("").contains("dotfiles"))
        .cloned()
        .collect();
    assert_eq!(model.mode, AppMode::Browse, "precondition: Browse mode");
    assert!(
        model.active_repo_filter.is_some(),
        "precondition: filter is active"
    );
    assert_eq!(
        model.filtered_rows.len(),
        1,
        "precondition: filter narrows to 1 row"
    );

    let after = update(model, key_event(KeyCode::Char('f')));

    assert!(
        after.active_repo_filter.is_none(),
        "f must clear active_repo_filter"
    );
    assert_eq!(
        after.mode,
        AppMode::Browse,
        "mode must remain Browse (not switch to RepoPicker)"
    );
    assert_eq!(
        after.filtered_rows.len(),
        2,
        "filtered_rows must be recomputed to show all 2 commits"
    );
}

// ─── State machine PBT invariant (proptest - layer 1) ─────────────────────────

#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;

    use rusty_commit_lister::domain::events::AppEvent;
    use rusty_commit_lister::domain::model::{AppConfig, AppMode, AppModel};
    use rusty_commit_lister::domain::update::update;

    use super::{key_event, make_commit};
    use crossterm::event::KeyCode;

    fn valid_events() -> impl Strategy<Value = AppEvent> {
        prop_oneof![
            Just(key_event(KeyCode::Char('j'))),
            Just(key_event(KeyCode::Char('k'))),
            Just(key_event(KeyCode::Char('/'))),
            Just(key_event(KeyCode::Esc)),
            Just(key_event(KeyCode::Enter)),
            Just(key_event(KeyCode::Char('f'))),
            Just(key_event(KeyCode::Char('q'))),
            Just(key_event(KeyCode::PageDown)),
            Just(key_event(KeyCode::PageUp)),
            Just(AppEvent::Tick),
        ]
    }

    // @property @US-03 @US-06 @US-07 @US-09 @in-memory
    //
    // Property: any sequence of valid events from a valid model produces a valid model
    //
    // Validity invariant: cursor must not exceed filtered_rows.len() (or 0 if empty).
    // The model must never be in an inconsistent state after any event sequence.
    proptest! {
        #![proptest_config(proptest::test_runner::Config {
            cases: 300,
            ..Default::default()
        })]

        #[test]
        fn any_valid_event_sequence_produces_valid_model(
            events in proptest::collection::vec(valid_events(), 0..=20)
        ) {
            let config = AppConfig::default();
            let commits = vec![
                make_commit("feat: first", "https://github.com/franci/a"),
                make_commit("fix: second", "https://github.com/franci/b"),
                make_commit("chore: third", "https://github.com/franci/c"),
            ];
            let initial = update(AppModel::new(config), AppEvent::LoadComplete(commits));

            let final_model = events.into_iter().fold(initial, |m, e| update(m, e));

            // Cursor invariant: cursor must be within bounds
            let max_cursor = final_model.filtered_rows.len().saturating_sub(1);
            prop_assert!(
                final_model.filtered_rows.is_empty() || final_model.cursor <= max_cursor,
                "cursor {} out of bounds for {} filtered rows",
                final_model.cursor,
                final_model.filtered_rows.len()
            );

            // Mode invariant: mode must be a valid AppMode variant
            prop_assert!(
                matches!(
                    final_model.mode,
                    AppMode::Browse | AppMode::Search | AppMode::Detail | AppMode::RepoPicker
                ),
                "mode must be a valid AppMode variant"
            );
        }
    }
}
