<!-- SPDX-License-Identifier: MIT -->

# BUG-114 — Calls' open calls wrap onto a second row

**Status:** Fixed — in 1.3.0, on `release/1.3.0`. Covered by `calls/CallsScreen.test.ts` › *keeps the open calls in one row that scrolls*.
**Branch:** `release/1.3.0`.
**Screens:** Calls (2C).
**Raised by:** the author, 2026-10-28: "in calls screen the tabs should be scrollable when they are allot for the screen, now they make 2nd row and go on!"

## Cause

The header and the open calls' strip both wrapped (`flex-wrap: wrap`). More open calls than the width could hold went onto a second row, then a third, and pushed the request down.

## Fix

- The header is one row. The strip takes the width the title, scope, Env and Share leave it, and scrolls sideways when the calls do not fit.
- The call shown scrolls into view when it is opened or chosen, so a call opened from the tree is never off the end of the strip.
