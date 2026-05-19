# ATDD Infrastructure Policy

Per `nw-distill` § Project Infrastructure Policy. One file per project. Apply-if-exists;
write-if-absent; rewrite with `--policy=fresh`. Git history is the audit trail.

First written: DISTILL wave 2026-05-18 — rusty-commit-lister (feature: rusty-commit-lister).

## Driving

| Port | Mechanism | Note |
|---|---|---|
| CLI binary (`rusty_commit_lister`) | `assert_cmd::Command::cargo_bin("rusty_commit_lister")` from `tempfile::TempDir` | `cargo test` builds and invokes the real binary via subprocess |

## Driven internal (real)

| Port | Mechanism | Note |
|---|---|---|
| Filesystem — config (`~/.config/rusty-commit-lister/config.toml`) | `tempfile::TempDir` per test; write real TOML to temp path | Fresh dir per test; no shared state |
| Filesystem — vault (`📅 Diaries` path) | `tempfile::TempDir` with subdirectory named with emoji segment | Exercises `OsString` round-trip with real `walkdir` traversal |

## Driven external / non-deterministic (fake)

| Port | Fake | Note |
|---|---|---|
| System clipboard (`ClipboardPort`) | `FakeClipboard` struct implementing `ClipboardPort` trait | `write()` captures to `Vec<String>`; `probe()` always returns `Ok(())` |
