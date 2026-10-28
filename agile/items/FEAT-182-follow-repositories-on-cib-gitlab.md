<!-- SPDX-License-Identifier: MIT -->

# FEAT-182 — Follow repositories on CIB GitLab

**Status:** Backlog — proposed in `docs/proposals/workbench-wide-home-and-load-repository.md` § 7.
**Branch:** none yet.
**Screens:** 2D step 4, Settings › Home, Home's *Change*.
**Raised by:** the author, 2026-10-28: "allow them to order the Home screen, which is renewed to fetch [selected repos or empty]."
**Depends on:** FEAT-181.

## Change

- `followed: Vec<Followed { host, id, path }>` in Workbench's settings, keyed by project id. It is never written to a repository.
- `gitlab_projects(search, page)` reads `GET /projects?membership=true&archived=false&simple=true&order_by=last_activity_at&per_page=50`, with paging and search in Rust. Results are cached through `forge/snapshot.rs`.
- The same list (search, toggle rows, *Start empty*) appears in onboarding step 4 and in Settings › Home.

## Acceptance criteria

1. The list pages and searches. A second visit paints from the cache, then revalidates.
2. A renamed or moved project stays followed (same id) and shows its new path.
3. Following nothing is a valid state, and Home shows its empty state (FEAT-183).
4. No followed data appears in any repository's `.spagitty/`.
