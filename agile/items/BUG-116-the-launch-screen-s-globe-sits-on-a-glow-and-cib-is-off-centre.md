<!-- SPDX-License-Identifier: MIT -->

# BUG-116 — The launch screen's globe sits on a glow, and "CIB" is off centre

**Status:** Fixed — in 1.3.0, on `release/1.3.0`. Not yet looked at in the running application.
**Branch:** `release/1.3.0`.
**Screens:** Launch.
**Raised by:** the author, 2026-10-28: "in splash screen the O logo in workbench is not transparent background also the design of the splash looks not aligned".

## Cause

- The globe file is transparent (`assets/brand/cib/cib-globe-round.png`: RGBA, no white pixels). The launch screen drew it with an orange `drop-shadow` 0.23em wide, about 24px at 104px. The glow filled the gaps between the globe's bars, so the "o" looked as if it sat on a background.
- "CIB" is set with 0.32em letter-spacing, which also follows the last letter, so it sat 0.16em left of the wordmark under it.

## Fix

- The glow is gone, with its `glow` prop and the `--globe-glow` tokens. The file is unchanged.
- "CIB" has the same 0.32em before it, which centres it.
