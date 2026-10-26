<!-- SPDX-License-Identifier: MIT -->

# BUG-102 — Closing the tab on show leaves its repository loaded

**Status:** Fixed — in the working tree on `release/1.0.1`, uncommitted. Covered by a tab-strip test that fails without the fix.
**Branch:** none yet.
**Screens:** the tab strip (chrome), and every screen behind it.
**Raised by:** the author, 2026-10-26: "when i close a repo from the tab it's data still loaded and no refresh event for the ui fired untill i switch between screens".

## Cause

`workspace.close` makes the neighbour tab active before anything opens it. `RepoTabs` then called `switchTo(next)`, whose first line returned when the tab was already active, so `repo.open(next)` never ran. The strip showed the new tab; everything under it still showed the closed repository until something else moved.

## Fix

`switchTo` returns early only when the tab is active *and* is the repository that is open (`repo.info.path`).

## Tests

- `chrome.test.ts` › *opens the neighbour when the tab on show is closed (BUG-102)*.
