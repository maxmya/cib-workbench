<!-- SPDX-License-Identifier: MIT -->

# FEAT-168 — The launch screen stays long enough to read

**Status:** Done — in the working tree on `release/1.0.1`, uncommitted. Covered by `chrome/launch-hold.test.ts`.
**Branch:** none yet.
**Screens:** the launch screen (chrome).
**Raised by:** the author, 2026-10-26: "make splash screan last at least 2 seconds 'if the loading is too fast i want users to read the terms name and othr things on it ! wdyt?'".

## Change

- The launch screen stays at least two seconds from when the window appeared, however soon the repository opens. A slow open is not made slower.
- A click or a key lets it go at once, but only once the repository is open: somebody who reads it every morning need not wait for it. A skip pressed earlier is kept until then.
- It still gives way when the open fails.

## Decided

Two seconds on every launch is a cost paid daily by the people who already know what it says, so the skip is part of the change rather than a later fix.
