<!-- SPDX-License-Identifier: MIT -->

# FEAT-183 — Home across followed repositories

**Status:** Backlog — proposed in `docs/proposals/workbench-wide-home-and-load-repository.md` § 8.
**Branch:** none yet.
**Screens:** 2A Home.
**Raised by:** the author, 2026-10-28: "work generally with calls and pipelines of selected repos" — Home is not tied to an open repository.
**Depends on:** FEAT-182. Shares the title bar slot with FEAT-184.

## Change

- One Rust read, `home_workbench_wide`, returns the rows for these cards across followed projects:
  - Needs you (the account-wide watcher read, plus failed pipelines and assigned issues);
  - Merge requests (`involved_merge_requests`);
  - Pipelines (the latest run per project, `pipelines?ref=<default>&per_page=1`, lighter than `pipelines::page`; no clone needed);
  - Issues (`scope=assigned_to_me`);
  - Releases (latest per project).

  It also returns per-project errors. Fan-out is capped at 4 in flight. A project read within the watcher's last interval is not read again. Following is capped at 50 projects.
- The Calls card reads the Workbench-wide collections (FEAT-170).
- `HomeCardId` gains `issues` and `calls`. Under the switch, the repository cards (`working`, `branches`, `commits`, `heat`, `farm`) move into a *Loaded* band shown only while a repository is loaded.
- Header: "Following N repositories on CIB GitLab · updated … · *Change*", *Customise Home*, and with a repository loaded, *All followed* | *<repo> only*.
- Empty state when nothing is followed: *Choose repositories* and *Load a repository*; Calls still shows.
- Reads happen when Home is shown, on Refresh, and on the forge watcher's clock. Home adds no timer.

## Acceptance criteria

1. With nothing loaded and three projects followed, every Workbench-wide card fills from GitLab, and every row names its project.
2. One project failing shows that project's error in its card; the others still show.
3. *<repo> only* filters every Workbench-wide card to the loaded project.
4. The card order set in onboarding (FEAT-180) or *Customise Home* is the order shown.
5. A row for a repository-scoped screen in a project that is not loaded opens Load repository (FEAT-184) with that project selected.
6. With `FEATURES.workbenchWide` off, Home is FEAT-140/FEAT-148 unchanged.
