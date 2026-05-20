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

## Slice-04: Commit Detail Panel (2026-05-19)

**Shipped**: Detail mode overlay rendering. Enter on any row in Browse opens a full-screen
"Commit Detail" panel with un-truncated date, time, message, folder, and URL. Missing URL shows
"— not available —". Status bar shows "Esc to return". Esc returns to same row (domain already
handled this). 1 step TDD'd through RED→GREEN→COMMIT.

**Key design choices**:
- `detail_lines(&CommitRecord) -> Vec<String>` extracted as pure helper — testable without Frame.
- `render_detail_overlay` is a thin wrapper that calls `detail_lines` and renders into a bordered Block.
- Detail branch inserted in `render_main_area` before the table path — no layout changes needed.

**Mutation coverage breakthrough**: Added 7 ratatui `TestBackend` render tests to
`view_specifications.rs`, achieving 100% kill rate on view.rs (up from 0% for render functions).
Closes the deferred gap from slices 02 and 03. TestBackend setup is 3 lines; the pattern is now
established for future view tests in this codebase.

## Slice-05: Clipboard Copy (2026-05-19)

**Shipped**: `c` in Detail mode copies the repository URL to the system clipboard. Confirmation
message appears in the detail overlay. If URL is absent, shows "Copy not available — no URL".
If clipboard is unavailable (SSH/headless), shows "Copy not available — select text manually" with
graceful degradation. `ArboardClipboardAdapter` probed non-fatally at startup; result sets
`AppConfig.clipboard_available`. 2 steps TDD'd through RED→GREEN→COMMIT.

**Key design choices**:
- `AppModel.clipboard_pending: Option<String>` — pure domain signals the effect; event loop picks
  it up and dispatches `ClipboardResult`. Zero I/O in the domain update function.
- `ArboardClipboardAdapter::new()` created fresh inside each write/probe call — arboard::Clipboard
  is not Send/Sync; no stored state, just a zero-size struct.
- Clipboard probe is non-fatal: warn + degrade. Vault probe remains fatal.
- Status bar in Detail mode updated to "c copy | Esc return".

**Mutation**: 100% kill rate on domain/model.rs, domain/update.rs, tui/view.rs (70 caught, 7 unviable).

## Slice-06: Repository Filter (2026-05-20)

**Shipped**: `f` key in Browse mode opens a RepoPicker overlay listing distinct repositories by commit
count. `j`/`k` navigate the list; Enter applies the filter; Esc cancels. When a filter is active, `f`
clears it. Status bar shows `"{repo} • {filtered}/{total} commits | f clear | q quit"` when filtered.
2 steps TDD'd through RED→GREEN→COMMIT.

**Key design choices**:
- `pub fn distinct_repos(&[CommitRecord]) -> Vec<(String, usize)>` in `update.rs` — shared by both
  domain (Enter key sets filter) and view (picker list rendering). Single source of truth; no
  divergence between what's displayed and what's selected.
- Last URL path segment preferred; folder last segment is the fallback when `url` is `None`.
  `unwrap_or_default()` silently skips records with no extractable name (rare edge).
- `f` key in Browse mode is a toggle: `active_repo_filter.is_some()` → clear; else → open picker.
  No dedicated "clear filter" key; single key covers both actions depending on state.
- RepoPicker uses ratatui `List` widget + reversed style on `picker_cursor` row — same TestBackend
  render test pattern established in slice-04.

**Mutation gaps logged for future slice**:
- `len > 0` → `len >= 0` in `handle_repo_picker_key` (3×, empty-picker guard) — no test covers empty `commit_rows` in picker
- `i == picker_cursor` → `i != picker_cursor` in `render_repo_picker` — highlight test doesn't assert non-selected rows lack reversed style

**Mutation**: 95.9% kill rate on domain/update.rs, tui/view.rs (93 caught, 7 unviable, 4 missed).

## Slice-07: Deferred Acceptance Tests (2026-05-20)

**Shipped**: All 5 deferred walking-skeleton acceptance tests activated. Three passed immediately after
unskipping (config validation exit-code-2, emoji vault path, q-exit in non-TTY mode). Two required
`src/main.rs` fixes: missing-config early-exit (print "No config file found, using defaults" before
vault probe) and empty-vault non-TTY output ("No commits found in the last N days"). 1 step TDD'd
through RED→GREEN→COMMIT.

**Key design choices**:
- `config_absent = !config_path.exists()` captured before `config_adapter.load()` — allows detecting
  the absent-file case without changing the adapter API or AppConfig semantics.
- Early return after printing notice prevents vault probe from running with `vault_path = PathBuf::from("")`
  (AppConfig default), which would otherwise exit with code 1.
- Non-TTY empty-vault output changed to "No commits found in the last N days" — observable by acceptance
  tests; TTY path unchanged (empty table row shown as before).
- 3 tests needed zero production changes (scan_days_back validation, emoji path, q-exit) — existing
  behavior was already correct, scaffolds just needed unskipping.

**Mutation**: 28.6% (2/7) on main.rs composition root. The 2 caught mutants cover the new config_absent
and empty-vault logic. The 5 missed are pre-existing boilerplate (default_config_path helper,
verbosity counting, clipboard warning) not exercised by acceptance tests. Domain/view logic at ≥95.9%.
