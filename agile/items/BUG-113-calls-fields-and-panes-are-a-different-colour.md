<!-- SPDX-License-Identifier: MIT -->

# BUG-113 — Calls' fields and panes are a different colour

**Status:** Fixed — in the working tree on `release/1.1.0`, uncommitted. Covered by `calls/varfield-paint.test.ts`.
**Branch:** none yet.
**Screens:** Calls (2C).
**Raised by:** the author, 2026-10-28: "inputs and response/request areas are odd colored not matching the app", and "remove those unessary part ‘Type {{ anywhere to insert one. Secrets stay on this computer and never go into the repository.’".

## Cause

Calls' fields were a well a shade under the card, and the request and response were dark slabs inside it.

## Fix

- Fields and the request and response areas are flat: the card's own colour, with a hairline and the accent ring on focus.
- The line under the variables saying how to insert one is gone.
