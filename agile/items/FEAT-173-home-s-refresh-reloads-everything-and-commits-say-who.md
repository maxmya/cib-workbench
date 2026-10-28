<!-- SPDX-License-Identifier: MIT -->

# FEAT-173 — Home's Refresh reloads everything, and its commits say who

**Status:** Done — in the working tree on `release/1.1.0`, uncommitted. No test of its own: the commits card and the store's `reload` have none yet, and the screen has not been looked at.
**Branch:** none yet.
**Screens:** Home (2A).
**Raised by:** the author, 2026-10-28: "fetch all should be refresh and it refresh all content of home with fetching and reload if working copy needs refresh prs pipelines etc", and "latest commits , mention who did the commits also".

## Change

- *Fetch all* is *Refresh*. It fetches, then reads both halves of Home again: the working copy, branches and commits, and the forge's pull requests, pipelines, releases and issues. What is on screen stays until the answers come, so the cards do not empty and refill.
- Each row of *Latest commits* shows its author's face and name, then the short id, under the subject.

## Acceptance criteria

- One press of *Refresh* fetches and reloads every card.
- A commit row says who made it.
