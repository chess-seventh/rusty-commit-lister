# Slice 03 — Full-Text Search

**Outcome**: Franci can find a commit she vaguely remembers in seconds — no grep, no app switch.

## Why this slice is third

Score 12 — second highest opportunity after the core TUI. Once browse works (slice 02), the
next friction point is the list being too long to scan manually when looking for something specific.
Search is the primary workaround replacement (grep -r).

## Scope

- `/` key activates inline search bar at the bottom of the TUI
- Real-time filtering as franci types: matches MESSAGE and REPO columns simultaneously
- Case-insensitive matching
- Status bar updates to show "N of Total commits" match count while typing
- `Esc` clears query and restores full list; cursor returns to top of filtered set
- `Enter` in search bar confirms and stays in filtered view (search bar closes)
- Empty query restores full list

## Acceptance criteria summary

- Typing "emoji" matches commits with "emoji" in message or repo regardless of case
- Match count updates within a single render frame after each keypress
- `Esc` from search restores full list and clears search bar
- `Esc` from main table (not in search) exits tool (no confusion)
- Filtered view shows "4 of 37 commits" in status bar

## Job traceability

`job-find-commit` — resolves the primary workaround (grep) for "find a commit I vaguely remember"

## Story IDs

- US-06: Inline search bar with real-time filtering (message + repo, case-insensitive)

## Estimated effort

1–2 days
