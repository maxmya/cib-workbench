<!-- SPDX-License-Identifier: MIT -->

# FEAT-169 — The rail in groups of related screens

**Status:** Done — in the working tree on `release/1.0.1`, uncommitted. Covered by `nav.test.ts`, the NavRail and Settings › Rail tests.
**Branch:** none yet.
**Screens:** the rail (chrome), Settings › Rail.
**Raised by:** the author, 2026-10-26: "better separations on rail put every collection of related functions below a separation line".

## Change

Repository had grown to fifteen rows under one line. The rows that belong together are groups of their own, each below a line:

| Group | Screens |
| --- | --- |
| Repository | Graph, Working copy, Conflicts, Branches, Merger |
| Collaboration | Pull requests, Review, Issues, Pipelines, Releases |
| Workspace | Terminal, Files, Calls, Recipes, Hooks |
| History | Log, Investigate, Reflog |
| Tools | Rebase, Stash, Tags, Untrack, Badges, All repositories |

Home, the Farm and Settings keep their own places. The line is drawn in `--line` rather than `--soft`, so it can be seen with the rail closed, when it is all that separates one group from the next. Settings › Rail shows the same groups as cards; a row still moves only within its group, and a stored order keeps working.
