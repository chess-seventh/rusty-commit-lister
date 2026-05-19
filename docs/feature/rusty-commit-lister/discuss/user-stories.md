<!-- markdownlint-disable MD024 -->
# User Stories — rusty-commit-lister

Generated: 2026-05-18 — DISCUSS wave
Feature: rusty-commit-lister
Persona: franci — sole user, terminal-first developer, Obsidian Diary user

---

## System Constraints

- Platform: macOS (Darwin) + Linux (NixOS/devenv); terminal-first
- Minimum terminal width: 80 columns; warn if narrower
- Dependencies: ratatui 0.26, crossterm 0.27, chrono 0.4, walkdir 2, toml 0.8
- Data contract: `## Commits` Markdown table from rusty-commit-saver — treat as stable, immutable
- Unicode path: `~/Documents/Wiki/📅 Diaries/0. Journal/YYYY/MM-MMM/YYYY-MM-DD.md` — must work end-to-end
- Performance: startup < 2 seconds for scan_days_back ≤ 30; < 100ms to first output
- Exit codes: 0 = success, 1 = runtime error, 2 = usage/config error; SIGINT → 130
- No technology prescriptions: solution details belong to DESIGN wave
- Config precedence: flags > env vars > config.toml > defaults

---

## US-01: Config Loader and Vault Path Resolution

### Elevator Pitch

**Before**: To orient myself about what I committed, I have to either grep through Obsidian files or open the app manually — no single-command way to configure where my notes live.
**After**: I run `rusty-commit-lister` and it reads `~/.config/rusty-commit-lister/config.toml` for my vault path and scan window, falling back to sensible defaults if absent.
**Decision enabled**: I can trust the tool is scanning the right location without re-reading config every session.

### Problem

franci is a terminal-first developer with her Obsidian vault at a Unicode emoji path. She finds it
frustrating to hard-code paths in tools or get cryptic startup errors when config is missing — she
wants a tool that either reads her config silently or gives her a clear, actionable message.

### Who

- franci | Terminal session, tool first run or daily use | Get the tool scanning the right vault immediately

### Solution

Config loader reads `~/.config/rusty-commit-lister/config.toml` on startup. Surfaces `vault_path`
and `scan_days_back`. Falls back to `scan_days_back = 7` and shows a one-line notice if config absent.
Vault path is resolved via `PathBuf` supporting Unicode (including `📅` emoji segment).

### Domain Examples

#### 1: Normal startup with config present
franci has `~/.config/rusty-commit-lister/config.toml` with `vault_path = "~/Documents/Wiki/📅 Diaries/0. Journal"` and `scan_days_back = 7`. Tool starts, reads config silently, begins scanning. No messages about config.

#### 2: Config missing — graceful fallback
franci installs the binary on a new machine. No config exists. Tool starts with `scan_days_back = 7` default and prints: `Using defaults — no config found at ~/.config/rusty-commit-lister/config.toml`. franci can still use the tool immediately.

#### 3: Invalid scan_days_back value
franci accidentally sets `scan_days_back = -1` in config. Tool exits with code 2 and prints:
`Config error: scan_days_back must be a positive integer (got -1). Edit ~/.config/rusty-commit-lister/config.toml to fix.`

### UAT Scenarios (BDD)

```gherkin
Scenario: Config loads silently when file is present and valid
  Given franci has a valid config.toml with vault_path and scan_days_back = 7
  When rusty-commit-lister starts
  Then it reads vault_path and scan_days_back without printing any config messages
  And scanning begins on the configured vault path

Scenario: Missing config falls back to defaults with notice
  Given no config file exists at ~/.config/rusty-commit-lister/config.toml
  When rusty-commit-lister starts
  Then scan_days_back defaults to 7
  And franci sees: "Using defaults — no config found at ~/.config/rusty-commit-lister/config.toml"
  And the tool continues to start normally

Scenario: Unicode emoji in vault path resolves correctly
  Given vault_path contains "📅 Diaries" in the directory segment
  When rusty-commit-lister resolves the path
  Then the path is valid and walkdir traversal finds files in that directory
  And no error or panic occurs due to the emoji character

Scenario: Invalid config value produces actionable error
  Given config.toml contains scan_days_back = -1
  When rusty-commit-lister starts
  Then it exits with code 2
  And the error message names the invalid value and the config file path to fix
```

### Acceptance Criteria

