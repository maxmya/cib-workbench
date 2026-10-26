<!-- SPDX-License-Identifier: MIT -->

# BUG-106 — A Calls collection cannot be removed

**Status:** Fixed — in the working tree on `release/1.0.1`, uncommitted. Covered by `CallsScreen.test.ts` › *taking a collection away* and the host's state tests.
**Branch:** none yet.
**Screens:** Calls (2C).
**Raised by:** the author, 2026-10-26: "cannot remove or delete calls collections", and "able to render collection, drop it or remove it from view".

## Cause

`calls_delete_collection` existed and nothing called it: a collection's only menu item was *New call*.

## Fix

- Each collection has a `⋯`, and the same menu on a right-click: *New call*, *Hide from the list* (the file stays), and *Delete collection…*, which asks first and says which file goes. A file that could not be read can be hidden or deleted too.
- Hidden collections are kept by stem in the person's state (`hidden`) and listed at the foot of the tree as *N collections hidden*, each with *Show* and *Delete…*. Their open calls close.
