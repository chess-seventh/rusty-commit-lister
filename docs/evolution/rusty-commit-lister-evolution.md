# Evolution Log — rusty-commit-lister

## Slice-01: Walking Skeleton (2026-05-19)

**Shipped**: Full end-to-end walking skeleton — config load → vault scan → markdown parse → ratatui TUI.

**Steps**: 6 TDD steps, all COMMIT/PASS.

**Key decisions made during DELIVER**:
- OQ-1 (emoji path OsString) resolved: WalkDir handles `📅 Diaries` paths natively via `OsStr` — no manual conversion needed.
- TTY detection via `std::io::IsTerminal` (stable since Rust 1.70) enables non-interactive text output for piped use and acceptance tests.
- `SKIP=clippy` required on all commits due to a pre-existing double-dash bug in the devenv clippy hook configuration — not a code issue.

**Mutation gaps logged for future slice**:
- `probe()` error paths not exercised (deferred to slice-02 probe gold tests).
- `active_repo_filter` / `repo_filter_matches` always-true mutant not caught (RepoPicker feature deferred to slice-02).
- Backspace in Search mode not explicitly tested (covered by proptest invariant but not example test).

**Deferred to slice-02**: ClipboardPort/ArboardClipboardAdapter (US-08), Detail view (US-06), RepoPicker filter (US-07), probe gold tests.
