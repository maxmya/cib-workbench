<!-- SPDX-License-Identifier: MIT -->

# FEAT-180 — First-launch onboarding

**Status:** Done — on `feature/workbench-2.0.0`, as Workbench 2.0.0, from the proposal's § 5. The manual sweep is still to run.
**Branch:** `feature/workbench-2.0.0`, with the other four of FEAT-180 to FEAT-184.
**Screens:** 2D Onboarding (new, `/welcome`), Settings › Behaviour.
**Raised by:** the author, 2026-10-28: "when one installs it the first time, they get onboarded to a screen that has Next buttons and explains what this software is."
**Depends on:** FEAT-181, FEAT-182, FEAT-183 (Home layout).

## Change

- A full-window sequence with no rail: *Welcome* (three pages) · *GitLab* · *Token* · *Home*. Step marks at the top; Back, page dots and Next at the bottom; the CIB logo and the approved notice below a hairline.
- Shown on launch when `onboarded` is unset and `FEATURES.workbenchWide` is on. When an account for `GITLAB.host` is already connected, steps 2–3 are skipped.
- Finishing writes `onboarded = { version: 1, at }` and lands on Home.
- Steps 2–3 carry *Set up later*, for someone off the VPN or with a locked keychain. It goes to the empty Home and leaves `onboarded` unset.
- Settings › Behaviour › *Run setup again*.
- A command-line path is loaded after onboarding finishes.

## Acceptance criteria

1. A fresh profile opens on `/welcome`. A profile with `onboarded` set opens on Home.
2. Back and Next move through all six pages. Back is hidden on the first page. The dots select a Welcome page.
3. Next on the Token step is disabled until FEAT-181's check succeeds. *Set up later* is always available on steps 2–3. It reaches the empty Home, and the next launch offers setup again.
4. Step 4 writes the followed repositories (FEAT-182) and `HomeLayout` (FEAT-148) only when Workbench is opened, not on every toggle.
5. *Run setup again* clears `onboarded` and opens `/welcome`. Existing accounts are kept, and steps 2–3 show "Connected as @login".
6. With `FEATURES.workbenchWide` off, nothing changes.
7. `resumeSession` tests cover onboarding-first and command-line-after-onboarding.

## As built

- `onboarded` is kept in `workbench.json`, with FEAT-182's list, for the same reason.
- Step 2's side panel is a numbered guide to 16.6.2's token page (see FEAT-181).
- The designs' sentences were trimmed where they explained rather than labelled. Welcome's first page shows the wordmark and the approved splash line only: the design's paragraph describing the product is not in `docs/branding.md`, so it waits for the author to approve one there.
- Plan: [`FEAT-180-plan.md`](../plans/FEAT-180-plan.md). Tests: [`FEAT-180-automated.md`](../testing/FEAT-180-automated.md). Sweep: [`FEAT-180-sweep.md`](../testing/FEAT-180-sweep.md).