- [ ] Config loads silently when `~/.config/rusty-commit-lister/config.toml` is present and valid
- [ ] Missing config triggers default fallback (scan_days_back=7) plus one-line notice
- [ ] Vault path with `📅` emoji resolves without panic or silent failure
- [ ] Invalid `scan_days_back` exits with code 2 and actionable error message naming the config file

### Outcome KPIs

- **Who**: franci
- **Does what**: starts tool without config friction or cryptic errors
- **By how much**: zero failed startups due to config errors after initial setup
- **Measured by**: count of config-related exit code 2 occurrences in shell history
- **Baseline**: no baseline (new tool)

### Technical Notes

- Unicode path handling via `OsString`/`PathBuf` — spike test early in slice-01
- Config format: TOML via `toml 0.8` crate (already in Cargo.toml plan)
- No secrets in config; vault_path is not sensitive
- `~` in vault_path must expand to home directory (`dirs` crate or manual `$HOME` substitution)

### job_id: job-orient
### Dependencies: None (first story — slice 01)

---

## US-02: Markdown Table Parser

### Elevator Pitch

**Before**: My commit data lives in Obsidian daily notes as Markdown tables written by rusty-commit-saver — but there is no way to read them programmatically without grepping.
**After**: `rusty-commit-lister` parses the `## Commits` table from each daily note and produces structured commit rows ready for display.
**Decision enabled**: I can see all my commits from the last week in one view without manually reading each note file.

### Problem

franci's commit data is in Markdown tables in Obsidian daily notes, written by rusty-commit-saver.
She finds it painful to query this data — grep misses structure and she has to mentally parse
table rows. A reliable parser removes this entirely.

### Who

- franci | Running the tool at session start | See structured commits without touching note files

### Solution

Parser reads a daily note file, locates the `## Commits` section, extracts table rows.
Produces structured records with: folder, time, commit message, repository URL.
Skips malformed rows with a debug log entry — no panic.

### Domain Examples

#### 1: Standard note with 5 commits
`2026-05-18.md` has a `## Commits` table with 5 rows. Parser produces 5 structs with all four
fields populated. franci sees 5 rows in the TUI.

#### 2: Note with no Commits section
`2026-05-16.md` was a Sunday with no commits — the section `## Commits` is absent. Parser returns
zero rows for this file. No error. Tool continues scanning other dates.

#### 3: Malformed row (missing REPOSITORY URL column)
One row in the table is from an older rusty-commit-saver version that did not write the URL column.
Parser skips that row, logs `[DEBUG] skipped malformed row at 2026-05-14.md:line 23`, and continues.

### UAT Scenarios (BDD)

```gherkin
Scenario: Standard note produces expected commit rows
  Given a daily note at 2026-05-18.md contains a "## Commits" table with 5 rows
  When the parser reads that note
  Then it produces 5 commit records
  And each record has folder, time, message, and url fields populated

Scenario: Note with no Commits section produces zero rows without error
  Given a daily note contains no "## Commits" section
  When the parser reads that note
  Then it returns an empty list of records
  And no error or warning is shown to franci

Scenario: Malformed row is skipped with debug log
  Given a row in the Commits table is missing the REPOSITORY URL column
  When the parser encounters that row
  Then that row is skipped
  And a debug log entry is written (not shown to franci in normal mode)
  And all other rows in that file are parsed correctly

@property
Scenario: Parser never panics on any Obsidian note file
  Given any file at the vault path with extension .md
  Then the parser produces a result (empty or populated) without panicking
  And franci sees no crash output
```

### Acceptance Criteria

- [ ] Parser produces one struct per valid row in the `## Commits` table
- [ ] Note with no `## Commits` section → 0 rows, no error
- [ ] Malformed row → skip + debug log, remaining rows parsed correctly
- [ ] Parser never panics on any `.md` file at the vault path

### Outcome KPIs

- **Who**: franci
- **Does what**: sees accurate commit count matching her actual commit history
- **By how much**: zero silent data loss — parsed count matches spot-check manual count
- **Measured by**: spot-check: count rows in one note file, verify TUI count matches
- **Baseline**: no baseline (new tool)

### Technical Notes

- Data contract: `FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL` — document as constant in parser module
- Parser must be tested with a real Obsidian note as fixture (not synthetic test data)
- `chrono 0.4` for time parsing; `YYYY-MM-DD HH:MM` format from TIME column
- Skip-and-log strategy: `tracing::debug!` for malformed rows; `tracing::warn!` for missing `## Commits` section if unexpected

