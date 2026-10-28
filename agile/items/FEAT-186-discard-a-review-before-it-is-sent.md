<!-- SPDX-License-Identifier: MIT -->

# FEAT-186 — Discard a review before it is sent

**Status:** Done — in 1.3.0, on `release/1.3.0`. Covered by `routes/review/room.test.ts` › *discards the review*.
**Branch:** `release/1.3.0`.
**Screens:** Review (the room's Finish review card).
**Raised by:** the author, 2026-10-28: "no way on review screen to discard review or delete a review, add this".

## Change

- Finish review has *Discard*. After a confirmation that counts the pending comments, it forgets the pull request's record: every pending comment, the summary and the viewed files.
- Nothing reaches the host before Finish review, so nothing is taken back there.
- *Discard* is disabled when there is nothing to throw away.
- `review.forgetRecord` uses the backend's existing forget (`set_review_state` with null); no new command.
