# Shared Artifacts Registry — rusty-commit-lister

Generated: 2026-05-18 — DISCUSS wave coherence validation
Owner: franci

---

## Registry

### vault_path

| Field | Value |
|---|---|
| Source of truth | `~/.config/rusty-commit-lister/config.toml` → `vault_path` key |
| Type | String (filesystem path, may contain Unicode emoji) |
| Example | `~/Documents/Wiki/📅 Diaries/0. Journal` |
| Consumers | launch step (config load), load step (file traversal), error messages (parse failure context) |
| Integration risk | HIGH — Unicode emoji segment (`📅 Diaries`) must survive `OsString`/`PathBuf` round-trip in Rust |
| Validation | Spike test in slice-01: load path from TOML → resolve via `PathBuf` → `walkdir` traverse → confirm files found |

---

### scan_days_back

| Field | Value |
|---|---|
| Source of truth | `~/.config/rusty-commit-lister/config.toml` → `scan_days_back` key |
| Default | `7` |
| Type | Positive integer |
| Consumers | launch step (status line: "last N days"), load step (date range calculation), status bar |
| Integration risk | MEDIUM — default must be consistent between config module and status bar display |
| Validation | If config absent, tool uses default 7 and shows notice; status bar must reflect actual value used |

---

### commit_rows

| Field | Value |
|---|---|
| Source of truth | Markdown table parser reading Obsidian daily notes (one row per `## Commits` table entry) |
| Format | Vec of structs: `{ date, time, folder, repo, message, url }` |
| Consumers | load step (row count display), browse step (table render), search step (filter input), inspect step (detail panel) |
| Integration risk | HIGH — parser must handle malformed rows without panic (skip-and-log) |
| Validation | Integration test: real Obsidian note fixture with known row count; assert parser output matches expected |

---

### note_date_format

| Field | Value |
|---|---|
| Source of truth | rusty-commit-saver output convention: note filename `YYYY-MM-DD.md`, TIME column `HH:MM` |
| Consumers | load step (date-range filtering by filename), browse step (date column display), inspect step (detail panel date field) |
| Integration risk | HIGH — if rusty-commit-saver changes date format, parser silently produces wrong dates or skips notes |
| Validation | Document expected format in parser module as a constant; parse error produces skip + log, not silent wrong data |

---

### selected_row

| Field | Value |
|---|---|
| Source of truth | TUI cursor state (in-memory `usize` index into `commit_rows`) |
| Consumers | browse step (▶ indicator + row highlight), status bar ("Row N/Total"), inspect step (detail panel source) |
| Integration risk | LOW — local in-memory state; risk is off-by-one errors on wrap and page jump |
| Validation | Unit test: cursor at last row + j → cursor wraps to 0. Cursor at 14 → open detail → Esc → cursor still 14 |

---

### search_query

| Field | Value |
|---|---|
| Source of truth | TUI search input state (in-memory `String`) |
| Consumers | search step (filter bar display), load/browse step (filtered `commit_rows` subset), status bar ("N of Total") |
| Integration risk | MEDIUM — Esc must distinguish "exit search mode" from "exit application" based on current mode |
| Validation | State machine test: search_active=true, Esc → search_active=false, full list restored; search_active=false, Esc → app exit |

---

### commit_url

| Field | Value |
|---|---|
| Source of truth | `REPOSITORY URL` column from the parsed Markdown table row (may be empty for older notes) |
| Consumers | inspect step (URL field in detail panel), act step (clipboard copy source) |
| Integration risk | MEDIUM — URL displayed in panel must be byte-for-byte identical to URL written to clipboard |
| Validation | Assert: clipboard content == detail panel URL string after pressing c |

---

### commit_folder_path

| Field | Value |
|---|---|
| Source of truth | `FOLDER` column from the parsed Markdown table row |
| Consumers | inspect step (Full path field in detail panel) |
| Integration risk | LOW — display only; no transformation needed |
| Validation | Render test: path with spaces and special chars displays without truncation in detail panel |

---

## Integration Checkpoints

| Checkpoint | Steps Involved | Validation Method |
|---|---|---|
| config → parser: vault_path resolves | launch → load | Spike test on real emoji path |
| parser → TUI: row count matches | load → browse | Assert status bar count == len(commit_rows) |
| cursor → detail: same row | browse → inspect | Esc from detail returns same cursor index |
| detail → clipboard: URL matches | inspect → act | Assert clipboard == detail_panel.url |
| search_query → filter → status: match count | search → browse → status bar | Assert filtered_rows.len() == status bar number |

---

## CLI Vocabulary Consistency

| Term | Where used | Consistent? |
|---|---|---|
| `rusty-commit-lister` | binary name, config path, error messages | Yes |
| `scan_days_back` | config key, status bar, error messages | Yes — use same snake_case everywhere |
| `vault_path` | config key, error messages | Yes |
| `## Commits` | parser contract (from rusty-commit-saver) | Stable — treat as immutable |
| `j/k` | keybindings in help line | Yes — vim convention |
| `/` | search activation | Yes — standard TUI search convention |
| `q`/`Esc` | exit / close panel | Esc is context-sensitive (search → exit search; main → exit app) — document clearly |

---

## Emotional Coherence Check

| Journey Phase | Emotional State | Coherent? | Notes |
|---|---|---|---|
| Launch → Loading spinner | Slightly disoriented → anticipatory | Yes | Spinner within 100ms prevents "is it frozen?" anxiety |
| Load → Row count displayed | Anticipatory → grounded | Yes | "37 commits" is the moment anxiety resolves |
| Browse → Navigation | Focused and scanning | Yes | Fluid j/k navigation maintains flow state |
| Search → Real-time filter | Mildly frustrated → relieved | Yes | Immediate feedback resolves "too many rows" friction |
| Inspect → Detail panel | Curious → satisfied | Yes | Full context visible in one keypress |
| Act → Clipboard confirmation | Action-ready → complete | Yes | Confirmation line removes doubt about whether copy worked |
| Exit → Clean terminal | Done → grounded | Yes | No artefacts; flow unbroken |

No jarring transitions. Confidence builds progressively. Error states guide to resolution.

---

## Coherence Validation Status

- [x] Journey completeness: all 7 steps have goals, actions, success criteria, failure modes
- [x] Emotional coherence: arc defined (disoriented → grounded), no jarring transitions, confidence builds
- [x] Horizontal integration: all shared artifacts have single source of truth, all consumers documented
- [x] CLI UX compliance: key conventions consistent, Esc context disambiguated, help line in status bar
- [x] Integration checkpoints: 5 checkpoints defined, each testable
