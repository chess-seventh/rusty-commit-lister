/// Adapter Integration Tests - rusty-commit-lister
///
/// Tags: @real-io @adapter-integration @US-01 @US-02 @US-04 @US-05
///
/// Every driven adapter has at least one test with real I/O (Mandate 6).
/// Sad paths are enumerated explicitly (Mandate 11 - layer 3+ example-only).
/// PBT machinery is NOT imported at this layer.
///
/// Adapters covered:
///   - `TomlConfigAdapter`: reads real TOML from `tempdir`, validates fields, rejects invalid
///   - `WalkdirScanAdapter`: scans tempdir with realistic note structure, returns `CommitRecord` slice
///   - Unicode path probe: `WalkdirScanAdapter` with 📅 in path segment - `OsString` round-trip
///   - `ArboardClipboardAdapter`: compile/instantiation smoke test (headless-safe)
use std::fs;
use std::path::PathBuf;

use chrono::NaiveDate;
use tempfile::TempDir;

use rusty_commit_lister::adapters::arboard_clipboard::ArboardClipboardAdapter;
use rusty_commit_lister::adapters::toml_config::TomlConfigAdapter;
use rusty_commit_lister::adapters::walkdir_vault::{Clock, WalkdirScanAdapter};
use rusty_commit_lister::ports::config_port::ConfigPort;
use rusty_commit_lister::ports::vault_port::VaultScanPort;

/// Test clock pinned to a fixed "today" so the `scan(days_back)` window is
/// deterministic — fixture filenames use fixed dates and cannot rot.
struct FixedClock(NaiveDate);

impl Clock for FixedClock {
    fn today(&self) -> NaiveDate {
        self.0
    }
}

/// Anchor date shared by the clock-injected scan tests.
const PINNED_TODAY: NaiveDate = match NaiveDate::from_ymd_opt(2026, 5, 20) {
    Some(d) => d,
    None => unreachable!(),
};

// ─── TomlConfigAdapter tests ─────────────────────────────────────────────────

/// @US-01 @real-io @adapter-integration
///
/// Scenario: `TomlConfigAdapter` reads `vault_path` and `scan_days_back` from real TOML file
///   Given a valid `config.toml` written to a tempdir
///   When `TomlConfigAdapter::load()` is called
///   Then it returns an `AppConfig` with the correct `vault_path` and `scan_days_back`
#[test]
fn toml_config_adapter_reads_vault_path_and_scan_days_back_from_real_file() {
    let dir = TempDir::new().expect("tempdir");
    let vault_dir = TempDir::new().expect("vault tempdir");

    let config_path = dir.path().join("config.toml");
    fs::write(
        &config_path,
        format!(
            "vault_path = {:?}\nscan_days_back = 14\n",
            vault_dir.path().to_str().unwrap()
        ),
    )
    .expect("write config");

    let adapter = TomlConfigAdapter::new(config_path);
    let config = adapter.load().expect("load should succeed");

    assert_eq!(config.scan_days_back, 14);
    assert_eq!(config.vault_path, PathBuf::from(vault_dir.path()));
}

/// @US-01 @real-io @adapter-integration
///
/// Scenario: `TomlConfigAdapter` reads the optional `zebra_color` key
///   Given a config.toml with `zebra_color = "green"`
///   When `TomlConfigAdapter::load()` is called
///   Then the returned `AppConfig.zebra_color` is "green"
///   And a config without the key falls back to the default
#[test]
fn toml_config_adapter_reads_zebra_color_and_defaults_when_absent() {
    let dir = TempDir::new().expect("tempdir");
    let vault_dir = TempDir::new().expect("vault tempdir");

    let with_color = dir.path().join("with_color.toml");
    fs::write(
        &with_color,
        format!(
            "vault_path = {:?}\nscan_days_back = 7\nzebra_color = \"green\"\n",
            vault_dir.path().to_str().unwrap()
        ),
    )
    .expect("write config");
    let config = TomlConfigAdapter::new(with_color).load().expect("load");
    assert_eq!(config.zebra_color, "green");

    let without_color = dir.path().join("without_color.toml");
    fs::write(
        &without_color,
        format!(
            "vault_path = {:?}\nscan_days_back = 7\n",
            vault_dir.path().to_str().unwrap()
        ),
    )
    .expect("write config");
    let defaulted = TomlConfigAdapter::new(without_color).load().expect("load");
    assert_eq!(
        defaulted.zebra_color, "default",
        "absent zebra_color must fall back to the default"
    );
}

