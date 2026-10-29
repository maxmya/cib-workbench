<!-- SPDX-License-Identifier: MIT -->

# FEAT-184 — Load repository instead of tabs

**Status:** Done — on `feature/workbench-2.0.0`, as Workbench 2.0.0, from the proposal's § 9. The manual sweep is still to run.
**Branch:** `feature/workbench-2.0.0`, with the other four of FEAT-180 to FEAT-184.
**Screens:** 2E Load repository (new dialog), chrome (title bar slot, rail), the launch.
**Raised by:** the author, 2026-10-28: "instead of multiple tabs, we shall have a load-repo function that makes the app load a specific repo to act upon."

## Change

- Under `FEATURES.workbenchWide`, `RepoTabs.svelte` is not rendered. `workspace.places` is kept as it is; `active` is the loaded repository; recents come from `recent_repos`.
- **Title bar slot.** With nothing loaded: "No repository loaded · Load repository `Ctrl+O`". With a repository loaded: path, branch, ↑/↓, *Switch* and *Unload*.
- **The 2E dialog.** Search; *On this machine* (recents, plus GitLab projects already cloned, matched by the FEAT-087 lookup); *On CIB GitLab* (followed first, then search) with *Clone and load*; *Open a folder…*. Esc, ↑/↓ and Enter are handled inside the dialog; Ctrl+O is registered with the app's other shortcuts.
- **Load** remembers the place being left, opens the new repository and restores its place. **Clone and load** uses the existing clone flow. **Unload** calls `close_repo` and `workspace.rest()`, then goes to Home.
- **Rail.** Repository-scoped rows are dimmed and `aria-disabled` with nothing loaded. Activating one opens 2E, then goes to that screen after the load. A repository-scoped route with nothing loaded opens 2E over Home.
- **Launch.** Lands on Home and reloads the last loaded repository, unless FEAT-172's setting (reworded *Open with nothing loaded*) is on.
- **Migration.** An old `spagitty.workspace` reads without loss; tabs are ignored, not deleted.

## Acceptance criteria

1. No tab strip shows under the switch. Turning the switch off brings the previous strip back intact.
2. Loading B while A is loaded leaves one repository loaded (B), and reloading A later returns to A's route and selected commit.
3. Clone and load shows progress in the row and ends with the repository loaded and in recents.
4. Unload leaves nothing loaded, keeps the place, and goes to Home from a repository-scoped screen.
5. A dimmed rail row is focusable, says why in its title, opens 2E, and navigates after the load.
6. The `resumeSession` sequence tests cover reload-last, FEAT-172 on, and a command-line path.
7. With `FEATURES.workbenchWide` off, the existing suite passes unchanged.

## As built

- Loading from a repository screen returns to the loaded repository's own place; loading from Home or another Workbench-wide screen stays there and keeps the selected commit for the graph.
- `gitlab_local_clones` answers the FEAT-087 lookup for every remembered repository in one read, so the dialog does not ask once per project.
- The forge lists (Pull requests, Issues, Pipelines, Releases) are not dimmed with nothing loaded; making them Workbench-wide is FEAT-185.
- The clone folder is remembered in `workbench.json` (`cloneInto`), defaulting to the folder of the most recent repository.
- Plan: [`FEAT-184-plan.md`](../plans/FEAT-184-plan.md). Tests: [`FEAT-184-automated.md`](../testing/FEAT-184-automated.md). Sweep: [`FEAT-184-sweep.md`](../testing/FEAT-184-sweep.md).
