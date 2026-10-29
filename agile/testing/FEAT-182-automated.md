<!-- SPDX-License-Identifier: MIT -->

# FEAT-182 — Automated tests

**Item:** [`agile/items/FEAT-182-follow-repositories-on-cib-gitlab.md`](../items/FEAT-182-follow-repositories-on-cib-gitlab.md)

| Where | Asserts |
| --- | --- |
| `crates/spagitty-core/src/workbench.rs` | The listing URL (membership, not archived, most active first, 50 a page, search encoded, none when blank); a page reads its projects and whether more follow; a renamed project keeps its id and takes its new path, and an unchanged list is no change. |
| `src-tauri/src/workbench.rs` | A missing or broken `workbench.json` is the defaults; it reads back what it was given and has no token; the followed list is on the built-in host, once each, at most 50. |
| `src/lib/workbench/projects.test.ts` | The saved page paints first, then GitLab's; search waits for the typing to stop; paging appends; a failure is said and what was shown is kept; a project is followed by id with its path and default branch. |
| `src/lib/workbench/onboarding-screen.test.ts` | Step 4's choice is written only by *Open Workbench*. |
