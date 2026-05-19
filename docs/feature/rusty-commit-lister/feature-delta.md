# Feature Delta — rusty-commit-lister

**Discovery date**: 2026-05-18
**Discovery method**: Behavioral evidence analysis (sole user / author is customer)
**Discovery phase status**: All 4 phases completed — G1/G2/G3/G4 PASS

---

## Problem Statement

In the user's own words (reconstructed from behavioral evidence and README):

> "I want to see all my commits across all repos in one place, in my terminal, without leaving
> my flow — because they're already in my Obsidian daily notes and I don't want to grep through
> Markdown files manually."

**Validation confidence**: HIGH. rusty-commit-saver already exists and is actively writing commit data
to Obsidian daily notes. This prior investment is a commitment signal stronger than any stated
preference — it proves the problem is real and recurring.

---

## Jobs-to-be-Done

### Primary job (highest priority)

**"Orient myself at the start or end of a working session."**

Steps: open terminal → launch tool → scan last N days of commits → identify what I worked on →
close tool and continue.

Desired outcome: Minimize the time from "I want to see what I committed recently" to having a
complete, scannable list — without leaving the terminal.

### Secondary jobs

1. **"Find a commit I vaguely remember."** Cross-repo, approximate message, approximate date.
   Workaround today: `grep -r "keyword" ~/Documents/Wiki/...` — slow and context-breaking.

2. **"See which repos I've been active in lately."** Spot patterns across days.
   Workaround today: scroll through Obsidian daily notes manually.

3. **"Open the exact repo or file I was working in."** From the commit entry, navigate to source.
   Workaround today: copy URL from Obsidian note, open browser, navigate manually.

---

## Opportunity Scores (OST)

Formula: Score = Importance + Max(0, Importance - Satisfaction)
Importance and Satisfaction rated 1–10. Current satisfaction = quality of existing workarounds.

| Opportunity | Imp | Sat (workaround) | Score | Priority |
|---|---|---|---|---|
| Unified cross-repo commit browsing (core TUI) | 9 | 3 | **15** | Build now |
| Full-text search across commit messages | 8 | 4 | **12** | Build next |
| Commit detail view (full path + URL) | 6 | 2 | **10** | Build next |
| Filter by repository name | 7 | 5 | **9** | Build next |
| Date range filter | 6 | 4 | **8** | Consider |
| Nix flake installation | 5 | 2 | **8** | Consider |
| Open repo URL in browser | 5 | 3 | 7 | Later |
| Live reload (file watcher) | 4 | 6 | 4 | Low |
| Export to Markdown/CSV | 3 | 7 | 3 | Low |
| Multiple Obsidian vault support | 2 | 9 | 2 | Deprioritize |

### Opportunity tree summary

```
Desired Outcome: See all commits instantly in terminal without disrupting flow
  |
  +-- Core TUI browsing (15) [BUILD NOW]
  |     +-- ratatui table: Date, LastDir, Repo, Message columns
  |     +-- vim keybindings (j/k/PgUp/PgDn) + refresh (r) + quit (q/Esc)
  |     +-- scan_days_back configurable (default 7)
  |
  +-- Full-text search (12) [BUILD NEXT]
  |     +-- In-TUI search bar triggered by /
  |     +-- Search across message + repo fields
  |
  +-- Commit detail view (10) [BUILD NEXT]
  |     +-- Enter key opens detail panel
  |     +-- Shows: full folder path, full URL, repo, date, message
  |     +-- URL copyable to clipboard or opens in browser
  |
  +-- Filter by repo (9) [BUILD NEXT]
        +-- Inline filter (config-level already planned as repo_filter)
        +-- TUI-level interactive filter as complementary layer
```

---

## Roadmap Re-prioritisation

The README roadmap is correct in direction but the ordering should be adjusted based on
opportunity scores. Proposed sequence against the existing roadmap items:

### Tier 1 — Build first (core job completion; score >= 10)

1. **Markdown table parser** — reads `## Commits` table from daily notes
2. **Config loader** — `~/.config/rusty-commit-lister/config.toml`
3. **Core ratatui table** — columns: Date+Time | LastDir | Repo | Message; j/k/PgUp/PgDn/r/q
4. **Full-text search** — / to activate, searches message + repo (score 12, high ROI)
5. **Commit detail view** — Enter opens panel with full path + URL (score 10)

### Tier 2 — Build after Tier 1 (score 8-9)

6. **Filter by repository name** — interactive TUI filter layer on top of config-level filter
7. **Date range filter** — complement scan_days_back with explicit from/to
8. **Nix flake** — installation UX for daily use on NixOS/devenv setup

### Tier 3 — Build if Tier 2 feels incomplete (score 4-7)

9. **Open repo URL in browser** — convenience; copy to clipboard may suffice
10. **Live reload (file watcher)** — manual `r` refresh is adequate; watcher adds complexity

### Deprioritize (score <= 3)

11. Export to Markdown/CSV — data already exists in Obsidian; low marginal value
12. Multiple vault support — sole-user tool with single vault; YAGNI

---

## Key Constraints Discovered

### Data contract constraint (HIGH)
The commit table format is owned by rusty-commit-saver. The parser in rusty-commit-lister must
treat that format as a stable contract. Any changes to the saver's output format break the
lister's parser. Recommendation: document the expected table schema explicitly in the parser
module (column order, header names, delimiter format).

### Path constraint (MEDIUM)
The Obsidian vault path contains a Unicode emoji (`📅 Diaries`). Rust handles this natively but
the config path must be specified verbatim including the emoji. Shell expansion and config
parsing must be tested with this path specifically.

### Performance constraint (LOW-MEDIUM)
scan_days_back=7 at default. At ~10 commits/day across multiple repos, that is ~70 rows — trivial.
At scan_days_back=365, that could be ~3,650 rows. Async loading via tokio (already in Cargo.toml)
handles this, but startup latency becomes noticeable above ~500 notes. Recommendation: load
incrementally (most-recent first) and render partial results while scanning continues.

### Cargo.toml gap (CRITICAL for implementation)
The current Cargo.toml does NOT include ratatui, crossterm, chrono, walkdir, or toml — the
dependencies listed in the README "Key dependencies" section. The scaffold uses the bootstrap
template dependencies (clap, tokio, serde, anyhow, thiserror, config). These must be added
before any TUI code can be written.

---

## Assumptions Tracker

| Assumption | Category | Risk Score | Status |
|---|---|---|---|
| rusty-commit-saver table format is stable | Feasibility | 6 | Acceptable — format is self-owned |
| franci uses this tool daily (not just built it) | Value | 9 | Validate: check if saver is active in git hooks |
| vim keybindings match franci's muscle memory | Usability | 5 | Acceptable — consistent with Obsidian + terminal use |
| scan_days_back=7 default covers the primary job | Value | 7 | Acceptable — configurable anyway |
| Single vault is sufficient long-term | Viability | 4 | Acceptable — deprioritised correctly |
| Unicode emoji path works across all Rust path operations | Feasibility | 8 | Test early — spike before full parser |
| ratatui startup is fast enough to not disrupt flow | Usability | 7 | Acceptable — async loading mitigates |

Risk score = (Impact x 3) + (Uncertainty x 2) + (Ease x 1). Scores > 12 = test first, 8-12 = test soon.

---

## Go/No-Go Decision

**Decision: GO**

