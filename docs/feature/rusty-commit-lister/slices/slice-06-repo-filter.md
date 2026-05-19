# Slice 06 — Repository Filter

**Outcome**: Franci can isolate commits from one repo instantly — useful for weekly repo-activity reviews.

## Why this slice is sixth

Score 9 — satisfies the secondary job "see which repos I've been active in lately." The search
in slice 03 already handles free-text filtering, but a dedicated repo filter is faster for the
"show me only dotfiles commits" use case (no need to type the full match query). This slice also
enables the future "filter by repo name from config" capability.

## Scope

- `f` key (or `F`) in the main table opens a repo picker overlay listing all distinct repo names
  found in the current scan window with commit counts (e.g., "dotfiles (12)  rusty-commit-saver (8)")
- Selecting a repo filters the main table to show only commits from that repo
- Status bar shows filter active: "dotfiles • 12 of 37 commits"
- `Esc` or `f` again clears filter and restores full list
- Filter composes with search (slice 03): both can be active simultaneously
- Config-level `repo_filter` in config.toml is pre-applied at load time (separate from TUI filter)

## Acceptance criteria summary

- `f` opens repo picker with distinct repo names + commit counts
- Selecting "dotfiles" shows only dotfiles commits in the table
- Status bar confirms active filter: "dotfiles • 12 of 37 commits"
- `Esc` from filter clears it and restores full list
- Filter + search both active simultaneously: filtered list further narrowed by search query

## Job traceability

`job-review-repo-activity` — "see which repos I've been most active in lately"

## Story IDs

- US-09: Interactive repo filter picker with compose support (works alongside search)

## Estimated effort

1–2 days
