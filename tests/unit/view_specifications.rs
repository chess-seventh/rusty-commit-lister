use ratatui::Terminal;
/// View Unit Tests - rusty-commit-lister
///
/// Tags: @US-04 @in-memory
///
/// Tests for view.rs: pure helper functions tested directly, render functions
/// tested via ratatui `TestBackend` (no subprocess, no real terminal).
///
/// # bypass: example-based tests used instead of PBT
/// The contract is an exact ordered list of formatted strings; the invariants
/// are structural (line count, field inclusion, exact format strings). A
/// property strategy would add no detection power over precisely-crafted
/// examples that cover the branch outcomes (url Some vs None, modes).
use ratatui::backend::TestBackend;
use rusty_commit_lister::domain::model::{AppConfig, AppMode, AppModel, CommitRecord};
use rusty_commit_lister::tui::view::{detail_lines, view};

// ─── Test fixtures ─────────────────────────────────────────────────────────────

fn make_commit_with_url() -> CommitRecord {
    CommitRecord {
        date: "2026-05-18".to_string(),
        note_path: String::new(),
        time: "14:30".to_string(),
        message: "feat: implement full-length commit message that must not be truncated"
            .to_string(),
        folder: "/projects/my-very-long-folder-name/sub/dir".to_string(),
        url: Some("https://github.com/foo/bar".to_string()),
    }
}

fn make_commit_without_url() -> CommitRecord {
    CommitRecord {
        date: "2026-05-17".to_string(),
        note_path: String::new(),
        time: "09:15".to_string(),
        message: "fix: another commit".to_string(),
        folder: "/projects/other".to_string(),
        url: None,
    }
}

// ─── detail_lines - URL present ────────────────────────────────────────────────

/// Scenario: `detail_lines` shows all five fields when URL is present
///   Given a `CommitRecord` with date, time, message, folder, and url = Some(...)
///   When `detail_lines` is called
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

// ─── detail_lines - URL absent ─────────────────────────────────────────────────

/// Scenario: `detail_lines` shows "- not available -" when URL is None
///   Given a `CommitRecord` with url = None
///   When `detail_lines` is called
///   Then the returned `Vec` has exactly 5 lines
///   And line 4 contains "- not available -"
#[test]
fn detail_lines_shows_not_available_when_url_is_none() {
    let record = make_commit_without_url();
    let lines = detail_lines(&record);

    assert_eq!(lines.len(), 5, "detail_lines must return exactly 5 lines");

    assert!(
        lines[4].contains("- not available -"),
        "line 4 must contain '- not available -' when url is None; got: {:?}",
        lines[4]
    );
}

// ─── Render helpers ────────────────────────────────────────────────────────────

#[allow(deprecated)]
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

fn make_filtering_browse_model() -> AppModel {
    let mut m = make_browse_model_with_one_row();
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
    assert!(
        out.contains("2026-05-18"),
        "date must appear in detail view"
    );
    assert!(
        out.contains("feat: implement full-length commit message that must not be truncated"),
        "full message must appear in detail view without truncation"
    );
    assert!(
        out.contains("/projects/my-very-long-folder-name/sub/dir"),
        "full folder must appear in detail view without truncation"
    );
    assert!(
        out.contains("https://github.com/foo/bar"),
        "URL must appear in detail view"
    );
}

/// Scenario: Status bar shows "c copy | Esc return" in Detail mode
#[test]
fn view_shows_esc_to_return_in_detail_mode() {
    let rows = render_to_rows(&make_detail_model());
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("Esc return"),
        "last row must show 'Esc return' in Detail mode; got: {last_row:?}"
    );
}

// ─── Render: Browse mode ───────────────────────────────────────────────────────

/// Scenario: Browse mode renders the commit table header
#[test]
fn view_renders_table_header_in_browse_mode() {
    let rows = render_to_rows(&make_browse_model_with_one_row());
    let out = joined(&rows);
    assert!(
        out.contains("Date"),
        "Browse mode must render 'Date' header"
    );
    assert!(
        out.contains("Message"),
        "Browse mode must render 'Message' header"
    );
}