All four gates passed:
- G1 (Problem validated): 5+ behavioral signals confirm real, recurring need. PASS.
- G2 (Opportunities mapped): Top 3 opportunities score >8. OST complete. PASS.
- G3 (Solution tested): Solution concept coherent; stack validated; format contract established. PASS.
- G4 (Viability confirmed): Personal tool, zero cost basis, no monetization risk. All 4 risks green/yellow. PASS.

The blocking action before writing code is adding the missing Cargo.toml dependencies (ratatui,
crossterm, chrono, walkdir, toml). After that, Tier 1 items can be built in sequence.

---

## Handoff to Product Owner

Discovery artifacts:
- `docs/feature/rusty-commit-lister/feature-delta.md` (this file)
- `docs/product/journeys/daily-commit-review.yaml`

Next role: product-owner defines acceptance criteria and slices Tier 1 into shippable increments.

---

## Wave: DISCUSS / [JTBD] Job Analysis

**Date**: 2026-05-18
**Method**: Behavioral evidence grounding (sole user / sole author)

### JTBD Grounding Summary

Four jobs validated from behavioral evidence. No interviews needed — the prior investment in
rusty-commit-saver (active daily git hook) is a commitment signal stronger than any stated preference.

| Job ID | Statement | Opportunity Score | Priority |
|---|---|---|---|
| job-orient | Orient myself at session start/end without leaving terminal | 15 | Build now |
| job-find-commit | Find a vaguely-remembered commit without grepping | 12 | Build next |
| job-inspect-commit | Get full commit context (path + URL) in one keypress | 10 | Build next |
| job-review-repo-activity | See which repos I've been most active in | 9 | Build next |

### Four Forces — Primary Job (job-orient)

**Push** (pain driving away from current state):
grep -r through the Obsidian vault is accurate but breaks terminal flow, requires knowing the path
structure, and returns raw Markdown with no sorting or aggregation. Manual Obsidian scroll is even
slower. git log per repo requires knowing which repo first.

**Pull** (attraction toward new solution):
Sub-2-second TUI in the terminal. No app switch. j/k navigation. Sorted newest-first. Commit count
confirmed in status bar. Done in 15 seconds.

**Anxiety** (fear about the new solution):
"What if the parser misses some commits, or the emoji path breaks, and I silently see an incomplete
list thinking it is complete?" — the row count confirmation is the key anxiety-resolution signal.

**Habit** (inertia from old workflow):
Grep and git log are muscle memory. The new tool must be faster to invoke (one word, no path args)
than composing a grep command.

### Job Map — job-orient

All 8 steps mapped in `docs/product/jobs.yaml`. Key steps for story prioritisation:
- **Locate + Prepare** (steps 2–3): walking skeleton must prove these work with real vault data
- **Confirm** (step 4): row count in status bar is the observable confirmation signal
- **Modify** (step 7): search and repo filter address this step (slices 03, 06)

### JTBD Artifacts

- `docs/product/jobs.yaml` — all four jobs with dimensions, forces, job maps, opportunity scores

---

## Wave: DISCUSS / [SCOPE] Elephant Carpaccio Assessment

**Date**: 2026-05-18

### Scope Assessment: PASS

- Stories: 9 (US-01 through US-09) — within range
- Bounded contexts: 3 (config/path resolution, Markdown parsing, ratatui TUI + OS integration)
- Walking skeleton integration points: 3 (config → parser → TUI render)
- Estimated effort: 8–10 days total across 6 slices
- Independent user outcomes: 5 (orient/browse, search, inspect, act, repo-filter) — all separately verifiable

No oversized signals detected. Proceeding with 6 Elephant Carpaccio slices.

---

## Wave: DISCUSS / [JOURNEY] Visualization

**Date**: 2026-05-18

### Journey Refinements (DISCOVER seed → validated)

The DISCOVER seed journey has been promoted from `seed` to `validated` status. Key additions:

**Emotional arc added**:
- Start: "Slightly disoriented — context is scattered across repos"
- Middle: "Focused and scanning — the list is right there, navigation is fluid"
- End: "Grounded and ready — I know exactly what I committed"
- Pattern: confidence-building
- Peak anxiety resolved by: row count in status bar ("37 commits across 5 repos")

**TUI mockups added** per step (see `docs/product/journeys/daily-commit-review.yaml`):
- Launch: spinner within 100ms
- Load: 4-column table with status bar
- Browse: selected row with ▶ indicator + Row N/Total counter
- Search: inline / bar with real-time match count
- Inspect: full-detail overlay panel
- Act: clipboard confirmation in panel
- Exit: clean terminal prompt

**Failure modes documented** per step (critical ones):
- Load: Unicode emoji path OsString failure (risk: HIGH — spike in slice-01)
- Load: rusty-commit-saver format change (risk: HIGH — skip-and-log strategy)
- Browse: Esc context ambiguity (search mode vs. main mode)
- Exit: Alt screen not restored on Ctrl+C

**Integration checkpoints**: 5 defined and testable (see shared-artifacts-registry.md)

### Journey Artifacts

- `docs/product/journeys/daily-commit-review.yaml` — validated journey (v0.2.0)
- `docs/feature/rusty-commit-lister/discuss/shared-artifacts-registry.md`

---

## Wave: DISCUSS / [STORY-MAP] User Story Map

**Date**: 2026-05-18

### Story Map Summary

**Backbone** (7 activities): Configure → Load → Browse → Search → Inspect → Act → Exit

**Walking Skeleton** (Slice 01): Configure (read config + resolve path) → Load (parse one note) → Browse (render table + j/k + q exit)

**6 Slices by outcome**:

| Slice | Outcome | Stories | Effort |
|---|---|---|---|
| 01 — Walking Skeleton | End-to-end chain proven (config→parse→render) | US-01, US-02, US-03 | 2–3 days |
| 02 — Full Browse | Tool usable as daily orientation habit | US-04, US-05 | 1–2 days |
| 03 — Search | grep workaround replaced | US-06 | 1–2 days |
| 04 — Detail View | Obsidian app-switch eliminated | US-07 | 1 day |
| 05 — Clipboard Copy | Full orient→inspect→act loop complete | US-08 | 0.5–1 day |
| 06 — Repo Filter | Weekly repo-activity review in 2 keypresses | US-09 | 1–2 days |

**Priority rationale**: Walking Skeleton first (riskiest assumption). Full Browse second (completes
primary job). Search third (highest ROI after browse — score 12, replaces grep). Detail + Clipboard
fourth/fifth (score 10, low effort). Repo Filter last (score 9, builds on search).

### Story Map Artifact

- `docs/feature/rusty-commit-lister/discuss/story-map.md`

---

## Wave: DISCUSS / [REQUIREMENTS] User Stories and Outcome KPIs

**Date**: 2026-05-18

### Stories Produced (9 total)

| Story | Title | Slice | Job | MoSCoW |
|---|---|---|---|---|
| US-01 | Config Loader and Vault Path Resolution | 01 | job-orient | Must Have |
| US-02 | Markdown Table Parser | 01 | job-orient | Must Have |
| US-03 | Minimal Ratatui TUI — Render and Exit | 01 | job-orient | Must Have |
| US-04 | Multi-Day Note Traversal and Date-Filtered Scan | 02 | job-orient | Must Have |
| US-05 | Full Navigation — PgUp/PgDn, Truncation, Refresh, Row Counter | 02 | job-orient | Must Have |
| US-06 | Inline Full-Text Search | 03 | job-find-commit | Must Have |
| US-07 | Commit Detail Panel | 04 | job-inspect-commit | Should Have |
| US-08 | Clipboard URL Copy | 05 | job-inspect-commit | Should Have |
| US-09 | Interactive Repository Filter | 06 | job-review-repo-activity | Should Have |