/// @US-01 @real-io @adapter-integration
///
/// Scenario: `TomlConfigAdapter` accepts `scan_days_back` = 0 (meaning "all")
///   Given a `config.toml` with `scan_days_back` = 0
///   When `TomlConfigAdapter::load()` is called
///   Then it returns `Ok(AppConfig)` with `scan_days_back` = 0 (no window)
#[test]
fn toml_config_adapter_accepts_scan_days_back_zero_as_all() {
    let dir = TempDir::new().expect("tempdir");
    let vault_dir = TempDir::new().expect("vault tempdir");

    let config_path = dir.path().join("config.toml");
    fs::write(
        &config_path,
        format!(
            "vault_path = {:?}\nscan_days_back = 0\n",
            vault_dir.path().to_str().unwrap()
        ),
    )
    .expect("write config");

    let adapter = TomlConfigAdapter::new(config_path);
    let config = adapter.load().expect("scan_days_back = 0 must be accepted");

    assert_eq!(
        config.scan_days_back, 0,
        "scan_days_back = 0 must load as 0 (no window)"
    );
}

/// @US-01 @real-io @adapter-integration
///
/// Scenario: `TomlConfigAdapter` applies default `scan_days_back` when config file is absent
///   Given no config file exists at the adapter's `config_path`
///   When `TomlConfigAdapter::load()` is called
///   Then it returns `Ok(AppConfig)` with `scan_days_back` = 7 (default)
#[test]
fn toml_config_adapter_returns_defaults_when_file_is_absent() {
    let dir = TempDir::new().expect("tempdir");
    let nonexistent = dir.path().join("not_here.toml");

    let adapter = TomlConfigAdapter::new(nonexistent);
    let config = adapter.load().expect("missing file should not be an error");

    assert_eq!(
        config.scan_days_back, 7,
        "default scan_days_back should be 7"
    );
}

// ─── WalkdirScanAdapter tests ─────────────────────────────────────────────────

/// @US-02 @US-04 @real-io @adapter-integration
///
/// Scenario: `WalkdirScanAdapter` finds daily notes in vault and returns `CommitRecord` slice
///   Given a vault directory containing two daily notes with commit rows
///   When `WalkdirScanAdapter::scan(days_back=7)` is called
///   Then it returns a non-empty `Vec<CommitRecord>`
///   And the records contain the expected folder and message values
#[test]
fn walkdir_scan_adapter_returns_commit_records_from_real_vault_directory() {
    let vault_dir = TempDir::new().expect("tempdir");

    // Fixed fixture filenames + an injected clock pinned to PINNED_TODAY: the
    // scan(7) window is deterministic, so these dates cannot rot regardless of
    // the real calendar.
    fs::write(
        vault_dir.path().join("2026-05-19.md"),
        "## Commits\n\n| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |\n| --- | --- | --- | --- |\n| /projects/rcl/src | 14:32 | feat: add TUI skeleton | https://github.com/franci/rcl |\n",
    ).expect("write note");

    fs::write(
        vault_dir.path().join("2026-05-18.md"),
        "## Commits\n\n| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |\n| --- | --- | --- | --- |\n| /projects/dotfiles | 09:15 | chore: update nvim | https://github.com/franci/dotfiles |\n",
    ).expect("write note");

    let adapter = WalkdirScanAdapter::with_clock(
        vault_dir.path().to_path_buf(),
        Box::new(FixedClock(PINNED_TODAY)),
    );
    let records = adapter.scan(7).expect("scan should succeed");

    assert!(
        !records.is_empty(),
        "scan should return at least one commit record"
    );
    assert!(
        records.iter().any(|r| r.message.contains("TUI skeleton")),
        "expected TUI skeleton commit in results"
    );
    assert!(
        records.iter().any(|r| r.message.contains("update nvim")),
        "expected nvim commit in results"
    );
}

/// @US-02 @US-04 @real-io @adapter-integration
///
/// Scenario: `WalkdirScanAdapter::scan(0)` ignores the date window and returns all notes
///   Given a note dated far outside any recent window (2026-01-01)
///   When `scan(0)` is called with the clock pinned to 2026-05-20
///   Then the old note is still returned (0 = no window)
#[test]
fn walkdir_scan_adapter_scan_zero_returns_all_notes_ignoring_window() {
    let vault_dir = TempDir::new().expect("tempdir");
    fs::write(
        vault_dir.path().join("2026-01-01.md"),
        "## Commits\n\n| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |\n| --- | --- | --- | --- |\n| /p/old | 08:00 | old new-year commit | https://github.com/franci/old |\n",
    )
    .expect("write note");

    let adapter = WalkdirScanAdapter::with_clock(
        vault_dir.path().to_path_buf(),
        Box::new(FixedClock(PINNED_TODAY)),
    );

    // A 7-day window from PINNED_TODAY (2026-05-20) excludes the January note...
    assert!(
        adapter.scan(7).expect("scan should succeed").is_empty(),
        "precondition: the old note is outside a 7-day window"
    );
    // ...but scan(0) means "no window" and returns it.
    let all = adapter.scan(0).expect("scan should succeed");
    assert!(
        all.iter().any(|r| r.message.contains("new-year commit")),
        "scan(0) must return notes regardless of date"
    );
}

