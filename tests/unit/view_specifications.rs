/// View Helper Unit Tests — rusty-commit-lister
///
/// Tags: @US-04 @in-memory
///
/// Tests for pure view helper functions extracted from src/tui/view.rs.
/// Layer: unit — pure function, no subprocess, no I/O, no ratatui Frame.
///
/// The tested surface: `detail_lines(record: &CommitRecord) -> Vec<String>`
/// This function is pub(crate) and builds the display lines for the Detail
/// overlay without touching ratatui rendering. It is the testable pure core
/// of render_detail_overlay().
///
/// # bypass: example-based tests used instead of PBT
/// The contract is an exact ordered list of formatted strings; the invariants
/// are structural (line count, field inclusion, exact format strings). A
/// property strategy would add no detection power over two precisely-crafted
/// examples that cover the two branch outcomes (url Some vs None).
use rusty_commit_lister::domain::model::CommitRecord;
use rusty_commit_lister::tui::view::detail_lines;

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
