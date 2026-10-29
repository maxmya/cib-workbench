<!-- SPDX-License-Identifier: MIT -->

# FEAT-183 — Plan

**Item:** [`agile/items/FEAT-183-home-across-followed-repositories.md`](../items/FEAT-183-home-across-followed-repositories.md)
**Branch:** `feature/workbench-2.0.0`, as 2.0.0.
**Status:** implemented.

## Approach

### One read in Rust

`spagitty_core::workbench::home_wide` returns `WideHome`:

- across the host, followed or not: the person's open merge requests
  (`scope=created_by_me`) and their review requests (`reviewer_username`),
  and open issues assigned to them;
- per followed project: the latest pipeline on its default branch, the latest
  on up to two of the person's own merge request branches there, and the
  latest release.

Two merge request lists rather than `involved_merge_requests`, which is review
requests only: the Merge requests card and the own-branch pipelines need the
person's own as well. Rows are `InProject<T> { projectId, project, item }`;
merge requests have a type of their own (`MergeRequest`) because
`PullRequest` cannot be read back from a saved file.

Per-project reads go through a pool of `IN_FLIGHT` (4) threads. A 403 or 404
is "has none" (CI or releases switched off), any other failure is a
`ProjectError` in that project's card, and the rest are still shown. A
project read within `fresh` seconds (the forge watcher's interval, from
Settings › Notifications) is taken from the last answer; Refresh passes
`force` and reads everything. Following is capped at 50.

`home_workbench_wide(force)` saves each answer; `home_workbench_wide_cached`
paints it first.

### Webview

`home.ts` is pure: `onlyProject` (*<repo> only*), `inFollowed`, `wideNeeds`
(review requests anywhere, failed latest runs, assigned issues, each with a
project chip), `recentCollections` (Calls, by last use in Calls' Workbench-wide
history). `wide-home.svelte.ts` reads, and rides the watcher's clock while
Home is on screen. `WideHome.svelte` draws the header, the Loaded band, the
empty state and the cards in `HomeLayout` order over `WIDE_HOME_CARDS`.

`HomeCardId` gains `issues` and `calls`. `shownCards`, `moveCardBy` and
`moveCardOnto` take the card list as an argument, defaulting to Spagitty's,
so the Spagitty path is unchanged.

`NeedsCard` draws `need.project` when it is set, and `NeedSource` gains
`link` for a merge request's page.

### Loaded band

`LoadedBand.svelte`: Working copy (`WorkingCard`), Branch (current branch
against upstream, last fetched, Push) and Recent commits (`CommitsCard`).
The 16-week activity and the Farm card are not in it; the Farm is on the rail.

### Rows

A row for a screen in a project that is loaded goes there; otherwise Load
repository opens with that project first, then goes to the screen.
