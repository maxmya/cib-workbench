<!-- SPDX-License-Identifier: MIT -->

# BUG-110 — A pipeline graph's lines cut through its cards

**Status:** Fixed — in the working tree on `release/1.1.0`, uncommitted. Covered by `pipelines/pipelines.test.ts` › *a line that jumps columns runs between the cards, never through one*.
**Branch:** none yet.
**Screens:** Pipelines (1Y).
**Raised by:** the author, 2026-10-28: "when the flow is badly written it shows awful graph", with a screenshot of twenty jobs needing one.

## Cause

A need was drawn as a curve from card to card. One that jumped columns cut through every card between them, and twenty jobs needing one build fanned twenty curves over each other.

## Fix

- Lines run square, through the gaps between cards and never through one. Jobs that need the same job share one trunk that branches.
