<!-- SPDX-License-Identifier: MIT -->

# FEAT-176 — Hooks in two tabs: Current hooks and Create hook

**Status:** Done — in the working tree on `release/1.1.0`, uncommitted. Covered by `hooks/order.test.ts`, `hooks/builder.test.ts`, `hooks/steps.rs` and `hooks.rs`.
**Branch:** none yet.
**Screens:** Hooks (1X).
**Raised by:** the author, 2026-10-28: "we should have 2 tabs , current hooks ( read from the repo , display its code and filter its category and list all of them and our algorithm say which runs first and on which stage from its type ) , second tab is Create Hook that allow devs to see template hooks that we can add to the repo ! and as we do they are categorized and explained".

## Change

- **Current hooks** lists every step the repository's hooks run, read from Husky, lefthook, pre-commit or plain git hooks, numbered in the order git runs them and grouped by the moment they run in (commit, message, push). Each step has a category, found from the program it runs, and a filter by category keeps each step's number. Choosing one shows its file with the step's line marked.
- **Create hook** has 18 templates in categories, each saying what it does and when it runs. Adding one writes it the way the repository keeps its hooks: appended to `.husky/<hook>`, as a Workbench block, or as lines to copy for lefthook or pre-commit. A template the repository already runs says so.
- *Runs* stays one button away.

## Acceptance criteria

- Current hooks shows every step, in run order, filterable by category.
- A template can be added and then appears in Current hooks.
