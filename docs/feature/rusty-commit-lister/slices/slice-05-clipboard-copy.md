# Slice 05 — Clipboard Copy

**Outcome**: Franci copies a repo URL from the detail panel in one keypress — ready to paste anywhere.

## Why this slice is fifth

Once the detail view exists (slice 04), the natural next action is "now I want to use this URL."
Clipboard copy is the minimum viable act — it works whether franci is going to a browser, a chat,
or a commit message. Browser-open (score 7) is explicitly Tier 3 and deferred.

## Scope

- In the detail panel: `c` copies the REPOSITORY URL to the system clipboard
- Confirmation line appears in panel: "URL copied to clipboard"
- If clipboard unavailable (SSH/headless): show URL as selectable/highlighted text instead; no crash
- No browser-open in this slice (deferred to slice 06 or later)

## Acceptance criteria summary

- `c` on a row with a URL: clipboard contains the exact URL shown in the panel
- Confirmation message visible in the panel after copy
- No crash or panic when clipboard is unavailable
- URL shown as text fallback in non-clipboard environments

## Job traceability

`job-inspect-commit` — act on a commit URL without leaving the terminal

## Story IDs

- US-08: Clipboard copy from detail panel with confirmation and graceful fallback

## Estimated effort

0.5–1 day
