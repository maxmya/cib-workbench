<!-- SPDX-License-Identifier: MIT -->

# BUG-109 — The review bar wraps to a second row

**Status:** Fixed — in the working tree on `release/1.1.0`, uncommitted. Covered by `review/pill.test.ts` › *how tight the review pill is*.
**Branch:** none yet.
**Screens:** Review (1R).
**Raised by:** the author, 2026-10-28: "the buttom rail that shouws files … colabs when i zoom in or side panel is larg , so it should get smaller to a fixed size its content should not go to another row".

## Cause

The bar had one size. With less room it wrapped onto a second row.

## Fix

- The bar stays on one row and is drawn tighter in steps as room shrinks: shorter labels, then less padding, down to a smallest size it keeps.
