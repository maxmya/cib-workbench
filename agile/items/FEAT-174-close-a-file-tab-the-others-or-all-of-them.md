<!-- SPDX-License-Identifier: MIT -->

# FEAT-174 — Close a file tab, the others, or all of them

**Status:** Done — in the working tree on `release/1.1.0`, uncommitted. Covered by `files/store.test.ts` › *closes every tab but one, and shows that one* and *closes every tab*.
**Branch:** none yet.
**Screens:** Files (1U).
**Raised by:** the author, 2026-10-28: "in tabs i shall have right click on tab close this tab close others or close all".

## Change

- A right-click on a tab opens a menu: *Close*, *Close others* and *Close all*. Closing the others shows the tab kept; closing all leaves Files with no file open.

## Acceptance criteria

- Each of the three closes what it says and nothing else.