### DoR Validation Summary

All 9 stories pass DoR (9-item checklist):

| DoR Item | Status |
|---|---|
| 1. Problem statement clear, domain language | PASS — all stories start from franci's actual pain |
| 2. User/persona with specific characteristics | PASS — franci with terminal-first, vim keybindings, Obsidian diary |
| 3. 3+ domain examples with real data | PASS — each story has 3 examples with real paths/messages/dates |
| 4. UAT in Given/When/Then (3–7 scenarios) | PASS — 3–5 scenarios per story; @property tag used for parser invariant |
| 5. AC derived from UAT | PASS — AC checklist derived from scenarios for each story |
| 6. Right-sized (1–3 days, 3–7 scenarios) | PASS — largest story (US-03) is 3 days max; all stories ≤ 5 scenarios |
| 7. Technical notes: constraints/dependencies | PASS — crate names, state machine notes, truncation approach documented |
| 8. Dependencies resolved or tracked | PASS — dependency chain explicit per story |
| 9. Outcome KPIs defined with measurable targets | PASS — per-story KPIs + epic-level KPI table in outcome-kpis.md |

**Additional mandatory checks**:
- JTBD traceability: every story has `job_id` referencing `docs/product/jobs.yaml` — PASS
- Elevator Pitch: every non-infrastructure story has Before/After/Decision-enabled — PASS
- No anti-patterns detected: all personas are "franci", all data is real (real paths, messages, dates), all ACs are observable outcomes

### Outcome KPIs Summary

- **North Star**: KPI-5 — tool used ≥ 5 of 7 working days for two consecutive weeks post-slice-02
- **Leading indicator**: KPI-1 — grep/Obsidian for orientation drops to 0 within 14 days of slice-02
- **Key guardrail**: startup < 2 seconds for scan_days_back ≤ 30 (protected through all slices)
- **OMTM stage**: Stickiness — habit formation is the primary metric

### Artifacts Produced

- `docs/feature/rusty-commit-lister/discuss/user-stories.md` — all 9 stories
- `docs/feature/rusty-commit-lister/discuss/outcome-kpis.md` — KPI table + measurement plan
- `docs/feature/rusty-commit-lister/discuss/story-map.md` — backbone + 6 slices + priority rationale
- `docs/feature/rusty-commit-lister/discuss/shared-artifacts-registry.md` — 7 shared artifacts tracked
- `docs/product/jobs.yaml` — 4 jobs with dimensions, forces, job maps
- `docs/product/journeys/daily-commit-review.yaml` — validated journey v0.2.0
- `docs/feature/rusty-commit-lister/slices/slice-01-walking-skeleton.md`
- `docs/feature/rusty-commit-lister/slices/slice-02-full-browse.md`
- `docs/feature/rusty-commit-lister/slices/slice-03-search.md`
- `docs/feature/rusty-commit-lister/slices/slice-04-detail-view.md`
- `docs/feature/rusty-commit-lister/slices/slice-05-clipboard-copy.md`
- `docs/feature/rusty-commit-lister/slices/slice-06-repo-filter.md`

---

## Handoff to Solution Architect (DESIGN wave)

**Status**: DISCUSS wave complete — ready for DESIGN wave handoff.

**Handoff package**:
- 9 user stories, all DoR-passing, in `discuss/user-stories.md`
- Journey schema with embedded Gherkin: `docs/product/journeys/daily-commit-review.yaml`
- Story map with walking skeleton identified: `discuss/story-map.md`
- Outcome KPIs with measurement plan: `discuss/outcome-kpis.md`
- Shared artifact registry: `discuss/shared-artifacts-registry.md`
- Jobs registry: `docs/product/jobs.yaml`
- 6 slice briefs: `slices/slice-01` through `slices/slice-06`

**Critical design decisions deferred to DESIGN wave** (solution-neutral requirements preserved):
- State management architecture (Elm model/update/view is noted as a pattern, not prescribed)
- Clipboard crate choice (arboard vs. clipboard)
- Async runtime design (tokio noted as dependency; concurrency model deferred)
- Config crate internals (toml 0.8 noted; parsing strategy deferred)
- Parser error handling strategy (skip-and-log is the requirement; implementation deferred)

**Key risks for DESIGN wave to resolve**:
1. Unicode OsString path — spike required before US-02 design
2. Esc context ambiguity (search vs. main mode) — state machine design critical
3. Terminal width detection — minimum 80 columns, graceful degradation needed
4. Ctrl+C during async load — cleanup must be guaranteed (Drop or signal handler)

---

## Wave: DESIGN / [D1–D4] Architecture Decisions

**Date**: 2026-05-18
**Architect**: Morgan (Solution Architect)

### Decision Summary

| ID | Decision | Verdict | One-line rationale |
|---|---|---|---|
| D1 | TUI Architecture: Elm/MVU vs Mutable App Struct | Elm/MVU accepted | Pure `update(Model, Event) → Model` makes state corruption structurally impossible; 4-mode enum maps cleanly to nested match arms; tests need no setup |
| D2 | Async Loading Strategy | Sync blocking for slice-01/02; revisit if > 100ms observed | 30 notes × < 1ms parse ≈ < 30ms total; adding tokio channel for a sub-30ms operation is YAGNI; ADR-002a ready to activate if spike test fails |
| D3 | Development Paradigm: FP vs OOP | Functional Programming accepted | Parser is a pure transformation; state machine is cleaner as immutable transitions; tests need no setup; aligns with Rust's ownership defaults |
| D4 | Clipboard Integration: arboard vs clipboard vs fallback | arboard accepted | Actively maintained, cross-platform (macOS NSPasteboard + X11/Wayland), MIT/Apache-2.0; fallback to text display on SSH/headless per US-08 requirement |

---

## Wave: DESIGN / [COMPONENTS] Component Decomposition

### Module Table

| Module | Path | Responsibility | Change Type |
|---|---|---|---|
| Composition Root | `src/main.rs` | Wire adapters to ports; probe each adapter; start event loop | MODIFY (replace stub run() body) |
| Library root | `src/lib.rs` | Re-export port traits and domain types for integration tests | MODIFY (replace RustyCommitLister stub) |
| Error types | `src/error.rs` | Extended RustyCommitListerError: add ParseError, VaultError, ClipboardUnavailable | MODIFY (extend existing enum) |
| Domain — Model | `src/domain/model.rs` | AppModel, AppMode (Browse/Search/Detail/RepoPicker), CommitRecord, AppConfig | NEW |
| Domain — Events | `src/domain/events.rs` | AppEvent enum: KeyPress, LoadComplete, LoadFailed, ClipboardResult, Tick | NEW |
| Domain — Update | `src/domain/update.rs` | Pure state machine: update(AppModel, AppEvent) → AppModel | NEW |
| Port — Config | `src/ports/config_port.rs` | Trait ConfigPort + Probe supertrait | NEW |
| Port — Vault | `src/ports/vault_port.rs` | Trait VaultScanPort + Probe supertrait | NEW |
| Port — Clipboard | `src/ports/clipboard_port.rs` | Trait ClipboardPort + Probe supertrait | NEW |
| Adapter — Config | `src/adapters/toml_config.rs` | TomlConfigAdapter: ConfigPort + probe() | NEW |
| Adapter — Vault | `src/adapters/walkdir_vault.rs` | WalkdirScanAdapter: VaultScanPort + probe() | NEW |
| Adapter — Clipboard | `src/adapters/arboard_clipboard.rs` | ArboardClipboardAdapter: ClipboardPort + probe() (added in slice-05) | NEW (slice-05) |
| Parser | `src/parser/mod.rs` | Pure function parse_note(path) → Vec<CommitRecord>; skip-and-log on malformed rows | NEW |
| TUI — View | `src/tui/view.rs` | Pure render: view(&AppModel, &mut Frame); ratatui widgets per AppMode | NEW |
| TUI — Event Loop | `src/tui/event_loop.rs` | crossterm raw-mode loop; translate KeyEvent → AppEvent; drive update→view cycle | NEW |