### job_id: job-orient
### Dependencies: US-01 (vault_path needed to locate note files)

---

## US-03: Minimal Ratatui TUI — Render and Exit

### Elevator Pitch

**Before**: There is no way to see parsed commits in a terminal table — the parser output has nowhere to go.
**After**: Running `rusty-commit-lister` launches a ratatui table showing loaded commit rows with j/k navigation and q to exit cleanly.
**Decision enabled**: I can verify the full chain works (config → parse → render) and start using the tool for daily orientation immediately.

### Problem

franci needs a terminal-native table that displays her parsed commits. Without a TUI, the parser
output is invisible. The minimal TUI proves the chain and provides the first usable version of
the tool — even if it only handles one note and basic navigation.

### Who

- franci | Opening terminal at session start | See commits in a navigable table immediately

### Solution

Ratatui TUI with a 4-column table (Date+Time | Repo | Message | Folder). Renders parsed
`commit_rows`. j/k navigation. q or Esc exits cleanly — terminal restored. Status bar shows
commit count and scan window. Loading indicator within 100ms of launch.

### Domain Examples

#### 1: Normal startup — 37 commits loaded
franci runs `rusty-commit-lister`. Within 1.5 seconds she sees a table with 37 rows.
Status bar: "37 commits across 5 repos — last 7 days". She scrolls with j/k, sees each row
highlight in sequence, presses q. Terminal prompt appears cleanly.

#### 2: Zero commits — empty state
franci runs the tool on a day she made no commits. TUI shows: "No commits found for the last
7 days. Vault: ~/Documents/Wiki/📅 Diaries/0. Journal". franci can see the config was read
correctly even though the list is empty.

#### 3: Startup time on large scan
franci sets scan_days_back=30. Tool starts scanning. Loading indicator appears immediately.
TUI renders rows progressively (or all at once after < 2 seconds). No frozen terminal.

### UAT Scenarios (BDD)

```gherkin
Scenario: TUI renders within startup time threshold
  Given franci has 37 commits in the last 7 days across 5 repos
  When franci runs `rusty-commit-lister`
  Then a loading indicator appears within 100ms
  And the full table renders within 2 seconds
  And the status bar shows "37 commits across 5 repos — last 7 days"

Scenario: j/k navigation moves row selection
  Given the TUI is displaying 37 commit rows
  And the cursor is on row 1
  When franci presses j three times
  Then the cursor is on row 4
  And row 4 is visually highlighted (▶ indicator)
  And the status bar shows "Row 4/37"

Scenario: q exits and restores terminal cleanly
  Given the TUI is displaying the commit table
  When franci presses q
  Then the TUI closes
  And the terminal prompt appears on a clean line
  And no ANSI escape artefacts remain visible

Scenario: Esc exits the same as q from main table
  Given the TUI is displaying the commit table (not in search mode)
  When franci presses Esc
  Then the TUI closes with exit code 0
  And terminal state is restored

Scenario: Empty scan window shows informative state
  Given no daily notes exist for the last 7 days
  When franci runs `rusty-commit-lister`
  Then the TUI shows "No commits found for the last 7 days"
  And the vault path that was searched is shown
  And exit code is 0 (not an error)
```

### Acceptance Criteria

- [ ] First output (loading indicator) within 100ms of launch
- [ ] Full table renders within 2 seconds for scan_days_back ≤ 30
- [ ] j/k moves cursor one row; selected row shows ▶ indicator + highlight
- [ ] Status bar shows "Row N/Total" and "X commits across Y repos — last N days"
- [ ] q exits with code 0 and terminal fully restored (no ANSI artefacts)
- [ ] Esc from main table exits same as q
- [ ] Empty scan window shows informative empty state with vault path

### Outcome KPIs

- **Who**: franci
- **Does what**: opens and uses the tool without any friction in the first session
- **By how much**: completes first orientation scan (launch → browse → exit) in < 2 minutes, unassisted
- **Measured by**: stopwatch + "consulted README?" binary log on first use
- **Baseline**: no baseline (new tool)

### Technical Notes

- ratatui 0.26 + crossterm 0.27 for terminal backend
- Elm architecture: Model (app state) / Update (event handler) / View (render function)
- Alt screen + raw mode on enter; restore on exit via `Drop` or explicit cleanup
- SIGINT (Ctrl+C) must also restore terminal — register handler in slice-01
- Message/folder columns truncate to available width with `…`; Date+Repo columns fixed width

