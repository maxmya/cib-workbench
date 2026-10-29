<!-- SPDX-License-Identifier: MIT -->

# FEAT-183 — Automated tests

**Item:** [`agile/items/FEAT-183-home-across-followed-repositories.md`](../items/FEAT-183-home-across-followed-repositories.md)

| Where | Asserts |
| --- | --- |
| `crates/spagitty-core/src/workbench.rs` | Requests across the host and runs and releases per followed project, each row with its project; usernames compared without case; one failing project is its own error and the others still show, a 403 being "none"; no more than four requests in flight, and they overlap; a project read within the interval is not read again, nor are the host-wide lists; only followed projects are read one by one, at most fifty; nothing returned can carry a token. |
| `src/lib/workbench/home.test.ts` | Needs you: reviews anywhere, failed runs and assigned issues, each with its project; Merge requests and Issues list followed projects only; *<repo> only* filters every card and keeps host-wide errors; the Calls card's three collections used last. |
| `src/lib/workbench/wide-home-screen.test.ts` | With nothing loaded every card fills and every row names its project; one project's failure shows in its card; a row in a project that is not loaded opens Load repository with it chosen; the empty state with *Choose repositories* and *Load a repository*, and Calls still shown; cards in the chosen order. |
| `src/lib/home/home.test.ts` | Unchanged and passing: Spagitty's Home and `HomeLayout` with the default card list. |