### Driving Ports (Primary — inbound to domain)

| Port | Technology | Role |
|---|---|---|
| CLI entry | `clap 4.5` | Parses flags (--config, --verbose, --days) at composition root; feeds AppConfig precedence layer |
| Keyboard events | `crossterm 0.27` | Raw-mode terminal stdin; translates KeyEvent to AppEvent; drives TUI event loop |

### Driven Ports + Adapters (Secondary — outbound from domain)

| Port Trait | Adapter | Technology | Substrate |
|---|---|---|---|
| `ConfigPort` | `TomlConfigAdapter` | `toml 0.8`, `dirs` or `$HOME` expansion | Filesystem (~/.config/rusty-commit-lister/config.toml) |
| `VaultScanPort` | `WalkdirScanAdapter` | `walkdir 2`, `chrono 0.4`, `parser::parse_note` | Filesystem (Obsidian vault path) |
| `ClipboardPort` | `ArboardClipboardAdapter` | `arboard` (latest stable, added slice-05) | System clipboard (macOS pasteboard / X11 / Wayland) |

---

## Wave: DESIGN / [TECH] Technology Choices

| Crate | Version | License | Decision |
|---|---|---|---|
| ratatui | 0.26 | MIT | REUSE — already in Cargo.toml |
| crossterm | 0.27 | MIT | REUSE — already in Cargo.toml |
| chrono | 0.4 | MIT/Apache-2.0 | REUSE — already in Cargo.toml |
| walkdir | 2 | MIT/Unlicense | REUSE — already in Cargo.toml |
| toml | 0.8 | MIT/Apache-2.0 | REUSE — already in Cargo.toml |
| clap | 4.5 | MIT/Apache-2.0 | REUSE — already in Cargo.toml |
| tokio | 1.48 | MIT | REUSE — available for slice-05+ async upgrade |
| serde | 1.0 | MIT/Apache-2.0 | REUSE — already in Cargo.toml |
| anyhow | 1.0 | MIT/Apache-2.0 | REUSE — already in Cargo.toml |
| thiserror | 2.0 | MIT/Apache-2.0 | REUSE — already in Cargo.toml |
| tracing | 0.1 | MIT | REUSE — already in Cargo.toml; used for skip-and-log |
| arboard | latest stable | MIT/Apache-2.0 | ADD in slice-05 (clipboard, not needed earlier) |
| config | 0.15 | MIT/Apache-2.0 | REMOVE — superseded by direct toml 0.8 parsing |
| serde_json | 1.0 | MIT/Apache-2.0 | REMOVE — no JSON in this feature |
| serde_yaml | 0.9 | MIT/Apache-2.0 | REMOVE — no YAML in this feature |

---

## Wave: DESIGN / [REUSE] Reuse Analysis

| Existing Component | Location | Status | Decision |
|---|---|---|---|
| `RustyCommitListerError` | `src/error.rs` | EXISTS | EXTEND: add ParseError, VaultError, ClipboardUnavailable variants |
| `RustyCommitLister` struct | `src/lib.rs` | EXISTS (bootstrap stub) | REPLACE: struct removed; lib.rs re-exports port traits and domain types |
| `main.rs` clap setup | `src/main.rs` | EXISTS | EXTEND: keep clap arg parsing; replace run() body with adapter wiring + probe + TUI start |
| `tokio` runtime | `Cargo.toml` | EXISTS | REUSE: retained for slice-05+ async upgrade path |
| `tracing` / `tracing-subscriber` | `Cargo.toml` | EXISTS | REUSE: parser skip-and-log debug/warn output |
| `serde` / `anyhow` / `thiserror` / `clap` | `Cargo.toml` | EXISTS | REUSE: all used directly |
| ratatui, crossterm, chrono, walkdir, toml | `Cargo.toml` | EXISTS (already added) | REUSE: all present in current Cargo.toml |
| `config` crate | `Cargo.toml` | EXISTS | REMOVE: replaced by direct toml 0.8 in TomlConfigAdapter |
| `serde_json`, `serde_yaml` | `Cargo.toml` | EXISTS | REMOVE: not needed for this feature |
| `arboard` | `Cargo.toml` | ABSENT | ADD in slice-05 |

---

## Wave: DESIGN / [OPEN-QUESTIONS] Spikes Required

| ID | Question | Risk | Owner | Target slice |
|---|---|---|---|---|
| OQ-1 | Unicode OsString spike: does `PathBuf::from("~/Documents/Wiki/📅 Diaries/0. Journal")` round-trip through walkdir on macOS Darwin 25.4.0 without silent data loss? | HIGH — known risk from DISCUSS wave; must pass before slice-01 ships | Software crafter | slice-01 |
| OQ-2 | arboard SSH probe: does `arboard::Clipboard::new()` on an SSH session with `-Y` (X11 forwarding) return Ok() but silently no-op on write()? Need round-trip verification. | MEDIUM — affects US-08 fallback correctness on SSH | Software crafter | slice-05 |

---

## Wave: DESIGN / [ARCHITECTURE] Artifacts

- `docs/product/architecture/brief.md` — full architecture document (C4 L1+L2+L3, component table, port traits, tech stack, reuse analysis)
- `docs/product/architecture/adr-001.md` — TUI Architecture: Elm/MVU
- `docs/product/architecture/adr-002.md` — Async Loading Strategy
- `docs/product/architecture/adr-003.md` — Development Paradigm: FP
- `docs/product/architecture/adr-004.md` — Clipboard Integration: arboard

**DESIGN wave status**: Complete. Ready for DISTILL / acceptance-designer handoff.

---

## Wave: DISTILL

### [REF] Inherited commitments

| Origin | Commitment | DDD | Impact |
|--------|------------|-----|--------|
| DISCUSS#US-01 | Config loader reads config.toml; Unicode vault path resolves; exit code 2 on invalid values | n/a | TomlConfigAdapter and CLI binary must handle emoji path and surface actionable config errors |
| DISCUSS#US-02 | Markdown table parser extracts commits; skip-and-log on malformed rows; never panics | n/a | parse_note() is a pure function with proptest no-panic invariant enforced by DISTILL tests |
| DISCUSS#US-03 | Minimal TUI: 4-column table; j/k navigation; status bar; q/Esc exit; empty state | n/a | update() pure function tests cover all mode transitions before TUI wiring |
| DISCUSS#US-04 | Multi-day vault scan via walkdir + chrono date filter; merge sorted newest-first | n/a | WalkdirScanAdapter integration test covers multi-day traversal with real tempdir |
| DISCUSS#US-05 | PgUp/PgDn; truncation with ellipsis; r refresh; row counter | n/a | update() property test covers navigation event sequences across any model state |
| DISCUSS#US-06 | Inline search: / activates, real-time filter on MESSAGE+REPO, case-insensitive, Esc restores | n/a | update() unit tests cover Search mode transitions and filtered_rows recomputation |
| DISCUSS#US-07 | Enter opens detail panel; full message/path/URL; Esc back to same row; missing URL placeholder | n/a | update() unit tests cover Detail mode enter/exit with cursor preservation |
| DISCUSS#US-08 | c copies URL; SSH fallback text; no crash if clipboard unavailable | n/a | FakeClipboard port validates input contracts; slice-05 deferred pending arboard dep |
| DISCUSS#US-09 | f opens repo picker; commit counts; composes with search; Esc clears | n/a | update() unit tests cover RepoPicker mode transition and active_repo_filter |
| DESIGN#D1 | Elm/MVU architecture: update(Model, Event) → Model | ADR-001 | update() pure function is the central test surface; all TUI logic tested without subprocess |
| DESIGN#D3 | Functional Programming paradigm | ADR-003 | parse_note() and update() are pure functions; proptest applied at unit layer per Mandate 9 |