### job_id: job-orient
### Dependencies: US-01 (config), US-02 (parsed rows)

---

## US-04: Multi-Day Note Traversal and Date-Filtered Scan

### Elevator Pitch

**Before**: The walking skeleton only parses one note file — I can't see a full week of commits.
**After**: `rusty-commit-lister` walks all daily notes within the `scan_days_back` window and renders the complete commit history across all repos.
**Decision enabled**: I can orient myself about a full week's work in a single scan, choosing which day range to review.

### Problem

franci works across multiple repos every day. She needs to see a week's history in one view —
not just today. The walking skeleton only proved one-note parsing; this story closes the gap.

### Who

- franci | Start of working week or retrospective | See all commits across all repos for the configured window

### Solution

`walkdir` traversal of the vault's journal directory, filtered by `chrono` date range
(today - scan_days_back to today). Collects all daily note files in range, passes each to
the parser, merges results sorted by date+time descending.

### Domain Examples

#### 1: 7-day scan across 5 repos
franci runs with default scan_days_back=7. Tool finds 7 daily note files (skipping any days
with no note), parses 37 total commits across 5 repos, renders them sorted newest-first.

#### 2: scan_days_back=1 — today only
franci wants to quickly check today's commits. scan_days_back=1. Only today's note is loaded.
If franci committed 6 times today, she sees 6 rows.

#### 3: scan_days_back=30 — month in review
franci sets scan_days_back=30 for a monthly review. Tool finds ~22 note files (excluding
weekends with no commits). 200+ rows loaded. Loading takes up to 2 seconds but completes.

### UAT Scenarios (BDD)

```gherkin
Scenario: All notes in scan window contribute rows
  Given franci has daily notes for 2026-05-12 through 2026-05-18 (7 notes)
  And scan_days_back = 7
  When rusty-commit-lister loads
  Then commits from all 7 notes appear in the table
  And commits are sorted newest-first (2026-05-18 at top)

Scenario: Date gap in notes handled correctly
  Given no daily note exists for 2026-05-15 (no commits that day)
  And notes exist for all other days in the scan window
  When rusty-commit-lister loads
  Then commits from the other 6 days appear correctly
  And no error is shown for the missing date

Scenario: Large scan window loads within time threshold
  Given scan_days_back = 30 and approximately 200 commits across 22 notes
  When rusty-commit-lister loads
  Then all rows are visible in the TUI within 2 seconds
  And the status bar shows the correct total commit count
```

### Acceptance Criteria

- [ ] All note files within the scan window are parsed and merged
- [ ] Results sorted newest-first by date+time
- [ ] Missing dates (no note file) are silently skipped — no error
- [ ] scan_days_back=30 completes within 2 seconds on franci's machine
- [ ] Status bar shows correct totals matching manual spot-check count

### Outcome KPIs

- **Who**: franci
- **Does what**: views a full week of commits in one TUI session
- **By how much**: first session covers full 7-day window; no gaps in commit history
- **Measured by**: spot-check: count commits in daily notes, verify TUI count matches
- **Baseline**: walking skeleton only loads 1 day

### Technical Notes

- `walkdir 2` for traversal; filter by filename pattern `YYYY-MM-DD.md`
- `chrono 0.4` for date arithmetic (today - N days)
- Notes outside the scan window must not be parsed (performance and correctness)
- Sort: merge-sort by `(date, row_in_file)` tuple for stable ordering

### job_id: job-orient
### Dependencies: US-01, US-02, US-03

---

## US-05: Full Navigation — PgUp/PgDn, Truncation, Refresh, Row Counter

### Elevator Pitch

**Before**: With 37 commits visible, I can only move one row at a time — scrolling to the bottom means 37 j presses.
**After**: PgUp/PgDn jumps a full viewport, truncated messages show `…`, r refreshes the list, and the status bar always shows my position.
**Decision enabled**: I can efficiently scan a long list and immediately know where I am without losing orientation.

### Problem

franci has up to 200 commits in a 30-day window. One-row-at-a-time navigation becomes tedious.
She needs page-level navigation and clear position feedback to maintain orientation in a long list.

### Who

- franci | Browsing a week+ of commits | Navigate quickly without losing position or missing data

### Solution

PgDn/PgUp jump by viewport height (visible rows). Truncation renders long messages/paths with
`…` at column boundary. `r` re-scans vault and rebuilds `commit_rows`, preserving cursor position
where possible. Status bar row counter updates on every navigation event.

### Domain Examples

