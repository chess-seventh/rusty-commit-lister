# Story Map: rusty-commit-lister

## User: franci
## Goal: Orient myself about what I committed recently, without leaving the terminal

---

## Backbone (Activities — left to right, chronological)

| Configure | Load | Browse | Search | Inspect | Act | Exit |
|---|---|---|---|---|---|---|
| Read config file | Scan Obsidian vault | Navigate rows j/k | Activate / search bar | Open detail panel | Copy URL to clipboard | Press q/Esc |
| Resolve vault path | Parse daily notes | PgUp/PgDn by page | Filter by message+repo | View full path + URL | Open URL in browser | Terminal restored |
| Fall back to defaults | Walk date range | Truncate long messages | Esc clears query | Handle missing URL | | |
| | Show row count | r-refresh | Real-time match count | Return to same row | | |
| | Show empty state | | | | | |

---

### Walking Skeleton (Slice 01)
Minimum end-to-end path that proves the chain works:

| Configure | Load | Browse | — | — | — | Exit |
|---|---|---|---|---|---|---|
| Read config (vault_path + scan_days_back) | Parse ONE daily note | Render rows in table + j/k | — | — | — | q exits cleanly |

**Stories**: US-01 (config loader), US-02 (parser — single note), US-03 (minimal TUI render + exit)

**Walking skeleton validates**: Unicode path → OsString → Markdown table parser → ratatui render → clean exit

---

### Slice 02 — Full Browse Experience (Release 1)
Target outcome: franci uses tool daily as primary orientation; stops manual scanning

| Configure | Load | Browse | — | — | — | Exit |
|---|---|---|---|---|---|---|
| (slice 01) | Walk all notes in scan window | PgUp/PgDn + wrap | — | — | — | (slice 01) |
| | walkdir + chrono date filter | Truncation with … | | | | |
| | Show commit + repo counts | Row N/Total status bar | | | | |
| | | r-refresh | | | | |

**Stories**: US-04 (multi-day scan), US-05 (full navigation)

---

### Slice 03 — Search (Release 2)
Target outcome: franci finds a vaguely-remembered commit in < 30 seconds; never reaches for grep

| Configure | Load | Browse | Search | — | — | Exit |
|---|---|---|---|---|---|---|
| (prev) | (prev) | (prev) | Real-time message+repo filter | — | — | (prev) |
| | | | Case-insensitive / bar | | | |
| | | | Esc restores full list | | | |
| | | | Match count in status bar | | | |

**Stories**: US-06 (inline search)

---

### Slice 04 — Detail View (Release 3)
Target outcome: franci gets full path + URL in one keypress; stops opening Obsidian for URLs

| Configure | Load | Browse | Search | Inspect | — | Exit |
|---|---|---|---|---|---|---|
| (prev) | (prev) | (prev) | (prev) | Detail panel overlay | — | (prev) |
| | | | | Full message, path, URL | | |
| | | | | Esc → same row position | | |
| | | | | Missing URL graceful | | |

**Stories**: US-07 (detail panel)

---

### Slice 05 — Clipboard Copy (Release 4)
Target outcome: franci pastes a URL directly from the tool; zero app switches

| Configure | Load | Browse | Search | Inspect | Act | Exit |
|---|---|---|---|---|---|---|
| (prev) | (prev) | (prev) | (prev) | (prev) | c → copy URL | (prev) |
| | | | | | Confirmation shown | |
| | | | | | SSH fallback text | |

**Stories**: US-08 (clipboard copy)

---

### Slice 06 — Repo Filter (Release 5)
Target outcome: franci isolates one repo's commits for weekly review in two keypresses

| Configure | Load | Browse | Search | Inspect | Act | Exit |
|---|---|---|---|---|---|---|
| repo_filter in config | (prev) | (prev) | Composes with repo filter | (prev) | (prev) | (prev) |
| | | f → repo picker | Active filter in status bar | | | |
| | | Esc clears filter | | | | |

**Stories**: US-09 (repo filter)

---

## Priority Rationale

| Priority | Slice | Value | Urgency | Effort | Score | Rationale |
|---|---|---|---|---|---|---|
| 1 | Slice 01 — Walking Skeleton | 5 | 5 | 5 | **5.0** | Validates riskiest assumption; nothing else can build without it |
| 2 | Slice 02 — Full Browse | 5 | 5 | 3 | **8.3** | Completes primary job (job-orient); daily use begins here |
| 3 | Slice 03 — Search | 5 | 4 | 2 | **10.0** | Replaces grep workaround; OST score 12; high ROI, low effort |
| 4 | Slice 04 — Detail View | 4 | 3 | 1 | **12.0** | OST score 10; eliminates Obsidian app-switch; low effort |
| 5 | Slice 05 — Clipboard Copy | 3 | 3 | 1 | **9.0** | Completes inspect→act loop; trivial to add after detail view |
| 6 | Slice 06 — Repo Filter | 3 | 2 | 2 | **3.0** | OST score 9; useful for weekly review; composable with search |

> Score = (Value × Urgency) / Effort. Tie-breaking: Walking Skeleton > Riskiest Assumption > Highest Value.

## Scope Assessment

PASS — 9 stories, 3 bounded contexts (config/parsing, TUI rendering, clipboard/OS), estimated 8–10 days total.
Each slice delivers independently verifiable user-observable behavior. No slice requires another
slice's completion except the linear dependency chain (01 → 02 → 03+ is technical, not business).

## Out of Scope (Tier 2/3 — not in this story map)

- Date range filter (Tier 2, score 8) — can be added as slice 07 after slice 06
- Nix flake (Tier 2, score 8) — devops concern, not a user journey story
- Open URL in browser (Tier 3, score 7) — deferred; clipboard copy covers the primary need
- Live reload (Tier 3, score 4) — manual r sufficient
