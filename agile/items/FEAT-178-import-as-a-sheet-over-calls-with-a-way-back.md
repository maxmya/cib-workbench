<!-- SPDX-License-Identifier: MIT -->

# FEAT-178 — Import as a sheet over Calls, with a way back

**Status:** Done — in the working tree on `release/1.1.0`, uncommitted. Covered by `calls/ImportSheet.test.ts` and the Calls screen tests.
**Branch:** none yet.
**Screens:** Calls (2C).
**Raised by:** the author, 2026-10-28: "import screen opening in the same and it's button location and it also has no back just done ! no save ( this all flow is fucked up lets redesign it from scratch )".

## Change

- Import is a sheet over Calls in three steps, Source, Pick and Save, with *Back* and *Cancel* at each. The last button says what it will do: *Import 12 calls into orders-api*. Nothing is written until it is pressed, and Calls shows the collection when the sheet closes.
- It opens from a `+` at the top of the Collections panel: *New call*, *New collection…* and *Import calls…*.
- `ImportScreen.svelte` is gone; `/calls/import` opens the sheet.

## Acceptance criteria

- Every step has a way back, and nothing is saved before the last button.
