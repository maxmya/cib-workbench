<!-- SPDX-License-Identifier: MIT -->

# FEAT-182 — Follow repositories on CIB GitLab

**Status:** Done — on `feature/workbench-2.0.0`, as Workbench 2.0.0, from the proposal's § 7. The manual sweep is still to run.
**Branch:** `feature/workbench-2.0.0`, with the other four of FEAT-180 to FEAT-184.
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

## As built

- `followed` is kept in `workbench.json`, beside `settings.json`, not in `settings.rs`: the Settings screen writes its whole settings object back on every toggle, and a stale copy would overwrite the list. `Followed` also keeps the default branch and clone address from the last listing.
- Plan: [`FEAT-182-plan.md`](../plans/FEAT-182-plan.md). Tests: [`FEAT-182-automated.md`](../testing/FEAT-182-automated.md). Sweep: [`FEAT-182-sweep.md`](../testing/FEAT-182-sweep.md).
