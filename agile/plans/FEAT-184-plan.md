<!-- SPDX-License-Identifier: MIT -->

# FEAT-184 — Plan

**Item:** [`agile/items/FEAT-184-load-repository-instead-of-tabs.md`](../items/FEAT-184-load-repository-instead-of-tabs.md)
**Branch:** `feature/workbench-2.0.0`, as 2.0.0.
**Status:** implemented.

## Approach

### The order of a load, behind a port

`src/lib/workbench/load.ts` holds `loadRepository` and `unloadRepository`,
tested by the calls they make, as `session.ts` is. Load remembers the place
being left, opens the new one, and:

- with a `then` (a dimmed rail row's screen), goes there;
- from a repository screen, goes back to the repository's own place;
- from Home, Calls or another Workbench-wide screen, stays, and wants the
  selected commit.

Unload remembers the place, closes, rests the workspace, and goes to Home
from a repository screen.

`loader.svelte.ts` supplies the real port, holds the dialog's state, and runs
*Clone and load*: `clone_plan`, `clone_start`, its own listeners for the
clone's events (matched by token, so the clone screen's store ignores them),
then a load. A destination that already holds the clone is loaded as it is.

### The dialog and the slot

`LoadDialog.svelte` (2E): one search over the recent repositories and CIB
GitLab (`ProjectList`). `gitlab_local_clones` — the FEAT-087 lookup for every
remembered repository at once — puts a project that is already cloned under
*On this machine*, not twice. Followed projects come first under *On CIB
GitLab*. The clone folder is `workbench.json`'s `cloneInto`, else the parent of
the most recent repository, else a picker; *Change* sets it. Esc, ↑/↓, Enter
and the focus trap are handlers on the dialog.

`RepoSlot.svelte` replaces `RepoTabs` in `TitleBar.svelte` under the switch.
`RepoTabs.svelte` stays for the Spagitty path.

### Shortcuts and the rail

Ctrl+O is a case in the layout's `shortcut`, beside Ctrl+P, and the palette
has *Load repository…* and *Unload repository*. `scoped.ts` lists the
repository screens (proposal § 3). The rail dims them with nothing loaded,
`aria-disabled`, focusable, titled "<Screen> — load a repository first", and
opens the dialog with the screen as `then`. The forge lists are not dimmed;
they are FEAT-185. A repository screen reached with nothing loaded goes to
Home and opens the dialog.

### Launch

`ResumePort` gains `onboard` and `restingRoute`, both optional, so the
existing launch tests are unchanged. Under the switch a launch that loads
nothing on purpose (FEAT-172, reworded *Open with nothing loaded*) lands on
Home.

### Migration

`workspace.svelte.ts` is unchanged: `places` and `active` are read as they
were, and `tabs` is kept, unused, so turning the switch off brings the strip
back.