#### 1: PgDn on a 37-row list with 10-row viewport
franci's terminal shows 10 rows at a time. She presses PgDn. Rows 11–20 appear. Cursor moves
to row 11. Status bar: "Row 11/37". Another PgDn → rows 21–30, cursor at row 21.

#### 2: Long message truncation
A commit message is "refactor: extract markdown table parser into separate module with full error handling". The column is 40 chars wide. TUI shows: `refactor: extract markdown table pars…`. No text overflows.

#### 3: Refresh after new commits
franci runs with scan_days_back=1. She makes 2 more commits in another terminal. She presses `r`.
The TUI re-scans and shows 2 new rows at the top. Cursor stays at the previously selected row
(or top if that row no longer exists).

### UAT Scenarios (BDD)

```gherkin
Scenario: PgDn advances by viewport height
  Given the table has 37 rows and the viewport shows 10 rows at a time
  And franci's cursor is on row 1
  When franci presses PgDn
  Then rows 11-20 are visible
  And the cursor is on row 11
  And the status bar shows "Row 11/37"

Scenario: Long commit message truncated with ellipsis
  Given a commit message exceeds the available column width
  When that row is displayed in the table
  Then the message shows the first N characters followed by "…"
  And no text from that column bleeds into adjacent columns

Scenario: r-refresh reloads data from vault
  Given franci has the TUI open showing 10 commits
  And 2 new commits have been written to today's daily note
  When franci presses r
  Then the table shows 12 commits
  And the new commits appear at the top (newest-first sort)

Scenario: Navigation wraps at list boundaries without crash
  Given franci's cursor is on the last row of the list
  When franci presses j
  Then the cursor wraps to row 1
  And no error or crash occurs
```

### Acceptance Criteria

- [ ] PgDn advances cursor and viewport by visible-row count
- [ ] PgUp reverses by the same amount
- [ ] Messages truncated with `…` — no column overflow
- [ ] `r` re-scans vault and updates table; cursor preserved at same row index (or top if out of range)
- [ ] j/k wrap at boundaries without crash
- [ ] Status bar shows "Row N/Total" updating on every navigation event

### Outcome KPIs

- **Who**: franci
- **Does what**: completes orientation scan in < 30 seconds for a 7-day window
- **By how much**: PgDn to bottom of 37-row list ≤ 4 keypresses (37/10 ≈ 4 pages)
- **Measured by**: count keypresses in self-test session
- **Baseline**: slice-02 navigation is j-only

### Technical Notes

- Viewport height = terminal rows minus header row minus status bar rows (typically 2)
- Truncation width calculated per render cycle based on actual terminal width
- `r` re-runs full load pipeline; async preferred to avoid TUI freeze during re-scan
- Wrap: cursor < 0 → last row; cursor > last row → 0

### job_id: job-orient
### Dependencies: US-03, US-04

---

## US-06: Inline Full-Text Search

### Elevator Pitch

**Before**: To find a specific commit I vaguely remember, I have to `grep -r "keyword" ~/Documents/Wiki/...` — breaking my terminal flow and requiring me to know the path structure.
**After**: I press `/` in the TUI, type a keyword, and the list narrows in real time to matching commits across message and repo fields.
**Decision enabled**: I can locate a specific commit without knowing which repo it was in or what day it happened.

### Problem

franci occasionally needs to find a specific commit she vaguely remembers — "that refactor I did
last week" or "something in obsidian-utils". The current workaround (grep -r) breaks her flow,
requires exact path knowledge, and returns raw Markdown, not a navigable list.

### Who

- franci | Browse view, list too long to scan manually | Find a specific commit by keyword in < 30 seconds

### Solution

`/` activates inline search bar at the bottom of the TUI. Typing filters `commit_rows` in
real time by substring match on MESSAGE and REPO fields (case-insensitive). Status bar shows
"N of Total". Esc clears query and restores full list. Enter stays in filtered view (search bar closes).

### Domain Examples

#### 1: Searching by keyword in message
franci presses `/` and types "emoji". 3 commits matching "emoji" across 2 repos appear.
Status bar: "3 of 37 commits". She navigates to the right one and presses Enter for detail.

#### 2: Searching by repo name
franci presses `/` and types "dotfiles". Only dotfiles commits appear. She can now scan her
dotfiles activity for the week without needing the repo filter.

#### 3: Esc clears search
franci types "rusty" and sees 4 results. She presses Esc. All 37 commits reappear. Search bar
is gone. Cursor is at the top of the full list.