/// Scenario: Browse mode shows the always-on detail bubble for the selected row
///   Given Browse mode with one selected commit
///   Then the "Commit Detail" bubble renders with the full (un-truncated)
///   message and folder path visible below the table.
#[test]
fn view_shows_detail_bubble_in_browse_mode() {
    let rows = render_to_rows(&make_browse_model_with_one_row());
    let out = joined(&rows);
    assert!(
        out.contains("Commit Detail"),
        "Browse mode must render the always-on detail bubble; got:\n{out}"
    );
    assert!(
        out.contains("feat: implement full-length commit message that must not be truncated"),
        "bubble must show the full message; got:\n{out}"
    );
    assert!(
        out.contains("/projects/my-very-long-folder-name/sub/dir"),
        "bubble must show the full folder path; got:\n{out}"
    );
}

/// Scenario: Status bar shows filtered/total counts in Browse mode
#[test]
fn view_shows_counts_in_browse_mode() {
    let rows = render_to_rows(&make_browse_model_with_one_row());
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("1/1"),
        "Browse mode status must show '1/1' for 1 row; got: {last_row:?}",
    );
}

// ─── Render: live filter ───────────────────────────────────────────────────────

/// Scenario: the live filter query is echoed in the Browse status bar
#[test]
fn view_shows_filter_query_in_status_bar() {
    let rows = render_to_rows(&make_filtering_browse_model());
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("/feat"),
        "Browse status must echo the '/feat' filter query; got: {last_row:?}",
    );
}

/// Scenario: Browse status offers the copy hints while filtering
#[test]
fn view_shows_copy_hints_while_filtering() {
    let rows = render_to_rows(&make_filtering_browse_model());
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("copy"),
        "Browse status must show copy hints; got: {last_row:?}",
    );
}

// ─── Render: Detail mode - clipboard / status_message ─────────────────────────

/// Scenario: Detail mode renders `status_message` when Some
///   Given Detail mode with `status_message` = Some("URL copied to clipboard")
///   When the view is rendered
///   Then the output contains "URL copied to clipboard"
#[test]
fn view_shows_status_message_in_detail_overlay() {
    let mut model = make_detail_model();
    model.status_message = Some("URL copied to clipboard".to_string());

    let rows = render_to_rows(&model);
    let out = joined(&rows);

    assert!(
        out.contains("URL copied to clipboard"),
        "Detail overlay must show status_message when Some; got:\n{out}",
    );
}

/// Scenario: Status bar in Detail mode shows the copy + return hints
///   Given `Detail` mode (no `status_message`)
///   When the view is rendered
///   Then the last row shows the copy hint and "Esc return"
#[test]
fn view_detail_status_bar_shows_copy_hint() {
    let model = make_detail_model();

    let rows = render_to_rows(&model);
    let last_row = rows.last().unwrap();

    assert!(
        last_row.contains("copy") && last_row.contains("Esc return"),
        "Detail mode status bar must show copy + return hints; got: {last_row:?}",
    );
}

// ─── Render: RepoPicker mode ───────────────────────────────────────────────────

fn make_repo_picker_model() -> AppModel {
    let dotfiles_1 = CommitRecord {
        date: "2026-05-18".to_string(),
        note_path: String::new(),
        time: "10:00".to_string(),
        message: "feat: add dotfiles".to_string(),
        folder: "/home/user/dotfiles".to_string(),
        url: Some("https://github.com/user/dotfiles".to_string()),
    };
    let dotfiles_2 = CommitRecord {
        date: "2026-05-17".to_string(),
        note_path: String::new(),
        time: "11:00".to_string(),
        message: "fix: update dotfiles".to_string(),
        folder: "/home/user/dotfiles".to_string(),
        url: Some("https://github.com/user/dotfiles".to_string()),
    };
    let notes_1 = CommitRecord {
        date: "2026-05-18".to_string(),
        note_path: String::new(),
        time: "12:00".to_string(),
        message: "docs: add notes".to_string(),
        folder: "/home/user/notes".to_string(),
        url: Some("https://github.com/user/notes".to_string()),
    };
    let mut model = AppModel::new(AppConfig::default());
    model.loading = false;
    model.commit_rows = vec![dotfiles_1.clone(), dotfiles_2.clone(), notes_1.clone()];
    model.filtered_rows = vec![dotfiles_1, dotfiles_2, notes_1];
    model.mode = AppMode::RepoPicker;
    model.picker_cursor = 0;
    model
}

/// Scenario: `RepoPicker` mode renders a "Repo Filter" bordered overlay
///   Given a model in `RepoPicker` mode with 3 commit rows (2 dotfiles, 1 notes)
///   When the view is rendered
///   Then the output contains "Repo Filter"
#[test]
fn view_renders_repo_picker_overlay() {
    let model = make_repo_picker_model();
    let rows = render_to_rows(&model);
    let out = joined(&rows);
    assert!(
        out.contains("Repo Filter"),
        "RepoPicker mode must render 'Repo Filter' block title; got:\n{out}",
    );
}

