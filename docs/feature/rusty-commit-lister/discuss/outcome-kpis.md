# Outcome KPIs — rusty-commit-lister

Generated: 2026-05-18 — DISCUSS wave
Owner: franci (sole user = sole evaluator)

---

## Feature: rusty-commit-lister (Tier 1 — Slices 01–06)

### Objective

Within two weeks of first working slice, rusty-commit-lister replaces every grep-and-scroll
workaround for daily commit orientation — franci opens it without thinking, finds what she
needs, and returns to work without breaking flow.

---

### Outcome KPIs

| # | Who | Does What | By How Much | Baseline | Measured By | Type |
|---|---|---|---|---|---|---|
| KPI-1 | franci | Opens rusty-commit-lister instead of grep or Obsidian scroll for session orientation | Grep/Obsidian for orientation drops to 0 within 14 days of slice-02 ship | Currently: grep or Obsidian every session | Self-observation: did I reach for grep today? (binary daily log) | Leading / Adoption |
| KPI-2 | franci | Completes orientation scan (launch → browse → exit) without consulting README or help | First unassisted session within 3 days of first run | Currently: no tool, no baseline | Observe: needed README? (binary) | Leading / Usability |
| KPI-3 | franci | Finds a specific commit using search (/) without grepping | First /search use within first week after slice-03 ships | Currently: always grep -r | Observe: did I use / or did I grep? (binary) | Leading / Adoption |
| KPI-4 | franci | Completes full orient-scan in < 30 seconds (launch to exit) for a 7-day window | < 30 seconds on first use; < 15 seconds after habit forms | Currently: grep + context-switch ≈ 90–120 seconds | Stopwatch self-test on representative session | Leading / Efficiency |
| KPI-5 | franci | Uses the tool at least once on each working day | Tool used ≥ 5 of any 7 consecutive working days | Currently: 0 uses/day | Shell history or binary daily log | Lagging / Retention |

---

### Metric Hierarchy

- **North Star**: KPI-5 — daily habitual use (5 of 7 working days). This is the adoption signal that the tool has replaced workarounds.
- **Leading Indicators**:
  - KPI-1 (workaround abandonment) predicts KPI-5
  - KPI-3 (search adoption) predicts KPI-1 for the "find a commit" sub-job
- **Guardrail Metrics** (must NOT degrade):
  - Startup time: < 2 seconds for scan_days_back ≤ 30 (measured at slice-01, protected through all slices)
  - Zero silent data loss: commit count on slice-02 matches manual count from daily notes (spot check)
  - Clean exit: terminal always restores after q or Ctrl+C (regression test per slice)

---

### Per-Slice KPI Targets

| Slice | Primary KPI | Success Signal |
|---|---|---|
| Slice 01 (Walking Skeleton) | Guardrail: startup < 2s, no crash, unicode path resolves | Automated: startup test + unicode path integration test |
| Slice 02 (Full Browse) | KPI-1: franci uses tool for one full session without grep | Self-observation day 1–3 after ship |
| Slice 03 (Search) | KPI-3: first / search invoked without README | Observe within 48h of slice-03 ship |
| Slice 04 (Detail View) | KPI-4: orient scan < 30s | Stopwatch self-test on day of ship |
| Slice 05 (Clipboard) | KPI-4: < 15s including URL copy | Stopwatch self-test |
| Slice 06 (Repo Filter) | KPI-5: tool used 5/7 working days for two weeks | 14-day daily log |

---

### Measurement Plan

| KPI | Data Source | Collection Method | Frequency | Owner |
|---|---|---|---|---|
| KPI-1 | Self-observation | Binary note: "grep today? Y/N" | Daily | franci |
| KPI-2 | Self-observation | Binary note: "consulted README? Y/N" | First 3 days only | franci |
| KPI-3 | Shell history or self-log | Check `history | grep 'grep -r'` vs tool open | Weekly | franci |
| KPI-4 | Manual stopwatch or `time rusty-commit-lister` | Run on representative 7-day session | Per slice ship | franci |
| KPI-5 | Shell history | `history | grep rusty-commit-lister | grep -c ''` per day | Weekly tally | franci |
| Guardrail: startup | Automated test | `cargo test startup_time` | Per CI run | franci |

---

### Hypothesis

We believe that a sub-2-second terminal-native commit browser for franci will achieve daily
habitual use (5 of 7 working days) within two weeks of the full-browse slice shipping.

We will know this is true when **franci** **opens rusty-commit-lister instead of grep or Obsidian
for session orientation** **on 5 of any 7 consecutive working days**.

---

### OMTM (One Metric That Matters)

Stage: **Stickiness** — does the tool become a daily habit?

OMTM: **KPI-5 — tool used at least 5 of 7 working days for two consecutive weeks post-slice-02.**

All other KPIs are leading indicators for or guardrails around this north star.

---

### Baseline Measurement Notes

Since franci is sole user and there are zero current uses of the tool:
- Baseline for KPI-1: 100% of orientation sessions use grep or Obsidian (behavioral evidence)
- Baseline for KPI-4: ~90–120 seconds (observed workaround: grep + context switch + Obsidian scroll)
- Baseline for KPI-5: 0 uses/day
- All baselines are qualitative/behavioral estimates — quantitative baseline established at slice-02 ship