### UAT Scenarios (BDD)

```gherkin
Scenario: Typing in search bar filters the list in real time
  Given 37 commits are loaded and franci presses /
  When franci types "emoji"
  Then only commits where MESSAGE or REPO contains "emoji" (case-insensitive) are shown
  And the status bar shows "N of 37 commits" where N is the match count

Scenario: Search is case-insensitive
  Given a commit message is "Fix: Handle Emoji Path"
  When franci types "emoji" in the search bar
  Then that commit appears in the filtered list

Scenario: Esc from search restores full list
  Given franci has typed "dotfiles" and sees a filtered list
  When franci presses Esc
  Then all 37 commits are visible again
  And the search bar is cleared from the display
  And the cursor is at the top of the full list

Scenario: Empty search query shows full list
  Given franci has activated search with /
  When franci types nothing (empty query)
  Then all 37 commits remain visible
  And the search bar shows the cursor but no filtering is applied

Scenario: Esc from main table (not search) exits the tool
  Given the TUI is showing the table and search is NOT active
  When franci presses Esc
  Then the TUI exits cleanly (same as q)
  And the terminal is restored
```

### Acceptance Criteria

- [ ] `/` activates search bar; typing filters MESSAGE and REPO columns simultaneously
- [ ] Filtering is case-insensitive
- [ ] Status bar shows "N of Total commits" match count updating per keypress
- [ ] Esc from search clears query and restores full list
- [ ] Esc from main table (search not active) exits the tool
- [ ] Empty query shows full unfiltered list

### Outcome KPIs

- **Who**: franci
- **Does what**: finds a vaguely-remembered commit using search (/) without grepping
- **By how much**: first / search invoked within 48 hours of slice-03 ship; grep usage for commit lookup drops to 0
- **Measured by**: self-observation + shell history grep count
- **Baseline**: 100% of "find a specific commit" sessions use grep (behavioral evidence)

### Technical Notes

- State machine: app_mode = Browse | Search | Detail; Esc behaviour depends on mode
- Real-time filter: apply substring filter on every keypress; avoid full re-parse (filter in-memory `commit_rows`)
- Case-insensitive: `str.to_lowercase().contains(&query.to_lowercase())`
- The search bar is rendered as an input widget at the bottom of the TUI (ratatui `Paragraph` in input mode)

### job_id: job-find-commit
### Dependencies: US-03, US-04, US-05

---

## US-07: Commit Detail Panel

### Elevator Pitch

**Before**: The table truncates long messages and folder paths — I have to open Obsidian to see the full URL or path.
**After**: Pressing Enter on any row opens a detail panel showing the full, untruncated message, folder path, and repository URL.
**Decision enabled**: I can get the full context of any commit — including its URL — without leaving the TUI.

### Problem

franci sometimes needs the full commit message, folder path, or repository URL from a commit she
has found. The table truncates these for readability. Opening Obsidian to read the raw note is
slow and breaks her flow.

### Who

- franci | Browse or search view, found the commit she wanted | Get full details without app switch

### Solution

`Enter` on any selected row opens a detail overlay panel. Displays full (un-truncated): date/time,
repo name, commit message, folder path, repository URL. `Esc` closes panel and returns to the
same row position. Missing URL shows "— not available —" with no crash.

### Domain Examples

#### 1: Opening detail for a commit with long message
franci finds a commit with a 120-character message that was truncated in the table. She presses
Enter. Detail panel shows the full message on multiple lines if needed. She reads it and presses Esc.

#### 2: Opening detail at row 14, Esc returns to row 14
franci navigates to row 14, presses Enter, reads the detail, presses Esc. Table is back with
row 14 highlighted. She did not lose her place.

#### 3: Old commit with no URL column
A commit from a note created before rusty-commit-saver started writing the URL column has no
URL value. Detail panel shows "— not available —" in the URL field. franci is not confused or
faced with an error.

### UAT Scenarios (BDD)

```gherkin
Scenario: Detail panel shows full untruncated commit message
  Given a commit row with a 120-character message is selected
  When franci presses Enter
  Then the detail panel opens
  And the full 120-character message is visible without truncation
  And the folder path and repository URL are also visible

Scenario: Closing detail returns to same row position
  Given franci has navigated to row 14 in the commit table
  And the detail panel is open for row 14
  When franci presses Esc
  Then the commit table is shown with row 14 selected
  And the scroll position is unchanged

Scenario: Missing URL shown as not-available placeholder
  Given a commit row has an empty REPOSITORY URL field
  When franci opens the detail panel for that row
  Then the URL field shows "— not available —"
  And no error or crash occurs

Scenario: Detail panel does not corrupt table state
  Given franci opens and closes the detail panel 5 times on different rows
  When franci returns to the main table after each close
  Then the table shows the same commit rows as before
  And navigation continues to work normally
```

