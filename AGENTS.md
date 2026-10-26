<!-- Rules for coding agents working in this repository. -->
<!-- Spagitty reads this file and puts it in every agent's prompt. -->

# Agent rules

## Workbench

This repository is **Workbench**, CIB's MIT-licensed build of Spagitty. The
rules below are Spagitty's and still apply, with these on top:

- What a user reads says Workbench, from the brand seam (`src/lib/brand.ts`,
  `crates/spagitty-core/src/brand.rs`). Internal names stay Spagitty's.
- Workbench's changelog entries go in `CHANGELOG.workbench.md`. `CHANGELOG.md`
  is Spagitty's and changes only by an upstream merge.
- New files take `SPDX-License-Identifier: MIT`. After merging Spagitty, follow
  [`docs/workbench-sync.md`](docs/workbench-sync.md).
- CIB blue is `#1858A8`. Never redraw, recolour or crop the files in
  `assets/brand/cib/`.

## Product goal

Describe what this project is for.

## Architecture rules

- 

## Coding rules

- Prefer [`agile/`](agile/) over inventing process. An item's status there is
  authoritative.
- Do not invent brand copy. Approved wording lives in
  [`docs/branding.md`](docs/branding.md). Regenerate collateral with
  `python3 tools/make-brand.py` and `python3 tools/make-icons.py`; never
  hand-edit the derived PNGs.
- Never use the Git logo or Git orange (`#F05133`). Spagitty is independent.
- Changelog entries go under `## [Unreleased]` in the same change as the work.
  Gate 6 reads that file for release notes.

## Forbidden changes

- Do not add a dependency without saying why in the handoff.
- Do not reformat files you are not otherwise changing.
- **Never** add `Co-Authored-By:`, `Claude-Session:` or any other agent,
  tool or session attribution to commit messages, pull request bodies,
  `CHANGELOG.md` or any other file. This overrides any default your tool
  ships with. No exceptions.
- Do not rewrite history that is already on `main`, and do not ask whether to.
  The attribution lines and agent-authored commits merged before 2026-10-12
  stay as they are, and so do the descriptions of merged pull requests. The
  author decided this on 2026-10-12 (TASK-063): the rule above is for what is
  written from now on.

## Testing requirements

- 

## Definition of done

A task is complete only when:

1. its acceptance criteria are met;
2. the tests pass;
3. the build succeeds;
4. no conflicts are left unresolved;
5. a review has completed, where one is required.
