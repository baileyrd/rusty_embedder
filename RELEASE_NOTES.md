# Release Notes

<!--
Two variants, pick the one that fits this repo's actual unit of change:

1. No version tags yet (pre-1.0, nothing published) — track by PR instead, same way
   AISF does it: one entry per merged PR against main, reverse chronological, each
   linking to its PR and (where one exists) to the doc that covers the change in full
   detail. Use "## PR #N — <summary>" headers.

2. Actual version tags exist — use "## vX.Y.Z - YYYY-MM-DD" headers instead, each
   linking to the PRs it shipped and a compare link to the previous tag. Add an
   "### Upgrade notes" subsection under any entry with a breaking change.

Either way, keep the tone AISF's file uses: bolded category tags inline in the
bullet (**Added:** / **Changed:** / **Fixed:**), not separate subheaders per
category — and state known limitations or deliberate scope cuts plainly instead of
leaving them implied.
-->

Tracks notable changes to this repo, one entry per merged PR against `main`,
reverse chronological (no version tags yet — pre-1.0, nothing published).

---

## PR #1 — Add .github PR and issue templates
**2026-08-12** · [#1](https://github.com/baileyrd/rusty_embedder/pull/1)

- **Added:** the `.github/PULL_REQUEST_TEMPLATE/` (feature, bug_fix, docs,
  chore) and `.github/ISSUE_TEMPLATE/` (bug_report, feature_request,
  config.yml) sets called for in `CONTRIBUTING.md`'s workflow step 5
  ("pick the template that matches"). These were hand-authored rather than
  copied from the `repo-config` skill, since its template payload doesn't
  currently ship them (see the known limitation below).
- Closes the gap noted in the previous entry — `repo-config` audit is now
  10/10.

## Repo governance setup
**2026-08-12** · [`40080f7`](https://github.com/baileyrd/rusty_embedder/commit/40080f7da7972e6216da7a789818eba8b459fafc)
(direct push, not a PR — repo had zero commits/branches, nothing to open a
PR against yet; this commit became the default branch)

- **Added:** standard governance file set via the `repo-config` skill — README,
  CONTRIBUTING, CODE_OF_CONDUCT, SECURITY, CHANGELOG, RELEASE_NOTES,
  ARCHITECTURE, and an ADR seed. Applied greenfield defaults (modular
  monolith, ports-and-adapters per `Atlas_Engineering_Standards_Library`
  ATLAS-001 Chapter 21/22) since this repo had no code, commits, or existing
  standard files to scan.
- **Known limitation:** no `.github/PULL_REQUEST_TEMPLATE` or
  `.github/ISSUE_TEMPLATE` were added — the `repo-config` skill's own asset
  payload doesn't currently include them despite its description saying it
  covers "two `.github/` template folders," so there was nothing to copy.
  Flagged to the repo owner rather than fabricated.
- No CI workflow yet — no `Cargo.toml`/`pyproject.toml` exists to select a
  stack-specific workflow for.
