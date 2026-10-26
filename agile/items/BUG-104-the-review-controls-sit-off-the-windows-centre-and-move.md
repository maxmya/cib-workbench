<!-- SPDX-License-Identifier: MIT -->

# BUG-104 — The review controls sit off the window's centre, and move

**Status:** Fixed — in the working tree on `release/1.0.1`, uncommitted. Covered by `review/pill.test.ts`; not looked at in the running application.
**Branch:** none yet.
**Screens:** Review (1R).
**Raised by:** the author, 2026-10-26: "in review screen the bottom rail shall be centered with the main bottom rain ( to not look odd ) also shall be fixed size ( it now expand a bit with file name and this change buttons location )".

## Cause

The pill was centred in the diff column, which is off the window's centre whenever the file list and the conversation differ in width, so it sat beside the toolbar under the window rather than over it. Its file name had a maximum width and no width, so a longer name pushed every button after it.

## Fix

- `centreOffset` moves the pill onto the window's centre line, as far as the column allows (8 px from either edge), measured again when the column, the pill or the window resizes.
- The file name has one width (150 px), cut with an ellipsis; *Viewed* and *Finish review* share a minimum width, so swapping one for the other moves nothing.
