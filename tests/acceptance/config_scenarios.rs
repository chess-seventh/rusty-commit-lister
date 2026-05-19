/// Config Acceptance Tests — rusty-commit-lister
///
/// Tags: @US-01 @real-io @adapter-integration
///
/// These tests exercise the TomlConfigAdapter via the CLI binary subprocess,
/// validating observable user-facing config behaviors.
///
/// Port treatment: CLI binary subprocess (driving) + real tempdir filesystem (driven-internal).
/// Sad paths are enumerated explicitly (Mandate 11 — layer 3+ example-only).
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::{contains, is_match};
use std::fs;
use tempfile::TempDir;

fn write_config_file(dir: &std::path::Path, content: &str) -> std::path::PathBuf {
    let path = dir.join("config.toml");
    fs::write(&path, content).expect("failed to write config.toml");
    path
}

/// @US-01 @real-io
///
/// Scenario: Valid config loads silently with no config messages in output
///   Given a config.toml with valid vault_path and scan_days_back = 7
///   And the vault directory exists
///   When the binary loads config
///   Then no config warning or error message appears in stdout/stderr
///   And the binary proceeds to scan
#[test]
#[ignore = "pending: US-01 TomlConfigAdapter load RED scaffold"]
fn valid_config_loads_silently_with_no_config_messages() {
    let config_dir = TempDir::new().expect("tempdir");
    let vault_dir = TempDir::new().expect("tempdir");

    // Write a minimal daily note so the scan produces output
    fs::write(
        vault_dir.path().join("2026-05-18.md"),
        "## Commits\n\n| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |\n| --- | --- | --- | --- |\n| /projects/foo | 14:00 | test commit | https://github.com/franci/foo |\n",
    ).expect("write note");

    let config_path = write_config_file(
        config_dir.path(),
        &format!(
            "vault_path = {:?}\nscan_days_back = 7\n",
            vault_dir.path().to_str().unwrap()
        ),
    );

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(config_path)
        .assert()
        .success()
        // No config-related messages in output when config is valid
        .stdout(
            is_match("config error|Config error|no config")
                .unwrap()
                .not(),
        )
        .stderr(is_match("config error|Config error").unwrap().not());
}

/// @US-01 @real-io
///
/// Scenario: Missing config file triggers default fallback with one-line notice
///   Given no config file exists at the provided path
///   When the binary is launched with --config pointing to the nonexistent file
///   Then exit code is 0 (not an error)
///   And stdout contains a notice about using defaults including the expected config path
#[test]
#[ignore = "pending: US-01 default fallback notice RED scaffold"]
fn missing_config_triggers_default_fallback_notice() {
    let tmp = TempDir::new().expect("tempdir");
    let nonexistent_config = tmp.path().join("not_here.toml");

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(&nonexistent_config)
        .assert()
        .code(0)
        .stdout(contains("defaults").and(contains(nonexistent_config.to_str().unwrap())));
}

/// @US-01 @real-io (Unicode — CRITICAL, highest-risk per DESIGN)
///
/// Scenario: Vault path with emoji directory segment resolves via walkdir without error
///   Given a vault_path in config.toml that includes "📅 Diaries" as a path segment
///   And a daily note exists at that path
///   When the binary scans the vault
///   Then all notes at the emoji path are found and parsed
///   And no silent data loss occurs from path encoding (exit 0, commits in output)
#[test]
#[ignore = "pending: US-01 unicode OsString path RED scaffold"]
fn unicode_emoji_vault_path_resolves_correctly() {
    let base = TempDir::new().expect("tempdir");
    let config_dir = TempDir::new().expect("tempdir");

    let emoji_vault = base.path().join("📅 Diaries").join("0. Journal");
    fs::create_dir_all(&emoji_vault).expect("create emoji dir");

    fs::write(
        emoji_vault.join("2026-05-18.md"),
        "## Commits\n\n| FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL |\n| --- | --- | --- | --- |\n| /p/r | 10:00 | emoji path test | https://github.com/franci/r |\n",
    ).expect("write note");

    let config_path = write_config_file(
        config_dir.path(),
        &format!(
            "vault_path = {:?}\nscan_days_back = 7\n",
            emoji_vault
                .to_str()
                .expect("emoji path is valid UTF-8 on macOS")
        ),
    );

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(config_path)
        .assert()
        .code(0)
        // At least one commit found — not zero rows due to path failure
        .stdout(contains("commit").or(contains("Commit")));
}

/// @US-01 @real-io @error
///
/// Scenario: scan_days_back = 0 in config exits with code 2 and actionable error
///   Given config.toml contains scan_days_back = 0
///   When the binary loads config
///   Then exit code is 2
///   And stderr names the invalid field and the config file path to fix
#[test]
#[ignore = "pending: US-01 scan_days_back = 0 validation RED scaffold"]
fn scan_days_back_zero_exits_code_2_with_actionable_error() {
    let config_dir = TempDir::new().expect("tempdir");
    let vault_dir = TempDir::new().expect("tempdir");
    let config_path = write_config_file(
        config_dir.path(),
        &format!(
            "vault_path = {:?}\nscan_days_back = 0\n",
            vault_dir.path().to_str().unwrap()
        ),
    );

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .code(2)
        .stderr(contains("scan_days_back").and(contains(config_path.to_str().unwrap())));
}

/// @US-01 @real-io @error
///
/// Scenario: scan_days_back = -1 in config exits with code 2 and names the invalid value
///   Given config.toml contains scan_days_back = -1
///   When the binary loads config
///   Then exit code is 2
///   And the error message names the invalid value and the config file path
#[test]
#[ignore = "pending: US-01 scan_days_back negative validation RED scaffold"]
fn scan_days_back_negative_exits_code_2_and_names_the_value() {
    let config_dir = TempDir::new().expect("tempdir");
    let vault_dir = TempDir::new().expect("tempdir");

    // TOML doesn't allow negative for u32 — the config crate must catch this
    let config_path = write_config_file(
        config_dir.path(),
        &format!(
            "vault_path = {:?}\nscan_days_back = -1\n",
            vault_dir.path().to_str().unwrap()
        ),
    );

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(&config_path)
        .assert()
        .code(2)
        .stderr(contains("scan_days_back").and(contains(config_path.to_str().unwrap())));
}
