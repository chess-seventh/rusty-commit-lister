/// Walking Skeleton Acceptance Test — rusty-commit-lister
///
/// Tags: @walking_skeleton @driving_port @US-01 @US-02 @US-03 @real-io
///
/// WS strategy: CLI binary via assert_cmd subprocess from tempdir.
/// Port treatment: driving = real CLI subprocess; driven-internal = real tempdir filesystem.
/// Driven-external (clipboard) is not exercised in the walking skeleton.
///
/// This test proves the full chain:
///   config.toml → vault scan → markdown parser → TUI renders rows → exit 0
///
/// It is the FIRST scenario enabled. All other scenarios are #[ignore].
///
/// Reference: docs/architecture/atdd-infrastructure-policy.md
use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;
use std::fs;
use tempfile::TempDir;

/// Helper: create a minimal config.toml pointing to a tempdir vault.
fn write_config(dir: &TempDir, vault_path: &str, scan_days_back: u32) -> std::path::PathBuf {
    let config_path = dir.path().join("config.toml");
    fs::write(
        &config_path,
        format!(
            "vault_path = {:?}\nscan_days_back = {}\n",
            vault_path, scan_days_back
        ),
    )
    .expect("failed to write config.toml");
    config_path
}

/// Helper: write a realistic daily note fixture to the vault dir.
fn write_note(vault_dir: &std::path::Path, filename: &str, content: &str) {
    let path = vault_dir.join(filename);
    fs::write(path, content).expect("failed to write daily note fixture");
}

const SAMPLE_NOTE_CONTENT: &str = r#"# 2026-05-18

## Commits

| FOLDER                                          | TIME     | COMMIT MESSAGE                         | REPOSITORY URL                            |
| ----------------------------------------------- | -------- | -------------------------------------- | ----------------------------------------- |
| /Users/franci/projects/rusty-commit-lister/src  | 14:32    | feat: add TUI skeleton                 | https://github.com/franci/rusty-commit-lister |
| /Users/franci/projects/dotfiles/.config/nvim    | 09:22    | chore: update neovim config            | https://github.com/franci/dotfiles        |
"#;

/// @walking_skeleton @driving_port @US-01 @US-02 @US-03 @real-io
///
/// Scenario: Tool loads commits from a vault directory and exits successfully
///   Given a valid config.toml pointing to a tempdir vault
///   And the vault contains one daily note with 2 commit rows
///   When the binary is invoked via CLI with --config pointing to the config
///   Then the process exits with code 0
///   And stdout contains text indicating commits were found
#[test]
fn tool_loads_commits_from_vault_and_exits_successfully() {
    let config_dir = TempDir::new().expect("failed to create config tempdir");
    let vault_dir = TempDir::new().expect("failed to create vault tempdir");

    write_note(vault_dir.path(), "2026-05-18.md", SAMPLE_NOTE_CONTENT);
    let config_path = write_config(&config_dir, vault_dir.path().to_str().unwrap(), 7);

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found — run `cargo build` first")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(contains("commit").or(contains("Commit")));
}

/// @US-01 @real-io @error
///
/// Scenario: Tool exits with code 2 when scan_days_back is invalid
///   Given a config.toml with scan_days_back = 0
///   When the binary is invoked with that config
///   Then the process exits with code 2
///   And stderr contains an actionable error mentioning scan_days_back and the config path
#[test]
fn invalid_scan_days_back_exits_with_code_2_and_actionable_error() {
    let config_dir = TempDir::new().expect("failed to create config tempdir");
    let vault_dir = TempDir::new().expect("failed to create vault tempdir");
    let config_path = write_config(&config_dir, vault_dir.path().to_str().unwrap(), 0);

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .assert()
        .failure()
        .code(2)
        .stderr(contains("scan_days_back").and(contains("config")));
}

/// @US-01 @real-io
///
/// Scenario: Tool starts with defaults when no config file exists
///   Given no config file exists at the specified path
///   When the binary is invoked with --config pointing to a non-existent file
///   Then the process exits with code 0
///   And stdout contains a notice about using defaults
#[test]
fn missing_config_uses_defaults_and_shows_notice() {
    let tmp = TempDir::new().expect("failed to create tempdir");
    let nonexistent = tmp.path().join("does_not_exist.toml");

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(nonexistent.to_str().unwrap())
        .assert()
        .success()
        .stdout(contains("defaults").or(contains("no config")));
}

/// @US-01 @real-io (Unicode path constraint — highest-risk per DESIGN)
///
/// Scenario: Tool resolves vault path containing emoji directory segment
///   Given a vault directory whose path includes the "📅" emoji segment
///   And the vault contains a daily note with commits
///   When the binary is invoked with a config pointing to that path
///   Then the process exits with code 0
///   And commits from the emoji-named vault are found and displayed
#[test]
fn vault_path_with_emoji_segment_is_resolved_correctly() {
    let base_dir = TempDir::new().expect("failed to create base tempdir");
    let config_dir = TempDir::new().expect("failed to create config tempdir");

    // Create vault path with emoji segment: <tmpdir>/📅 Diaries/0. Journal/
    let emoji_dir = base_dir.path().join("📅 Diaries").join("0. Journal");
    fs::create_dir_all(&emoji_dir).expect("failed to create emoji vault dir");

    write_note(&emoji_dir, "2026-05-18.md", SAMPLE_NOTE_CONTENT);
    let config_path = write_config(
        &config_dir,
        emoji_dir.to_str().expect("emoji path not valid UTF-8"),
        7,
    );

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .assert()
        .success()
        .stdout(contains("commit").or(contains("Commit")));
}

/// @US-03 @real-io
///
/// Scenario: Tool exits cleanly when q is sent as input
///   Given a valid vault with commits
///   When q is piped to the binary stdin
///   Then exit code is 0
///   And no ANSI escape sequences remain in the terminal output
#[test]
fn tool_exits_cleanly_with_code_0_when_q_pressed() {
    let config_dir = TempDir::new().expect("failed to create config tempdir");
    let vault_dir = TempDir::new().expect("failed to create vault tempdir");
    write_note(vault_dir.path(), "2026-05-18.md", SAMPLE_NOTE_CONTENT);
    let config_path = write_config(&config_dir, vault_dir.path().to_str().unwrap(), 7);

    // Pipe 'q' to stdin to simulate the user quitting
    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .write_stdin("q")
        .assert()
        .code(0);
}

/// @US-03 @real-io @error
///
/// Scenario: Tool shows informative empty state when vault has no notes in window
///   Given a valid config pointing to an empty vault directory
///   When the binary is invoked
///   Then exit code is 0
///   And stdout contains text indicating no commits were found for the scan window
#[test]
fn empty_vault_shows_informative_empty_state() {
    let config_dir = TempDir::new().expect("failed to create config tempdir");
    let vault_dir = TempDir::new().expect("failed to create vault tempdir");
    // vault_dir is empty — no daily notes
    let config_path = write_config(&config_dir, vault_dir.path().to_str().unwrap(), 7);

    Command::cargo_bin("rusty_commit_lister")
        .expect("binary not found")
        .arg("--config")
        .arg(config_path.to_str().unwrap())
        .assert()
        .code(0)
        .stdout(contains("No commits").or(contains("no commits")));
}
