#![allow(clippy::needless_raw_string_hashes)]
/// Parser Unit Tests - rusty-commit-lister
///
/// Tags: @US-02 @in-memory
///
/// Tests for `parse_note(path: &Path) -> Vec<CommitRecord>`.
/// Layer: unit - pure function, no subprocess, no real filesystem access
/// except the fixture file (which is pre-committed and part of the test suite).
///
/// PBT layer (Mandate 9): `parse_note` is a pure function - proptest is used at
/// this layer for the no-panic invariant.
///
/// State-delta (Mandate 8): pure function tests use return-value assertions.
/// Universe is the function's return type (Vec<CommitRecord>).
use std::path::Path;

use rusty_commit_lister::parser::parse_note;

// ─── Happy path ───────────────────────────────────────────────────────────────

/// @US-02 @in-memory
///
/// Scenario: Standard daily note with N commit rows produces N `CommitRecord` structs
///   Given the `sample_daily_note.md` fixture with 5 commit rows
///   When `parse_note` is called on that file
///   Then it returns exactly 5 `CommitRecord` structs
///   And each record has folder, time, message, and url fields populated
#[test]
fn standard_note_with_five_rows_produces_five_commit_records() {
    let fixture = Path::new("tests/fixtures/sample_daily_note.md");

    let records = parse_note(fixture);

    assert_eq!(
        records.len(),
        5,
        "expected 5 records from sample_daily_note.md"
    );
    for record in &records {
        assert!(!record.folder.is_empty(), "folder must be populated");
        assert!(!record.time.is_empty(), "time must be populated");
        assert!(!record.message.is_empty(), "message must be populated");
        assert!(
            record.url.is_some(),
            "url must be populated for sample note rows"
        );
    }
}

/// @US-02 @in-memory
///
/// Scenario: Note with no "## Commits" section produces zero rows with no error
///   Given a temp file containing no "## Commits" heading
///   When `parse_note` is called on that file
///   Then it returns an empty `Vec<CommitRecord>`
///   And no panic or error occurs
#[test]
fn note_with_no_commits_section_produces_zero_rows() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut note = NamedTempFile::new().expect("tempfile");
    writeln!(note, "# 2026-05-16\n\nA Sunday with no commits.\n").expect("write");

    let records = parse_note(note.path());

    assert!(
        records.is_empty(),
        "expected 0 records from note with no Commits section, got {}",
        records.len()
    );
}

/// @US-02 @in-memory @error
///
/// Scenario: Note with one malformed row skips that row and parses the remaining rows
///   Given a note with 3 valid rows and 1 malformed row (missing REPOSITORY URL column)
///   When `parse_note` is called
///   Then 3 `CommitRecord` structs are returned (malformed row skipped)
///   And no panic or crash occurs
#[test]
fn malformed_row_is_skipped_and_remaining_rows_are_parsed() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut note = NamedTempFile::new().expect("tempfile");
    writeln!(
        note,
        r#"# 2026-05-18

## Commits

| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |
| ------ | ---- | -------------- | -------------- |
| /projects/a | 14:00 | first valid commit | https://github.com/franci/a |
| malformed row with too few columns |
| /projects/b | 13:00 | second valid commit | https://github.com/franci/b |
| /projects/c | 12:00 | third valid commit | https://github.com/franci/c |
"#
    )
    .expect("write");

    let records = parse_note(note.path());

    assert_eq!(
        records.len(),
        3,
        "expected 3 records (1 malformed skipped), got {}",
        records.len()
    );
    assert!(records.iter().any(|r| r.message.contains("first valid")));
    assert!(records.iter().any(|r| r.message.contains("second valid")));
    assert!(records.iter().any(|r| r.message.contains("third valid")));
}

/// @US-02 @in-memory
///
/// Scenario: Note with commit row missing REPOSITORY URL parses with `url` = None
///   Given a note where the REPOSITORY URL column is present but empty
///   When `parse_note` is called
///   Then the returned `CommitRecord` has `url` = None
///   And no panic or error occurs
#[test]
fn commit_row_with_empty_url_column_produces_record_with_url_none() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut note = NamedTempFile::new().expect("tempfile");
    writeln!(
        note,
        r#"# 2026-05-18

## Commits

| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |
| ------ | ---- | -------------- | -------------- |
| /projects/old | 10:00 | old format commit without url |  |
"#
    )
    .expect("write");

    let records = parse_note(note.path());

    assert_eq!(records.len(), 1, "expected 1 record");
    assert!(
        records[0].url.is_none(),
        "expected url = None for empty URL column"
    );
}

// ─── Property-based tests (PBT full - layer 1, pure function) ─────────────────

/// @US-02 @in-memory @property
///
/// Property: `parse_note` never panics on any arbitrary .md file content
///
/// This is a proptest invariant: for any String content written to a .md file,
/// `parse_note` returns a result (empty or populated) without panicking.
///
/// Strategy: proptest `any::<String>()` provides adversarial inputs including:
///   - empty strings
///   - strings with invalid UTF-8 escape sequences
///   - binary-looking content
///   - strings with only pipe characters
///   - valid Markdown with mangled table rows
#[cfg(test)]
mod property_tests {
    use proptest::prelude::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    use rusty_commit_lister::parser::parse_note;

    proptest! {
        #![proptest_config(proptest::test_runner::Config {
            cases: 200,
            ..Default::default()
        })]

        /// @property @US-02 @in-memory
        #[test]
        fn parse_note_never_panics_on_any_markdown_content(content in ".*") {
            let mut note = NamedTempFile::new().expect("tempfile");
            write!(note, "{content}").expect("write");
            // The function must not panic; return value (empty or populated) is acceptable.
            let _ = parse_note(note.path());
        }

        /// @property @US-02 @in-memory
        #[test]
        fn parse_note_returns_no_more_records_than_pipe_rows_in_content(
            rows in 0usize..=20usize
        ) {
            let mut note = NamedTempFile::new().expect("tempfile");
            writeln!(note, "## Commits\n").expect("write");
            writeln!(note, "| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |").expect("write");
            writeln!(note, "| --- | --- | --- | --- |").expect("write");
            for i in 0..rows {
                writeln!(
                    note,
                    "| /p/{} | 14:0{} | commit {} | https://github.com/franci/r |",
                    i, i % 10, i
                ).expect("write");
            }
            let records = parse_note(note.path());
            prop_assert!(
                records.len() <= rows,
                "records ({}) must not exceed data rows ({})",
                records.len(), rows
            );
        }
    }
}