/// @US-01 @real-io @adapter-integration (Unicode path - CRITICAL)
///
/// Scenario: `WalkdirScanAdapter` handles vault path with emoji directory segment
///   Given a vault path containing "📅 Diaries" in a segment
///   And a daily note exists at that path
///   When `WalkdirScanAdapter::scan(7)` is called
///   Then the note is found and parsed without error
///   And the `OsString` round-trip for the emoji segment succeeds (no silent data loss)
#[test]
fn walkdir_scan_adapter_handles_emoji_path_segment_without_data_loss() {
    let base = TempDir::new().expect("tempdir");
    let emoji_dir = base.path().join("📅 Diaries").join("0. Journal");
    fs::create_dir_all(&emoji_dir).expect("create emoji dir");

    // Fixed filename inside the pinned scan(7) window (see PINNED_TODAY).
    fs::write(
        emoji_dir.join("2026-05-19.md"),
        "## Commits\n\n| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |\n| --- | --- | --- | --- |\n| /p/r | 10:00 | emoji path test commit | https://github.com/franci/r |\n",
    ).expect("write note");

    // Probe: verify the OsString round-trip
    let emoji_path = emoji_dir.as_os_str();
    let roundtrip = PathBuf::from(emoji_path);
    assert_eq!(
        roundtrip.to_str().unwrap(),
        emoji_dir.to_str().unwrap(),
        "OsString round-trip for emoji path must be lossless"
    );

    let adapter =
        WalkdirScanAdapter::with_clock(emoji_dir.clone(), Box::new(FixedClock(PINNED_TODAY)));
    let records = adapter.scan(7).expect("scan should succeed");

    assert!(
        !records.is_empty(),
        "emoji vault must yield at least one record - OsString round-trip succeeded"
    );
    assert!(
        records
            .iter()
            .any(|r| r.message.contains("emoji path test commit")),
        "commit from emoji vault directory must appear in results"
    );
}

/// @US-04 @real-io @adapter-integration @error
///
/// Scenario: `WalkdirScanAdapter` returns empty result (not error) when vault has no notes in window
///   Given a vault directory with no .md files matching the scan window dates
///   When `WalkdirScanAdapter::scan(3)` is called
///   Then it returns `Ok(vec![])` - empty, not an error
#[test]
fn walkdir_scan_adapter_returns_empty_vec_when_no_notes_in_window() {
    let vault_dir = TempDir::new().expect("tempdir");
    // No files written - empty vault

    let adapter = WalkdirScanAdapter::new(vault_dir.path().to_path_buf());
    let records = adapter
        .scan(3)
        .expect("empty vault should return Ok, not error");

    assert!(records.is_empty(), "expected zero records from empty vault");
}

// ─── ArboardClipboardAdapter tests ───────────────────────────────────────────

/// @US-05 @adapter-integration
///
/// Scenario: `ArboardClipboardAdapter::new()` creates a zero-size struct without panicking
///   Given no preconditions
///   When `ArboardClipboardAdapter::new()` is called
///   Then it returns an instance (headless-safe: no clipboard ops)
///
/// This is a compile-time + instantiation smoke test. Actual clipboard I/O is
/// tested manually (arboard requires a display server - fails in CI/headless).
#[test]
fn arboard_clipboard_adapter_new_does_not_panic() {
    let _adapter = ArboardClipboardAdapter::new();
    // If this compiles and runs without panic, the struct is wired correctly.
    // ArboardClipboardAdapter is zero-size - no heap allocation, no I/O.
}

/// @US-02 @real-io @adapter-integration @error
///
/// Scenario: `WalkdirScanAdapter` silently skips daily note with no Commits section
///   Given a vault with one note that has no "## Commits" heading
///   When `WalkdirScanAdapter::scan(7)` is called
///   Then it returns `Ok(vec![])` - no error, no records
#[test]
fn walkdir_scan_adapter_skips_note_with_no_commits_section() {
    let vault_dir = TempDir::new().expect("tempdir");

    // Note dated inside the pinned scan(7) window so this genuinely exercises the
    // no-Commits-section path (not a date-filtered miss).
    fs::write(
        vault_dir.path().join("2026-05-18.md"),
        "# 2026-05-18\n\nJust a journal entry with no commits today.\n",
    )
    .expect("write note");

    let adapter = WalkdirScanAdapter::with_clock(
        vault_dir.path().to_path_buf(),
        Box::new(FixedClock(PINNED_TODAY)),
    );
    let records = adapter
        .scan(7)
        .expect("note without commits section should not error");

    assert!(
        records.is_empty(),
        "expected zero records from note with no Commits section"
    );
}
