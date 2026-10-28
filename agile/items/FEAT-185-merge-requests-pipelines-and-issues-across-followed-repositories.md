<!-- SPDX-License-Identifier: MIT -->

# FEAT-185 — Merge requests, Pipelines and Issues across followed repositories

**Status:** Backlog — proposed in `docs/proposals/workbench-wide-home-and-load-repository.md` § 3 and § 13. The author decided on 2026-10-28 that it follows once FEAT-180 to FEAT-184 have shipped.
**Branch:** none yet.
**Screens:** 1H Pull requests (Merge requests), 1Y Pipelines (live), 1W Issues.
**Raised by:** the author, 2026-10-28: "a lot of features should not be repo oriented … work generally with calls and pipelines of selected repos."
**Depends on:** FEAT-182, FEAT-184.

## Change

- With nothing loaded, these three screens list across followed projects. Each row carries a project chip, and a project filter sits in the header.
- With a repository loaded, they open filtered to it, with *All followed* one click away.
- Their rail rows are never dimmed.
- Pipelines › Preview (which reads the local CI file) stays repository-scoped.

## Acceptance criteria

1. Each screen works with nothing loaded and shows rows from every followed project.
2. The project filter and *All followed* switch without a reload of the window.
3. Writes (comment, merge, retry a job) act on the row's project, whether or not it is loaded.
4. With `FEATURES.workbenchWide` off, the screens are unchanged.
