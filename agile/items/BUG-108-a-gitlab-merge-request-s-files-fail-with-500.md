<!-- SPDX-License-Identifier: MIT -->

# BUG-108 — A GitLab merge request's files fail with 500

**Status:** Fixed — in the working tree on `release/1.1.0`, uncommitted. Covered by four `forge/gitlab.rs` tests with simulated answers; not tried against the reporter's server.
**Branch:** none yet.
**Screens:** Pull requests (1H), Review (1R).
**Raised by:** the author, 2026-10-28: "when i open one of them i get : … 500 Internal Server Error … ( although lis of commits is readed ) but not files".

## Cause

The files came only from `/merge_requests/:iid/diffs`, which reads the diffs GitLab stored at the last push. A server that cannot read its stored diffs answers 500, while the commits, read another way, load.

## Fix

- On a server error, ask again with `access_raw_diffs=true`, which has GitLab work them out from the repository. If that fails too, compare the merge request's base and head with `/repository/compare`.
- When every way fails, the first server error is what is said. A refusal (401, 403, 404) is not gone round. A server too old for `/diffs` reads `/changes`, as before.
