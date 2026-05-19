# Slice 04 — Commit Detail View

**Outcome**: Franci can see the full path and URL for any commit with one keypress — no Obsidian needed.

## Why this slice is fourth

Score 10 — third highest opportunity. Once franci has found the commit (browse or search), the
current workaround to get the full URL is to open Obsidian and scroll to the daily note. This
slice eliminates that app switch entirely.

## Scope

- `Enter` on any selected row opens a detail panel overlay
- Panel shows full (un-truncated): date/time, repo name, commit message, folder path, repository URL
- `Esc` closes panel and returns to the same row position in the table (scroll position preserved)
- If REPOSITORY URL is absent (older note format): show "— not available —", no crash
- Panel layout: minimum 80 columns; degrades gracefully below that

## Acceptance criteria summary

- Full 120-character message visible in detail panel without truncation
- Folder path with spaces or special characters renders correctly
- `Esc` from detail returns to same row (row 14 → detail → Esc → row 14 still selected)
- Missing URL shows "— not available —" without error
- Panel does not corrupt the underlying table state

## Job traceability

`job-inspect-commit` — resolves "Open the exact repo I was working in" without leaving terminal

## Story IDs

- US-07: Commit detail panel (Enter → overlay with full fields → Esc return)

## Estimated effort

1 day