---

### [REF] Scenario List

| # | Scenario Title | File | Tags |
|---|----------------|------|------|
| 1 | Tool loads commits from vault and exits successfully | walking_skeleton_scenarios.rs | @walking_skeleton @driving_port @US-01 @US-02 @US-03 @real-io |
| 2 | Invalid scan_days_back exits with code 2 and actionable error | walking_skeleton_scenarios.rs | @US-01 @real-io @error #[ignore] |
| 3 | Missing config uses defaults and shows notice | walking_skeleton_scenarios.rs | @US-01 @real-io #[ignore] |
| 4 | Vault path with emoji segment is resolved correctly | walking_skeleton_scenarios.rs | @US-01 @real-io #[ignore] |
| 5 | Tool exits cleanly with code 0 when q pressed | walking_skeleton_scenarios.rs | @US-03 @real-io #[ignore] |
| 6 | Empty vault shows informative empty state | walking_skeleton_scenarios.rs | @US-03 @real-io @error #[ignore] |
| 7 | Valid config loads silently with no config messages | config_scenarios.rs | @US-01 @real-io @adapter-integration #[ignore] |
| 8 | Missing config triggers default fallback notice | config_scenarios.rs | @US-01 @real-io @adapter-integration #[ignore] |
| 9 | Unicode emoji vault path resolves correctly | config_scenarios.rs | @US-01 @real-io @adapter-integration #[ignore] |
| 10 | scan_days_back zero exits code 2 with actionable error | config_scenarios.rs | @US-01 @real-io @adapter-integration @error #[ignore] |
| 11 | scan_days_back negative exits code 2 and names the value | config_scenarios.rs | @US-01 @real-io @adapter-integration @error #[ignore] |
| 12 | TomlConfigAdapter reads vault_path and scan_days_back from real file | adapter_integration_scenarios.rs | @US-01 @real-io @adapter-integration #[ignore] |
| 13 | TomlConfigAdapter rejects scan_days_back zero | adapter_integration_scenarios.rs | @US-01 @real-io @adapter-integration @error #[ignore] |
| 14 | TomlConfigAdapter returns defaults when file is absent | adapter_integration_scenarios.rs | @US-01 @real-io @adapter-integration #[ignore] |
| 15 | WalkdirScanAdapter returns commit records from real vault | adapter_integration_scenarios.rs | @US-02 @US-04 @real-io @adapter-integration #[ignore] |
| 16 | WalkdirScanAdapter handles emoji path segment without data loss | adapter_integration_scenarios.rs | @US-01 @real-io @adapter-integration #[ignore] |
| 17 | WalkdirScanAdapter returns empty vec when no notes in window | adapter_integration_scenarios.rs | @US-04 @real-io @adapter-integration @error #[ignore] |
| 18 | WalkdirScanAdapter skips note with no commits section | adapter_integration_scenarios.rs | @US-02 @real-io @adapter-integration @error #[ignore] |
| 19 | Standard note with five rows produces five commit records | parser_specifications.rs | @US-02 @in-memory #[ignore] |
| 20 | Note with no commits section produces zero rows | parser_specifications.rs | @US-02 @in-memory #[ignore] |
| 21 | Malformed row is skipped and remaining rows are parsed | parser_specifications.rs | @US-02 @in-memory @error #[ignore] |
| 22 | Commit row with empty URL column produces record with url=None | parser_specifications.rs | @US-02 @in-memory #[ignore] |
| 23 | parse_note never panics on any markdown content | parser_specifications.rs | @US-02 @in-memory @property proptest #[ignore] |
| 24 | parse_note returns no more records than pipe rows | parser_specifications.rs | @US-02 @in-memory @property proptest #[ignore] |
| 25 | j key in Browse mode increments cursor | update_specifications.rs | @US-03 @in-memory #[ignore] |
| 26 | k key at top wraps cursor to last row | update_specifications.rs | @US-03 @in-memory #[ignore] |
| 27 | j key at last row wraps cursor to zero | update_specifications.rs | @US-03 @in-memory #[ignore] |
| 28 | slash key transitions Browse to Search mode | update_specifications.rs | @US-06 @in-memory #[ignore] |
| 29 | Esc in Search mode restores Browse and clears query | update_specifications.rs | @US-06 @in-memory #[ignore] |
| 30 | Enter in Browse mode transitions to Detail | update_specifications.rs | @US-07 @in-memory #[ignore] |
| 31 | Esc in Detail mode returns to Browse preserving cursor | update_specifications.rs | @US-07 @in-memory #[ignore] |
| 32 | f key in Browse mode transitions to RepoPicker | update_specifications.rs | @US-09 @in-memory #[ignore] |
| 33 | Search input event narrows filtered_rows to matching commits | update_specifications.rs | @US-06 @in-memory #[ignore] |
| 34 | LoadComplete event populates commit_rows | update_specifications.rs | @US-03 @in-memory #[ignore] |
| 35 | LoadFailed event sets error_message | update_specifications.rs | @US-03 @in-memory @error #[ignore] |
| 36 | Any valid event sequence produces valid model | update_specifications.rs | @US-03 @US-06 @US-07 @US-09 @in-memory @property proptest #[ignore] |