### Acceptance Criteria

- [ ] Enter opens detail panel over the current row
- [ ] Full un-truncated message, folder path, and URL visible in panel
- [ ] Esc closes panel and returns to the same row index (scroll preserved)
- [ ] Missing URL shows "— not available —" without error
- [ ] Panel does not corrupt underlying table state after close

### Outcome KPIs

- **Who**: franci
- **Does what**: gets full commit context without opening Obsidian
- **By how much**: Obsidian-opens for URL retrieval drops to 0 within one week of slice-04 ship
- **Measured by**: self-observation: did I open Obsidian for a URL today?
- **Baseline**: every URL retrieval requires opening Obsidian and scrolling

### Technical Notes

- Detail panel: render as a `Block` + `Paragraph` overlay, centered or full-width
- App mode: Browse → Detail on Enter; Detail → Browse on Esc
- Cursor index preserved in app state; not re-calculated on panel close
- URL and path fields: no truncation in detail panel; wrap long lines if needed

### job_id: job-inspect-commit
### Dependencies: US-03, US-05

---

## US-08: Clipboard URL Copy

### Elevator Pitch

**Before**: Even with the detail panel showing the URL, I have to manually select and copy it — or switch to Obsidian.
**After**: Pressing `c` in the detail panel copies the repository URL to the system clipboard with a one-line confirmation.
**Decision enabled**: I can paste the URL immediately into a browser, chat, or terminal — zero mouse interaction.

### Problem

franci often needs to act on a repository URL — open it in a browser, paste it into a message,
or reuse it in a git command. Even with the detail view, manual text selection in a terminal is
fiddly and breaks keyboard-driven flow.

### Who

- franci | Detail panel open, URL visible | Copy URL to clipboard in one keypress

### Solution

In the detail panel, `c` copies the REPOSITORY URL to the system clipboard. Confirmation line
appears: "URL copied to clipboard". If clipboard is unavailable (SSH/headless): show URL as
highlighted/selectable text instead — no crash.

### Domain Examples

#### 1: Copy URL in normal macOS terminal
franci presses `c` in the detail panel. `https://github.com/franci/obsidian-utils` is in her
clipboard. Panel shows: "URL copied to clipboard". She switches to browser, Cmd+V, done.

#### 2: Copy in SSH session (clipboard unavailable)
franci is SSH-ed into a remote machine. She presses `c`. No clipboard available. Panel shows:
`https://github.com/franci/obsidian-utils` highlighted, and a note: "Copy not available — select text manually."
She can triple-click or use terminal selection. No crash.

#### 3: Copy when URL is not available
franci presses `c` on a commit with no URL. Panel shows: "No URL to copy." No crash.

### UAT Scenarios (BDD)

```gherkin
Scenario: c copies URL to clipboard and shows confirmation
  Given the detail panel is open for a commit with URL "https://github.com/franci/obsidian-utils"
  When franci presses c
  Then the system clipboard contains "https://github.com/franci/obsidian-utils"
  And the panel displays "URL copied to clipboard"

Scenario: Clipboard unavailable falls back to text display
  Given rusty-commit-lister is running in an environment without clipboard access
  When franci presses c
  Then the URL is displayed as plain selectable text in the panel
  And a message indicates clipboard is not available
  And no crash or panic occurs

Scenario: c on a row with no URL shows friendly message
  Given the detail panel shows "— not available —" in the URL field
  When franci presses c
  Then the panel shows "No URL to copy"
  And nothing is written to the clipboard
```

### Acceptance Criteria

- [ ] `c` in detail panel copies URL to system clipboard
- [ ] Clipboard confirmation message shown after copy
- [ ] Clipboard unavailable: URL shown as text with "select manually" note — no crash
- [ ] Pressing `c` when URL is "— not available —" shows "No URL to copy" without crash

### Outcome KPIs

- **Who**: franci
- **Does what**: completes the orient → inspect → act loop without any mouse interaction
- **By how much**: URL retrieval + paste completes in < 5 seconds from TUI; zero mouse clicks required
- **Measured by**: stopwatch self-test: time from pressing Enter (detail) to URL in clipboard
- **Baseline**: URL retrieval via Obsidian ≈ 30–60 seconds including app switch

