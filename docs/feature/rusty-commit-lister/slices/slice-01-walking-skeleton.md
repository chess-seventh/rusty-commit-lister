# Slice 01 — Walking Skeleton

**Outcome**: End-to-end path proven — one real note, one parsed row, one rendered TUI cell.

## Why this slice is first

This is the riskiest assumption: does the full chain (Obsidian note → Markdown parser → ratatui
table → terminal) actually work in Rust with the Unicode vault path? Everything else builds on
this. Validating it first prevents late integration surprises.

## Scope (exactly what is included)

- Add Cargo.toml dependencies: `ratatui 0.26`, `crossterm 0.27`, `chrono 0.4`, `walkdir 2`, `toml 0.8`
- Config loader: reads `~/.config/rusty-commit-lister/config.toml`; surfaces `vault_path` and
  `scan_days_back` (default 7). Falls back gracefully if config absent.
- Markdown table parser: reads a single Obsidian daily note, extracts rows from the `## Commits`
  table. Columns: `FOLDER | TIME | COMMIT MESSAGE | REPOSITORY URL`.
- Unicode path handling: vault path with `📅 Diaries` must resolve via `OsString`/`PathBuf`.
- Minimal ratatui TUI: launches, renders a table with the 4 columns, shows the loaded rows,
  accepts `q`/`Esc` to exit cleanly.
- Status bar: shows commit count and scan window (e.g., "37 commits across 5 repos — last 7 days").
- NO search, NO detail view, NO filtering — those are slices 03+.

## Excluded (explicitly out of scope for this slice)

- Search (`/` activation) — slice 03
- Commit detail view (Enter) — slice 04
- Clipboard copy — slice 05
- Date range filter — slice 06
- PgUp/PgDn — slice 02 (added after basic j/k works)

## Acceptance criteria summary

- `rusty-commit-lister` runs and renders at least one commit row from a real daily note
- Unicode vault path resolves without panic or silent failure
- `q` exits cleanly — terminal prompt visible, no ANSI artefacts
- Status bar shows row count
- Startup time < 2 seconds for a 7-day scan on franci's machine

## Job traceability

`job-orient` — validates "Minimize time to identify active repositories in the scan window"
(job-map steps: locate, prepare, confirm, execute)

## Risk addressed

| Risk | How this slice addresses it |
|---|---|
| Unicode path OsString failure | Exercised in parser on real vault path |
| ratatui render crash | Full render cycle executed |
| Parser contract mismatch | Real Obsidian note used as fixture |
| Cargo.toml missing deps | Resolved as first action |

## Story IDs

- US-01: Config loader and vault path resolution
- US-02: Markdown table parser (single note → commit rows)
- US-03: Minimal ratatui table TUI (render + j/k + q exit)

## Estimated effort

2–3 days (Rust TUI setup + parser + integration)
