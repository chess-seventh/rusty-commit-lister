# Slice 02 — Full Browse Experience

**Outcome**: Franci can comfortably navigate a week of commits across all repos without reaching for grep.

## Why this slice is second

The walking skeleton proves the chain works. This slice makes it actually usable as a daily
orientation tool — the primary job. PgUp/PgDn and a proper multi-day scan complete the browse
loop that slice 01 only sketched.

## Scope

- Multi-day scan: walk all daily note files in the `scan_days_back` window (not just one note)
- `walkdir` traversal with correct date-range filtering
- `chrono` date parsing from note filenames (YYYY-MM-DD.md pattern)
- Responsive navigation: `j`/`k` (one row), `PgUp`/`PgDn` (one viewport page)
- Truncation with ellipsis (…) for long messages and folder paths
- Row counter in status bar updates on every navigation event: "Row N/Total"
- `r` to refresh (re-scan vault and redraw)
- Scroll wraps at top and bottom without crash

## Acceptance criteria summary

- 37 commits across 7 daily notes rendered and navigable in < 2 seconds
- PgDn jumps by visible-row count; PgUp reverses
- Truncated messages show … without overflowing adjacent columns
- `r` re-scans vault and updates table while preserving cursor position (best effort)
- Status bar shows "Row N/Total" updating in real time

## Job traceability

`job-orient` — job-map steps: execute (browse), monitor (track position)

## Story IDs

- US-04: Multi-day note traversal and date-filtered scan
- US-05: Full navigation (PgUp/PgDn, wrap, truncation, r-refresh, row counter)

## Estimated effort

1–2 days
