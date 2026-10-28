<!-- SPDX-License-Identifier: MIT -->

# FEAT-175 — Calls' side panels widen, and the variables can be put away

**Status:** Done — in the working tree on `release/1.1.0`, uncommitted. Covered by the panel layout's tests; the drag has not been tried in the running application.
**Branch:** none yet.
**Screens:** Calls (2C).
**Raised by:** the author, 2026-10-28: "side panels ( right and left ) should be expandable", and "right panel should be hidable".

## Change

- The collections on the left and the variables on the right are each widened by dragging their edge, and keep the width (`callsTree`, `callsVars` in the panel layout).
- The variables can be put away: they become a tab down the right edge, which brings them back.

## Acceptance criteria

- Both panels can be dragged wider and remember it.
- The variables panel can be hidden and shown again.
