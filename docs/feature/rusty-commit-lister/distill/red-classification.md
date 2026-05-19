# RED Classification — rusty-commit-lister

**DISTILL wave — 2026-05-18**
**Purpose**: Pre-DELIVER fail-for-the-right-reason gate. DELIVER reads this file at PREPARE phase.

## Classification

| Scenario | Test name | Failure mode | Classification |
|----------|-----------|--------------|----------------|
| #1 (walking skeleton) | `tool_loads_commits_from_vault_and_exits_successfully` | `panicked at src/main.rs:51:5: Not yet implemented -- RED scaffold` | MISSING_FUNCTIONALITY ✓ |
| #2–#36 | all #[ignore]-marked tests | Not run (skip marker) | SCAFFOLDED_PENDING ✓ |

## Gate Result

**PASS** — zero BROKEN or WRONG_ASSERTION failures detected.

Scenario #1 fails because the composition root (`main.rs`) contains `panic!("Not yet implemented -- RED scaffold")`. The assert_cmd subprocess captures this as exit code 101 (process panicked). The test infrastructure is correct — it compiled, ran, and reached the assertion. The failure is genuine MISSING_FUNCTIONALITY.

## Instructions for DELIVER

1. Start with scenario #1 (`tool_loads_commits_from_vault_and_exits_successfully`).
2. It is currently enabled (no `#[ignore]`).
3. Implement the composition root in `main.rs` to make it GREEN.
4. Then unskip scenario #2 (`invalid_scan_days_back_exits_code_2`) and implement US-01 config validation.
5. Continue unskipping one scenario at a time.

Detection command to verify scaffolds remain: `grep -r "SCAFFOLD: true" src/`