### Technical Notes

- Clipboard: `arboard` crate (cross-platform) or `clipboard` crate; handle `Error` on unavailable
- SSH detection: if clipboard errors, catch and display fallback text — do not propagate as crash
- `c` only active in Detail mode; ignored in Browse and Search modes

### job_id: job-inspect-commit
### Dependencies: US-07 (detail panel must exist first)

---

## US-09: Interactive Repository Filter

### Elevator Pitch

**Before**: To see only commits from one repo, I have to type a search query that matches the repo name — which works but requires remembering exact names.
**After**: Pressing `f` opens a repo picker showing all distinct repo names with commit counts. Selecting one filters the list instantly.
**Decision enabled**: I can quickly isolate one repo's activity for a weekly review without typing anything.

### Problem

franci occasionally wants to see all her dotfiles commits, or all her rusty-commit-saver work,
for a weekly review. The search (/) works as a workaround but requires knowing the exact repo
name. A picker that shows all repos with counts is faster and more discoverable.

### Who

- franci | Browse view, wants to isolate one repo | See that repo's commits without typing

### Solution

`f` opens a repo picker overlay listing all distinct repo names found in the current scan window
with commit counts. Arrow keys or j/k select a repo. Enter applies filter. Status bar shows
active filter. `Esc` or `f` clears filter and restores full list. Filter composes with search.

### Domain Examples

#### 1: Weekly dotfiles review
franci presses `f`. Picker shows: "dotfiles (12)  rusty-commit-saver (8)  obsidian-utils (6)  rusty-commit-lister (11)". She selects "dotfiles". Table shows 12 dotfiles commits. Status bar: "dotfiles • 12 of 37 commits".

#### 2: Combined repo filter + search
franci applies repo filter for "dotfiles", then presses `/` and types "nvim". Table narrows to
dotfiles commits mentioning nvim. Status bar: "dotfiles ∩ nvim • 3 of 37 commits".

#### 3: Clearing filter
franci has "dotfiles" filter active. She presses `f` again (or Esc). Full list of 37 commits
returns. Status bar reverts to "37 commits across 5 repos — last 7 days".

### UAT Scenarios (BDD)

```gherkin
Scenario: f opens repo picker with all distinct repos and counts
  Given the table has 37 commits across 5 repos
  When franci presses f
  Then a picker overlay appears listing all 5 repo names
  And each name shows its commit count in parentheses

Scenario: Selecting a repo filters the table
  Given the repo picker is open
  When franci selects "dotfiles"
  Then the main table shows only commits where REPO is "dotfiles"
  And the status bar shows "dotfiles • 12 of 37 commits"

Scenario: Repo filter composes with search
  Given franci has applied repo filter for "dotfiles"
  When franci activates search and types "nvim"
  Then the table shows commits from dotfiles that also contain "nvim" in the message
  And the status bar reflects both active constraints

Scenario: f or Esc clears active repo filter
  Given repo filter "dotfiles" is active
  When franci presses f (or Esc)
  Then all 37 commits are visible again
  And the status bar shows the full commit count
```

### Acceptance Criteria

- [ ] `f` opens picker with all distinct repo names + commit counts
- [ ] Selecting a repo filters the table to that repo's commits
- [ ] Status bar shows active filter and filtered + total count
- [ ] `f` or `Esc` from picker or filtered view clears the filter
- [ ] Filter composes with search: both can be active simultaneously
- [ ] Config-level `repo_filter` (pre-filter at load time) is independent of this TUI filter

### Outcome KPIs

- **Who**: franci
- **Does what**: isolates one repo's commits for review in ≤ 2 keypresses
- **By how much**: weekly repo review takes < 60 seconds using filter; previously required Obsidian scroll + mental tally
- **Measured by**: self-observation on weekly review day
- **Baseline**: no repo filter exists; workaround is / search with typed repo name

### Technical Notes

- Picker: ratatui `List` widget in overlay block
- Distinct repos: derived from `commit_rows` at render time (no separate data structure needed)
- Filter state in app model: `active_repo_filter: Option<String>`
- Composes with `search_query`: both applied as `AND` conditions to filtered view
- Config `repo_filter` applied at parse time (different from TUI filter); both can coexist

### job_id: job-review-repo-activity
### Dependencies: US-03, US-05, US-06 (compose with search)
