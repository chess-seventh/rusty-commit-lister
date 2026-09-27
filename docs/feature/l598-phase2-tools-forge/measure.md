# L598 — rusty-commit-lister is canonical on the forge under tools/

One of the four repositories this lane moves. Every number below is command
output, not a number anybody typed; an item marked NOT YET MEASURED is exactly
that, never inferred.

## Must-prove 1 — the forge carries every branch and tag GitHub has

⛔ **NOT YET MEASURED — blocked on the push-create, which Claude Code's
auto-mode classifier refused as this session's own action ("Data
Exfiltration").** Franci runs the push himself; this section is filled in once
it lands.

BEFORE, measured 2026-09-27 with `git ls-remote --refs`/`--tags` against
`origin` (GitHub), `refs/pull/*` excluded:

```text
4cf2f7e55fea91b030a421c5346c72f8e0a9b281  refs/heads/dependabot/cargo/clap-4.6.7
c5f10986bae2581f5ac79b50ce29d3972b6cc9f7  refs/heads/dependabot/cargo/thiserror-2.0.20
df33b86c7c6b43ebf316f6f2d53182dd8db7d5f7  refs/heads/dependabot/cargo/toml-1.1.6spec-1.1.0
9ea5047d938b9f2db56516400fa84472cc31e9e3  refs/heads/feat/L235
69367d68c87dec19c6f773c4b323d79ab75acee2  refs/heads/feat/L598
6b1aa2ca57e04f3af5087c2c6e096aee3d281430  refs/heads/feat/saver-and-release
d4563d9ab37d3bc87ed54fcb4d46c4901ceddb84  refs/heads/github-pages
69367d68c87dec19c6f773c4b323d79ab75acee2  refs/heads/main
476d13f17e9c8bc3f72c09d687feddef9f611c9e  refs/tags/v0.2.0
8b3a1f1d2c0b6e07e64d43f7c25737ecd9ab6820  refs/tags/v0.2.1
b0910d1df7d839973de0b0f5b1e8b8453b78b87c  refs/tags/v0.2.2
bca6e7de182252bf4b76c5ae920e0e2f8d31dcc8  refs/tags/v0.3.0
7649eae94e6cc46185ca353543e5a9e0c204a00c  refs/tags/v0.3.1
777a20f7ad41f8b6554ebf960a968d73b1e8a774  refs/tags/v0.3.2
54781b08db5af706fe137442b2ae5965e85f0849  refs/tags/v0.4.0
```

⚠ **`feat/L598` IS THIS LANE'S OWN CLAIM BRANCH**, taken after the above, so it
appears in any later forge listing and is not a discrepancy. `github-pages` is
a build-artifact branch this repository's docs workflow wrote, carried as-is
like any other ref — not a source branch, but not excluded either.

The command Franci runs, from
`/home/seventh/src/claude-worktrees/rusty-commit-lister/L598`:

```bash
git push forge 'refs/remotes/origin/*:refs/heads/*' --follow-tags
```

AFTER: ⛔ NOT YET MEASURED. To close this item: `git ls-remote --refs
--tags forge` against `origin`'s list above with `comm -23`, expecting nothing
printed.

## Must-prove 2 — no file under .github/workflows after the lane

✅ Satisfied on this branch. All seven workflows (`ci.yml`, `codeql.yml`,
`deny.yml`, `docs.yml`, `pr-title.yml`, `release.yml`, `security-audit.yml`)
moved to `.github/workflows-paused/`, each carrying a header naming why;
`.github/workflows/` is empty.

## Must-prove 3 — a push to the forge creates a task on a seat that exists

⛔ **NOT YET MEASURED** — depends on Must-prove 1's push landing and a
subsequent push (or the eventual pull request) triggering
`.forgejo/workflows/gate.yaml`. Record the task id here once one runs.

## Must-prove 4 — no push mirror is created, and the README says why

✅ **No mirror configured or requested by this lane.** README.md's new
"Where this repository lives" section names the forge as canonical, gives the
clone line, and says the mirror is an admin-level forge setting outside this
repository's own configuration — same wording as the sibling migrations
(L562/564/566/567/568 and this lane's own rusty-ntfy half).