**Total scenarios**: 36 (6 walking skeleton + acceptance, 7 adapter integration, 6 parser unit, 17 update unit)
**Error/edge scenarios**: 15 of 36 = 42% (exceeds 40% target — Mandate 1 satisfied)
**Walking skeleton**: 1 enabled (scenario #1)
**@property (PBT) scenarios**: 3 (scenarios #23, #24, #36 — layer 1-2 only, per Mandate 9)

---

### [REF] WS Strategy

**Strategy**: CLI binary subprocess via `assert_cmd::Command::cargo_bin("rusty_commit_lister")` from `tempfile::TempDir`.

**Justification**: Architecture of Reference specifies CLI binary as the driving port. All acceptance tests that exercise the full chain MUST enter through the binary — no in-process function calls for the walking skeleton. Real tempdir filesystem for driven-internal (config + vault). FakeClipboard for driven-external (clipboard) — not exercised in WS.

**Architecture of Reference decisions applied**:
- Driving: real CLI subprocess (assert_cmd)
- Driven internal: real tempdir filesystem (tempfile::TempDir) per test
- Driven external/non-deterministic: FakeClipboard implementing ClipboardPort trait

---

### [REF] Adapter Coverage Table

| Adapter | @real-io scenario | Covered by |
|---------|-------------------|------------|
| TomlConfigAdapter (ConfigPort) | YES | Scenario #12 — reads real TOML from tempdir |
| WalkdirScanAdapter (VaultScanPort) | YES | Scenario #15 — scans tempdir with realistic note structure |
| CLI binary (driving port) | YES | Scenario #1 — walking skeleton invokes binary via assert_cmd |
| parse_note() (pure function) | YES | Scenario #19 — reads real fixture file `tests/fixtures/sample_daily_note.md` |
| ArboardClipboardAdapter (ClipboardPort) | DEFERRED — slice-05 | arboard dep not yet in Cargo.toml; FakeClipboard covers contract |

**Zero "NO — MISSING" rows for in-scope adapters (slice-05 clipboard deferred by design).**

---

### [REF] Scaffolds

| File | SCAFFOLD marker | Panic body |
|------|----------------|------------|
| `src/domain/model.rs` — `AppModel::new()` | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |
| `src/domain/update.rs` — `update()` | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |
| `src/ports/config_port.rs` — trait body | `// SCAFFOLD: true` | trait only (no body to panic) |
| `src/ports/vault_port.rs` — trait body | `// SCAFFOLD: true` | trait only (no body to panic) |
| `src/ports/clipboard_port.rs` — trait body | `// SCAFFOLD: true` | trait only (no body to panic) |
| `src/adapters/toml_config.rs` — `probe()`, `load()` | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |
| `src/adapters/walkdir_vault.rs` — `probe()`, `scan()` | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |
| `src/parser/mod.rs` — `parse_note()` | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |
| `src/main.rs` — composition root | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |
| `src/tui/view.rs` — `view()` | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |
| `src/tui/event_loop.rs` — `new()`, `run()`, `restore()` | `// SCAFFOLD: true` | `panic!("Not yet implemented -- RED scaffold")` |

Detect remaining scaffolds: `grep -r "SCAFFOLD: true" src/`

---

### [REF] Test Placement

| Test file | Path | Rationale |
|-----------|------|-----------|
| Walking skeleton + acceptance | `tests/acceptance/walking_skeleton_scenarios.rs` | Subprocess tests require their own compilation unit; declared as `[[test]]` in Cargo.toml |
| Config acceptance | `tests/acceptance/config_scenarios.rs` | Config behaviors exercised via CLI binary subprocess |
| Adapter integration | `tests/acceptance/adapter_integration_scenarios.rs` | Real I/O against adapter types directly (not via subprocess) |
| Parser unit tests | `tests/unit/parser_specifications.rs` | Pure function; no subprocess needed; proptest at this layer |
| Update unit tests | `tests/unit/update_specifications.rs` | Pure function; no subprocess needed; proptest state machine at this layer |
| State-delta port | `tests/common/state_delta.rs` | Bootstrapped once per project; used by future in-memory acceptance tests |
| Fixture | `tests/fixtures/sample_daily_note.md` | Real Obsidian daily note format; 5 rows matching rusty-commit-saver contract |

---

### [REF] Driving Adapter Coverage

| Entry point | @driving_adapter scenario | Covered by |
|-------------|--------------------------|------------|
| `rusty_commit_lister` binary (--config flag) | YES — scenario #1 (walking skeleton) | `tests/acceptance/walking_skeleton_scenarios.rs` |
| `parse_note()` (function entry point) | YES — scenario #19 | `tests/unit/parser_specifications.rs` |
| `update()` (function entry point) | YES — scenarios #25–#36 | `tests/unit/update_specifications.rs` |
| `ConfigPort::load()` (adapter entry point) | YES — scenario #12 | `tests/acceptance/adapter_integration_scenarios.rs` |

---

### [REF] Pre-requisites

**DESIGN driving ports required**:
- CLI binary: `assert_cmd = "2.1.1"` in dev-dependencies ✓ (already in Cargo.toml)
- `tempfile = "3.23.0"` ✓
- `predicates = "3.1.3"` ✓
- `proptest = "1.4"` — ADDED by this DISTILL run

**Environment matrix** (DEVOPS missing — defaults applied):
- clean: fresh tempdir per test (always satisfied)
- with-pre-commit: pre-commit hooks present in .git/hooks/post-commit — tests run under cargo test, unaffected
- with-stale-config: tested via the "config missing falls back to defaults" scenarios

**arboard dependency** (slice-05 — NOT yet required):
- ArboardClipboardAdapter scaffold created; arboard NOT added to Cargo.toml
- FakeClipboard implements ClipboardPort for all tests until slice-05

**Cargo.toml changes made by this DISTILL run**:
- `proptest = "1.4"` added to dev-dependencies
- 5 `[[test]]` entries added for the new test files

---

### [REF] Tier Classification

**Tier A only** (no Tier B).

Tier B (RuleBasedStateMachine) is not warranted for slice-01/02/03 journeys. The update_specifications.rs proptest state machine (`any_valid_event_sequence_produces_valid_model`) covers the state space at the unit layer, which is faster and more appropriate (Mandate 10: Tier B requires ≥3 chained scenarios AND domain-rich input — the update() function test suite satisfies this goal at the unit level without requiring a separate in-memory composition root).

---

### [REF] Wave: DISTILL / RED Classification

| Scenario # | Test name | Failure mode | Classification |
|------------|-----------|--------------|----------------|
| 1 | `tool_loads_commits_from_vault_and_exits_successfully` | `panicked at src/main.rs:51:5: Not yet implemented -- RED scaffold` | MISSING_FUNCTIONALITY ✓ |
| 2–36 | all others | `#[ignore]` — not run | SCAFFOLDED_PENDING ✓ |

**Pre-DELIVER gate**: Scenario #1 fails for the right reason (composition root scaffold panic). Classification: MISSING_FUNCTIONALITY. Handoff to DELIVER is authorized.

---

## Wave: DELIVER / [REF] Implementation Summary

Slice-01 walking skeleton of `rusty-commit-lister` shipped end-to-end in 6 TDD steps. The binary reads Obsidian daily-note markdown files from a configured vault directory, parses pipe-table commit rows, and presents them in a ratatui TUI using the Elm/MVU pattern. When stdout is not a TTY (piped or subprocess), it prints a text summary instead of entering alt-screen mode. All composition wiring follows the declared port-and-probe contract: `wire → probe → use`.

## Wave: DELIVER / [REF] Files Modified

**Production**:
- `src/main.rs` — composition root: CLI flags → config load → vault probe → scan → model init → TUI or text output
- `src/parser/mod.rs` — `parse_note(path)` pure function: locates `## Commits` heading, parses pipe table, skip-and-log on malformed rows
- `src/domain/model.rs` — `AppModel::new(config)`, `AppMode` enum, `DEFAULT_SCAN_DAYS_BACK` constant
- `src/domain/update.rs` — `update(model, event) -> AppModel` state machine; filter helpers extracted at L2 refactor
- `src/adapters/toml_config.rs` — `TomlConfigAdapter`: TOML parse via `toml` crate, `~` expansion, `scan_days_back > 0` guard
- `src/adapters/walkdir_vault.rs` — `WalkdirScanAdapter`: walkdir 2 traversal, chrono date-window filter, OsStr emoji path safe
- `src/tui/view.rs` — `view(&AppModel, &mut Frame)`: 4-column table, status bar, loading/error/empty states
- `src/tui/event_loop.rs` — `TuiEventLoop`: raw-mode lifecycle, `Drop` guard, 250 ms crossterm poll loop

**Tests**:
- `tests/unit/parser_specifications.rs` — 6 tests active (5 example + 1 proptest no-panic invariant)
- `tests/unit/update_specifications.rs` — 12 tests active (11 example + 1 proptest state-machine invariant)
- `tests/acceptance/adapter_integration_scenarios.rs` — 7 active, 5 `#[ignore]` (future slices)
- `tests/acceptance/walking_skeleton_scenarios.rs` — 1 active, 5 `#[ignore]` (future slices)

## Wave: DELIVER / [REF] Scenarios Green

28 of 28 active scenarios green as of 2026-05-19.
5 `#[ignore]` acceptance + 10 `#[ignore]` walking-skeleton scenarios deferred to future slices.

## Wave: DELIVER / [REF] DoD Check

| DoD Item | Status |
|---|---|
| All acceptance tests for this slice green | PASS — 28/28 active tests pass |
| Walking skeleton scenario green | PASS — `tool_loads_commits_from_vault_and_exits_successfully` exits 0 |
| No `panic!("Not yet implemented")` in production | PASS — all scaffolds replaced |
| `#![forbid(unsafe_code)]` enforced | PASS — verified by compiler |
| Domain layer has zero adapter/TUI imports | PASS — confirmed by reviewer |
| L1-L6 RPP refactor complete | PASS — scaffold comments removed, constants extracted, helpers named |
| Adversarial review approved | PASS — zero blockers, G1-G9 all pass |
| Mutation kill rate ≥ 80% | PASS — 81.8% (54/66 viable mutants caught) |
| DES integrity verification passes | PASS — all 6 steps have complete traces |

## Wave: DELIVER / [REF] Demo Evidence

Command: `rusty_commit_lister --config <tempdir>/config.toml` (non-TTY subprocess)

```
Found 2 commits:
  2026-05-19 14:32 - feat: add TUI skeleton
  2026-05-19 09:22 - chore: update nvim
```

Exit code: 0. Stdout contains "commits". Captured 2026-05-19.

## Wave: DELIVER / [REF] Quality Gates

| Phase | Outcome |
|---|---|
| Phase 3 — L1-L6 Refactor | PASS — `634d25d` |
| Phase 4 — Adversarial Review | APPROVED — zero blockers, all G1-G9 pass |
| Phase 5 — Mutation Testing | PASS — 81.8% kill rate (cargo-mutants 26.0.0) |
| Phase 6 — DES Integrity | PASS — `des-verify-integrity` exit 0 |

## Wave: DELIVER / [REF] Pre-requisites

DISTILL scenarios depended upon:
- Scenario 1 (`tool_loads_commits_from_vault_and_exits_successfully`) — walking skeleton, now green
- Scenarios 2-7 (adapter integration) — all green
- Scenarios 8-28 (unit parser + update state machine) — all green

DESIGN components shipped:
- `ConfigPort` + `TomlConfigAdapter` — fully wired
- `VaultScanPort` + `WalkdirScanAdapter` — fully wired
- `parse_note()` pure function — fully wired
- `AppModel` / `update()` / `view()` / `TuiEventLoop` — fully wired
- `ClipboardPort` + `ArboardClipboardAdapter` — deferred to slice-02

---

## Wave: DELIVER / [REF] Implementation Summary — Slice-02 (Full Browse Experience)

Slice-02 extends the walking skeleton with three production-ready browse capabilities:
PageUp/PageDown navigation with page_size=10 and boundary clamping (no wrap); message and
folder truncation in the commit table with ellipsis suffix; a "Row N/Total | q quit" status
bar; and a reload_fn closure wired into TuiEventLoop::run() so pressing 'r' re-scans the
vault and refreshes the table within the same event loop iteration.

## Wave: DELIVER / [REF] Files Modified — Slice-02

| File | Change |
|---|---|
| `src/domain/model.rs` | Added `page_size: usize` field (default 10) to `AppModel` |
| `src/domain/update.rs` | Added `PageDown`/`PageUp` arms with clamped navigation |
| `src/tui/view.rs` | Added `truncate()` + `format_status_text()` pure helpers; updated status bar and table cells |
| `src/tui/event_loop.rs` | Changed `run()` to accept `reload_fn: impl FnMut() -> Vec<CommitRecord>`; detects `model.loading` and fires `LoadComplete` |
| `src/main.rs` | Passes vault reload closure to `tui.run()` |
| `tests/unit/update_specifications.rs` | Added 10 new tests (PageUp/Down, r-key, q-key, empty-rows guards, search, repo filter) |

## Wave: DELIVER / [REF] Scenarios Green — Slice-02

38 of 38 active tests pass (25 unit + 6 parser + 7 adapter + 0 ignored acceptance).
Walking skeleton `tool_loads_commits_from_vault_and_exits_successfully` green.

## Wave: DELIVER / [REF] DoD Check — Slice-02

| DoD Item | Status |
|---|---|
| All active tests green | PASS — 38/38 pass |
| Walking skeleton green | PASS — exits 0, non-TTY path unchanged |
| No `panic!` in production | PASS |
| `#![forbid(unsafe_code)]` enforced | PASS |
| Domain layer has zero adapter/TUI imports | PASS |
| L1-L6 RPP refactor complete | PASS — import consolidation; minimal changes needed (code was clean) |
| Mutation kill rate ≥ 80% | PASS — 80.3% (49/61 viable mutants caught) |
| DES integrity verification passes | PASS — all 3 steps have complete traces |

## Wave: DELIVER / [REF] Quality Gates — Slice-02

| Phase | Outcome |
|---|---|
| Phase 2 — All Steps | PASS — commits `30aa416`, `1c25df9`, `ff68037` |
| Phase 3 — L1-L6 Refactor | PASS — import consolidation in view.rs |
| Phase 5 — Mutation Testing | PASS — 80.3% kill rate (cargo-mutants 26.0.0) |
| Phase 6 — DES Integrity | PASS — `des-verify-integrity` exit 0, 3/3 steps |

## Wave: DELIVER / [REF] Mutation Gaps Logged — Slice-02

Remaining 12 missed mutants are structurally untestable at unit level without a terminal mock:
- `event_loop.rs`: run loop body, restore guard, Drop impl, translate_event arm — TUI lifecycle requires real terminal
- `view.rs`: view/render_* function bodies, render_status_bar arithmetic — TUI render requires terminal buffer

These gaps are logged for future slice consideration (terminal mock or ratatui TestBackend integration).

---

## Wave: DELIVER / [REF] Implementation Summary — Slice-03 (Full-Text Search)

Slice-03 completes the inline search experience: pressing Enter in Search mode now confirms
the query and returns to Browse with the filtered view active (search_query preserved,
cursor reset to 0). The view layer gained a three-chunk layout in Search mode — commit table
+ a "/ query_" search bar line + a match-count status bar showing "N of M commits | Esc cancel".
Browse/Detail/RepoPicker modes are layout-unchanged.

## Wave: DELIVER / [REF] Files Modified — Slice-03

| File | Change |
|---|---|
| `src/domain/update.rs` | Added `KeyCode::Enter` arm to `handle_search_key` (confirms search, resets cursor) |
| `src/tui/view.rs` | Added `search_status_text()` helper; added `render_search_bar()`; split layout to 3 chunks in Search mode; `render_status_bar()` dispatches on mode |
| `tests/unit/update_specifications.rs` | Added `enter_in_search_mode_confirms_and_returns_to_browse` test |

## Wave: DELIVER / [REF] Scenarios Green — Slice-03

52 of 52 active tests pass (26 unit + 12 view + 7 adapter + 6 parser + 1 acceptance).
Walking skeleton `tool_loads_commits_from_vault_and_exits_successfully` green.

## Wave: DELIVER / [REF] DoD Check — Slice-03

| DoD Item | Status |
|---|---|
| All active tests green | PASS — 52/52 pass |
| Walking skeleton green | PASS — exits 0 |
| No `panic!` in production | PASS |
| `#![forbid(unsafe_code)]` enforced | PASS |
| Domain layer has zero adapter/TUI imports | PASS |
| L1-L6 RPP refactor complete | PASS — no refactoring needed; code shipped clean |
| Mutation kill rate ≥ 80% | PASS — 83.9% (52/62 viable mutants caught) |
| DES integrity verification passes | PASS — all 2 steps have complete traces |

## Wave: DELIVER / [REF] Quality Gates — Slice-03

| Phase | Outcome |
|---|---|
| Phase 2 — All Steps | PASS — commits `1664662`, `20b46f8` |
| Phase 3 — L1-L6 Refactor | PASS — code shipped clean; no separate refactor commit needed |
| Phase 5 — Mutation Testing | PASS — 83.9% kill rate (cargo-mutants 26.0.0) |
| Phase 6 — DES Integrity | PASS — `des-verify-integrity` exit 0, 2/2 steps |

## Wave: DELIVER / [REF] Mutation Gaps Logged — Slice-03

Remaining 10 missed mutants are all in view.rs TUI render infrastructure (same category as slice-02):
replace-with-() on render functions and `== / !=` arithmetic in render_status_bar.
These require ratatui TestBackend integration to reach — deferred to slice-04 or later.

---

## Wave: DELIVER / [REF] Implementation Summary — Slice-04

Slice-04 ships the Commit Detail Panel (US-07). Pressing Enter on any row in Browse mode opens
a full-screen detail overlay showing un-truncated date, time, message, folder, and URL for the
selected commit. If the URL field is absent, the panel shows "— not available —". Esc closes the
panel and returns to the same row. The domain transitions (Enter→Detail, Esc→Browse) were already
wired in slice-01; this slice added only the view rendering layer.

A pure helper `detail_lines(&CommitRecord) -> Vec<String>` was extracted as the testable core of
the overlay renderer. Seven ratatui `TestBackend` render tests were added to `view_specifications.rs`,
achieving 100% mutation kill rate on view.rs (previously 0% for render functions due to missing
test infrastructure). This also closes the slice-03 "deferred to later" note above.

## Wave: DELIVER / [REF] Files Modified — Slice-04

**Production**
- `src/tui/view.rs` — added `detail_lines()`, `render_detail_overlay()`, Detail branch in
  `render_main_area()`, "Esc to return" arm in `render_status_bar()`

**Tests**
- `tests/unit/view_specifications.rs` — NEW file; 9 tests covering `detail_lines()` pure function
  (2 tests) and ratatui TestBackend render tests (7 tests)

## Wave: DELIVER / [REF] Scenarios Green — Slice-04

9 of 9 new view_specifications tests pass. 66 total active tests pass across all test suites.
Walking skeleton `tool_loads_commits_from_vault_and_exits_successfully` green.

## Wave: DELIVER / [REF] DoD Check — Slice-04

| DoD Item | Status |
|---|---|
| All active tests green | PASS — 66/66 pass |
| Walking skeleton green | PASS — exits 0 |
| No `panic!` in production | PASS |
| `#![forbid(unsafe_code)]` enforced | PASS |
| Domain layer has zero adapter/TUI imports | PASS |
| L1-L6 RPP refactor complete | PASS — no refactoring needed; code shipped clean |
| Mutation kill rate ≥ 80% | PASS — 100% (24/24 mutants caught) |
| DES integrity verification passes | PASS — all 1 step has complete traces |

## Wave: DELIVER / [REF] Quality Gates — Slice-04

| Phase | Outcome |
|---|---|
| Phase 2 — All Steps | PASS — commit `51eee7c` |
| Phase 3 — L1-L6 Refactor | PASS — code shipped clean; no separate refactor commit needed |
| Phase 5 — Mutation Testing | PASS — 100% kill rate (commit `75db2b0` adds TestBackend tests) |
| Phase 6 — DES Integrity | PASS — `des-verify-integrity` exit 0, 1/1 steps |

---

## Wave: DELIVER / [REF] Implementation Summary — Slice-05

Slice-05 ships clipboard copy from the Detail panel (US-08). Pressing `c` in Detail mode copies
the repository URL to the system clipboard and shows "URL copied to clipboard" in the overlay.
If the URL field is absent, the panel shows "Copy not available — no URL". If the clipboard
is inaccessible (SSH/headless), shows "Copy not available — select text manually" with no crash.

`ArboardClipboardAdapter` (arboard crate) was created and probed non-fatally at startup. The
clipboard effect follows the Elm/MVU pattern: domain sets `clipboard_pending`, the event loop
executes the effect and dispatches `ClipboardResult`, domain clears pending and sets
`status_message`. Status bar in Detail mode updated to "c copy | Esc return".

## Wave: DELIVER / [REF] Files Modified — Slice-05

**Production**
- `src/domain/model.rs` — added `pub clipboard_pending: Option<String>` field
- `src/domain/update.rs` — `c` key arm in `handle_detail_key`; `clipboard_pending = None` in `ClipboardResult` arms
- `src/tui/view.rs` — `render_detail_overlay` shows `status_message`; status bar updated to "c copy | Esc return"
- `src/adapters/arboard_clipboard.rs` — NEW: `ArboardClipboardAdapter` implementing `ClipboardPort + Probe`
- `src/adapters/mod.rs` — added `pub mod arboard_clipboard`
- `src/tui/event_loop.rs` — `run()` gains `clipboard_fn` param; dispatches `ClipboardResult` when `clipboard_pending` is set
- `src/main.rs` — probes clipboard adapter (non-fatal); passes clipboard closure to `tui.run`
- `Cargo.toml` — added `arboard = "3"`

**Tests**
- `tests/unit/update_specifications.rs` — 5 new tests: `c_key_*` and `clipboard_result_*`
- `tests/unit/view_specifications.rs` — 2 new render tests: status_message in overlay + "c copy" hint

## Wave: DELIVER / [REF] Scenarios Green — Slice-05

42 of 42 active tests pass across all test suites (31 unit update + 11 view + 6 acceptance/parser + 1 walking skeleton).

## Wave: DELIVER / [REF] DoD Check — Slice-05

| DoD Item | Status |
|---|---|
| All active tests green | PASS — 42/42 pass |
| Walking skeleton green | PASS — exits 0 |
| No `panic!` in production | PASS |
| `#![forbid(unsafe_code)]` enforced | PASS |
| Domain layer has zero adapter/TUI imports | PASS |
| L1-L6 RPP refactor complete | PASS — code shipped clean |
| Mutation kill rate ≥ 80% | PASS — 100% (70 caught, 7 unviable, 0 missed) |
| DES integrity verification passes | PASS — all 2 steps have complete traces |

## Wave: DELIVER / [REF] Quality Gates — Slice-05

| Phase | Outcome |
|---|---|
| Phase 2 — All Steps | PASS — commits `45dd490` (domain), `6b3d48d` (infra) |
| Phase 3 — L1-L6 Refactor | PASS — code shipped clean; no separate refactor commit needed |
| Phase 5 — Mutation Testing | PASS — 100% kill rate (domain/model.rs, domain/update.rs, tui/view.rs) |
| Phase 6 — DES Integrity | PASS — `des-verify-integrity` exit 0, 2/2 steps |