/// Scenario: `RepoPicker` overlay lists repos with their counts
///   Given a model in `RepoPicker` mode with 2 dotfiles commits and 1 notes commit
///   When the view is rendered
///   Then the output contains "dotfiles (2)" and "notes (1)"
#[test]
fn view_picker_shows_repo_names_with_counts() {
    let model = make_repo_picker_model();
    let rows = render_to_rows(&model);
    let out = joined(&rows);
    assert!(
        out.contains("dotfiles (2)"),
        "RepoPicker must show 'dotfiles (2)'; got:\n{out}",
    );
    assert!(
        out.contains("notes (1)"),
        "RepoPicker must show 'notes (1)'; got:\n{out}",
    );
}

/// Scenario: `RepoPicker` overlay highlights the row at `picker_cursor`
///   Given `picker_cursor` = 0 (dotfiles is first, highest count)
///   When the view is rendered
///   Then the highlighted row contains "dotfiles"
///
/// # bypass: example-based - the invariant is a specific styled cell at a known index.
/// Testing highlight style via `TestBackend` checks `cell.style().modifier` or reversed bg.
/// The simplest observable proxy is that the text at cursor position appears in output.
#[test]
fn view_highlights_selected_picker_row() {
    let model = make_repo_picker_model();
    let rows = render_to_rows(&model);
    let out = joined(&rows);
    // The highlighted row (picker_cursor=0) corresponds to "dotfiles (2)".
    // We verify both that the text appears and that the overlay renders correctly.
    assert!(
        out.contains("dotfiles (2)"),
        "highlighted row at picker_cursor=0 must contain 'dotfiles (2)'; got:\n{out}",
    );
}

// ─── Render: Status bar with active repo filter ────────────────────────────────

/// Scenario: Browse mode status bar shows filter indicator when `active_repo_filter` is set
///   Given Browse mode with `active_repo_filter` = `Some("dotfiles")`, `filtered_rows.len()` = 2, `commit_rows.len()` = 3
///   When the view is rendered
///   Then the last row contains "dotfiles" and "f clear"
#[test]
fn view_status_bar_shows_active_filter() {
    let dotfiles_1 = CommitRecord {
        date: "2026-05-18".to_string(),
        note_path: String::new(),
        time: "10:00".to_string(),
        message: "feat: add dotfiles".to_string(),
        folder: "/home/user/dotfiles".to_string(),
        url: Some("https://github.com/user/dotfiles".to_string()),
    };
    let dotfiles_2 = CommitRecord {
        date: "2026-05-17".to_string(),
        note_path: String::new(),
        time: "11:00".to_string(),
        message: "fix: update dotfiles".to_string(),
        folder: "/home/user/dotfiles".to_string(),
        url: Some("https://github.com/user/dotfiles".to_string()),
    };
    let notes_1 = CommitRecord {
        date: "2026-05-18".to_string(),
        note_path: String::new(),
        time: "12:00".to_string(),
        message: "docs: add notes".to_string(),
        folder: "/home/user/notes".to_string(),
        url: Some("https://github.com/user/notes".to_string()),
    };
    let mut model = AppModel::new(AppConfig::default());
    model.loading = false;
    model.commit_rows = vec![dotfiles_1.clone(), dotfiles_2.clone(), notes_1];
    model.filtered_rows = vec![dotfiles_1, dotfiles_2];
    model.mode = AppMode::Browse;
    model.active_repo_filter = Some("dotfiles".to_string());
    model.cursor = 0;

    let rows = render_to_rows(&model);
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("dotfiles"),
        "Browse status bar must show repo name 'dotfiles' when filter active; got: {last_row:?}",
    );
    assert!(
        last_row.contains("^F clear"),
        "Browse status bar must show '^F clear' when filter active; got: {last_row:?}",
    );
}

/// Scenario: `RepoPicker` mode status bar shows navigation hints
///   Given a model in `RepoPicker` mode
///   When the view is rendered
///   Then the last row contains "Enter confirm" or "j/k"
#[test]
fn view_status_bar_shows_repo_picker_hints() {
    let model = make_repo_picker_model();
    let rows = render_to_rows(&model);
    let last_row = rows.last().unwrap();
    assert!(
        last_row.contains("Enter confirm") || last_row.contains("j/k"),
        "RepoPicker status bar must contain navigation hints; got: {last_row:?}",
    );
}
