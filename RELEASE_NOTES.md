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

## Repo governance setup
**2026-08-12** · (PR link to be added once merged)

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
