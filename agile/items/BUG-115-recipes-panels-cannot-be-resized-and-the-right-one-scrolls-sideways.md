<!-- SPDX-License-Identifier: MIT -->

# BUG-115 — Recipes' panels cannot be resized, and the right one scrolls sideways

**Status:** Fixed — in 1.3.0, on `release/1.3.0`. Covered by `routes/recipes/page.test.ts` › *its panels* and `panels.test.ts`. The sideways scroll has not been looked at in the running application.
**Branch:** `release/1.3.0`.
**Screens:** Recipes (1V).
**Raised by:** the author, 2026-10-28: "Recipes screen panels are not expandable ! both right and left should be expandable , right should be hidable , right is broken ( scrolling horizontally )".

## Cause

- The blocks and the run panel had fixed widths (`flex: 0 0 244px` and `300px`), with no splitter.
- A flex item is as wide as its content unless told otherwise. One long line in the folded-away script, or an agent's long name in the select, made the run panel wider than itself, so it scrolled sideways.

## Fix

- Both sides drag wider or narrower, through the same splitter and stored widths as Calls (`recipesBlocks`, `recipesRun`).
- The run panel can be put away with its chevron and brought back from a strip on the right. Ctrl+Enter still runs while it is away.
- No part of the run panel is wider than the panel. The script wraps long lines, and the level buttons shorten with an ellipsis.
