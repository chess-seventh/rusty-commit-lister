/// View Unit Tests — rusty-commit-lister
///
/// Tags: @US-04 @in-memory
///
/// Tests for view.rs: pure helper functions tested directly, render functions
/// tested via ratatui TestBackend (no subprocess, no real terminal).
///
/// # bypass: example-based tests used instead of PBT
/// The contract is an exact ordered list of formatted strings; the invariants
/// are structural (line count, field inclusion, exact format strings). A
/// property strategy would add no detection power over precisely-crafted
/// examples that cover the branch outcomes (url Some vs None, modes).
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rusty_commit_lister::domain::model::{AppConfig, AppMode, AppModel, CommitRecord};
use rusty_commit_lister::tui::view::{detail_lines, view};

// ─── Test fixtures ─────────────────────────────────────────────────────────────

fn make_commit_with_url() -> CommitRecord {
    CommitRecord {
        date: "2026-05-18".to_string(),
        time: "14:30".to_string(),
        message: "feat: implement full-length commit message that must not be truncated".to_string(),
        folder: "/projects/my-very-long-folder-name/sub/dir".to_string(),
        url: Some("https://github.com/foo/bar".to_string()),
    }
}

fn make_commit_without_url() -> CommitRecord {
    CommitRecord {
        date: "2026-05-17".to_string(),
        time: "09:15".to_string(),
        message: "fix: another commit".to_string(),
        folder: "/projects/other".to_string(),
        url: None,
    }
}

// ─── detail_lines — URL present ────────────────────────────────────────────────

/// Scenario: detail_lines shows all five fields when URL is present
///   Given a CommitRecord with date, time, message, folder, and url = Some(...)
///   When detail_lines is called
///   Then the returned Vec has exactly 5 lines
///   And line 0 contains the date
///   And line 1 contains the time
///   And line 2 contains the full (un-truncated) message
///   And line 3 contains the full (un-truncated) folder
///   And line 4 contains the URL string
#[test]
fn detail_lines_shows_all_fields_with_url() {
    let record = make_commit_with_url();
    let lines = detail_lines(&record);

    assert_eq!(lines.len(), 5, "detail_lines must return exactly 5 lines");

    assert!(
        lines[0].contains("2026-05-18"),
        "line 0 must contain the date; got: {:?}",
        lines[0]
    );
    assert!(
        lines[1].contains("14:30"),
        "line 1 must contain the time; got: {:?}",
        lines[1]
    );
    assert!(
        lines[2].contains("feat: implement full-length commit message that must not be truncated"),
        "line 2 must contain the full (un-truncated) message; got: {:?}",
        lines[2]
    );
    assert!(
        lines[3].contains("/projects/my-very-long-folder-name/sub/dir"),
        "line 3 must contain the full (un-truncated) folder; got: {:?}",
        lines[3]
    );
    assert!(
        lines[4].contains("https://github.com/foo/bar"),
        "line 4 must contain the URL; got: {:?}",
        lines[4]
    );
}

// ─── detail_lines — URL absent ─────────────────────────────────────────────────

/// Scenario: detail_lines shows "— not available —" when URL is None
///   Given a CommitRecord with url = None
///   When detail_lines is called
///   Then the returned Vec has exactly 5 lines
///   And line 4 contains "— not available —"
#[test]
fn detail_lines_shows_not_available_when_url_is_none() {
    let record = make_commit_without_url();
    let lines = detail_lines(&record);

    assert_eq!(lines.len(), 5, "detail_lines must return exactly 5 lines");

    assert!(
        lines[4].contains("— not available —"),
        "line 4 must contain '— not available —' when url is None; got: {:?}",
        lines[4]
    );
}

// ─── Render helpers ────────────────────────────────────────────────────────────

fn render_to_rows(model: &AppModel) -> Vec<String> {
    let backend = TestBackend::new(120, 25);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| view(model, frame)).unwrap();
    let buf = terminal.backend().buffer().clone();
    (0..25u16)
        .map(|y| {
            (0..120u16)
                .map(|x| buf.get(x, y).symbol().to_string())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

fn joined(rows: &[String]) -> String {
    rows.join("\n")
}

fn make_browse_model_with_one_row() -> AppModel {
    let mut model = AppModel::new(AppConfig::default());
    model.loading = false;
    model.commit_rows = vec![make_commit_with_url()];
    model.filtered_rows = vec![make_commit_with_url()];
    model.mode = AppMode::Browse;
    model.cursor = 0;
    model
}

fn make_detail_model() -> AppModel {
    let mut m = make_browse_model_with_one_row();
    m.mode = AppMode::Detail;
    m
}

fn make_search_model() -> AppModel {
    let mut m = make_browse_model_with_one_row();
    m.mode = AppMode::Search;
    m.search_query = "feat".to_string();
    m
}

// ─── Render: Detail mode ────────────────────────────────────────────────────────

/// Scenario: Detail mode renders the "Commit Detail" block title
#[test]
fn view_renders_detail_overlay_title() {
    let rows = render_to_rows(&make_detail_model());
    assert!(
        joined(&rows).contains("Commit Detail"),
        "Detail mode must render block title 'Commit Detail'; got:\n{}",
        joined(&rows)
    );
}

/// Scenario: Detail mode renders commit fields without truncation
#[test]
fn view_renders_detail_fields() {
    let rows = render_to_rows(&make_detail_model());
    let out = joined(&rows);
    assert!(out.contains("2026-05-18"), "date must appear in detail view");
    assert!(
        out.contains("feat: implement full-length commit message that must not be truncated"),
        "full message must appear in detail view without truncation"
    );
    assert!(
        out.contains("/projects/my-very-long-folder-name/sub/dir"),
        "full folder must appear in detail view without truncation"
    );
    assert!(out.contains("https://github.com/foo/bar"), "URL must appear in detail view");
}

/// Scenario: Status bar shows "Esc to return" in Detail mode
#[test]
fn view_shows_esc_to_return_in_detail_mode() {
    let rows = render_to_rows(&make_detail_model());
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("Esc to return"),
        "last row must show 'Esc to return' in Detail mode; got: {:?}",
        last_row
    );
}

// ─── Render: Browse mode ───────────────────────────────────────────────────────

/// Scenario: Browse mode renders the commit table header
#[test]
fn view_renders_table_header_in_browse_mode() {
    let rows = render_to_rows(&make_browse_model_with_one_row());
    let out = joined(&rows);
    assert!(out.contains("Date"), "Browse mode must render 'Date' header");
    assert!(out.contains("Message"), "Browse mode must render 'Message' header");
}

/// Scenario: Status bar shows 1-based row/total in Browse mode
#[test]
fn view_shows_row_one_of_one_in_browse_mode() {
    let rows = render_to_rows(&make_browse_model_with_one_row());
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("Row 1/1"),
        "Browse mode status must show 'Row 1/1' for cursor=0 with 1 row; got: {:?}",
        last_row
    );
}

// ─── Render: Search mode ───────────────────────────────────────────────────────

/// Scenario: Search mode renders the search bar
#[test]
fn view_renders_search_bar_in_search_mode() {
    let rows = render_to_rows(&make_search_model());
    let out = joined(&rows);
    assert!(
        out.contains("/ feat"),
        "Search mode must render '/ feat' search bar; got:\n{}",
        out
    );
}

/// Scenario: Status bar shows N-of-M match count in Search mode
#[test]
fn view_shows_match_count_in_search_mode() {
    let rows = render_to_rows(&make_search_model());
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("commits | Esc cancel"),
        "Search mode status must contain 'commits | Esc cancel'; got: {:?}",
        last_row
    );
}
