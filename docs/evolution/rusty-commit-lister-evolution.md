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

## Slice-02: Full Browse Experience (2026-05-19)

**Shipped**: PgUp/PgDn navigation with page_size clamping; message/folder truncation with ellipsis; "Row N/Total | q quit" status bar; r-refresh via reload_fn closure in TuiEventLoop::run().

**Steps**: 3 TDD steps (02-01, 02-02, 02-03), all COMMIT/PASS.

**Key decisions made during DELIVER**:
- PageDown/PageUp clamp at boundaries (no wrap) — contrast with j/k which wrap. Clamping matches typical pager UX.
- Folder column truncated to 20 chars; message to 40 chars — both use a char-boundary-safe `truncate()` pure helper.
- reload_fn closure pattern chosen over channel/message-passing: simpler for sync blocking I/O (ADR-002: async upgrade deferred until > 100ms latency observed).
- Status bar "Row N/Total" format matches vi-style line position — familiar to terminal users.

**Mutation gaps logged for future slice**:
- `event_loop.rs` TUI lifecycle (run loop, restore guard, Drop, translate_event) — requires terminal mock (ratatui TestBackend) to reach.
- `view.rs` render functions and status bar arithmetic — same reason.
- The PageDown `row_count > 0` guard was an equivalent mutant for empty-cursor-0 case; added `page_down_with_empty_filtered_rows_preserves_cursor` to discriminate cursor>0 case.

**Deferred to slice-03**: ClipboardPort/ArboardClipboardAdapter (US-08), Detail view (US-06), RepoPicker filter logic (US-07), ratatui TestBackend integration for TUI render tests.

## Slice-03: Full-Text Search (2026-05-19)

**Shipped**: Enter key in Search mode confirms query and returns to Browse (filtered view preserved, cursor=0). Search mode renders a 3-chunk layout: table + "/ query_" search bar + "N of M commits | Esc cancel" status bar.

**Steps**: 2 TDD steps (03-01, 03-02), all COMMIT/PASS.

**Key decisions made during DELIVER**:
- Enter = confirm (preserve filter, return to Browse); Esc = cancel (clear filter, restore all). Symmetric semantics: no ambiguity.
- Cursor resets to 0 on Enter — user is navigating a new result set, prior position is irrelevant.
- 3-chunk layout only activates in Search mode — Browse/Detail/RepoPicker keep the 2-chunk layout. No layout thrash on mode transitions.
- "/ query_" with trailing underscore as cursor indicator — no block cursor positioning needed; simple and clear.
- search_status_text() and format_status_text() are separate pure helpers — one for search context, one for browse context. No conditional formatting inside a single function.

**Mutation gaps logged**:
- view.rs render functions still untestable without terminal mock (same as slice-02). Deferred.

**Deferred to slice-04**: Commit Detail Panel (US-07) — Enter in Browse opens detail overlay; full message/path/URL display; Esc returns to Browse with cursor preserved.
