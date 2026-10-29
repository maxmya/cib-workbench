<!-- SPDX-License-Identifier: MIT -->

# FEAT-180 — Plan

**Item:** [`agile/items/FEAT-180-first-launch-onboarding.md`](../items/FEAT-180-first-launch-onboarding.md)
**Branch:** `feature/workbench-2.0.0`, as 2.0.0.
**Status:** implemented.

## Approach

### Steps as a pure module

`src/lib/workbench/onboarding.ts`: `next`, `back`, `canGoOn` and `marks` over
`{ step, page }`. Welcome has three pages; with an account already connected
for the built-in host, GitLab and Token are passed over both ways and drawn
done.

### The screen

`Onboarding.svelte` at `/welcome`. The layout draws a bare title bar (window
buttons and drag area), no rail and no status strip while `onboard.showing`.
Steps 2 and 3 reuse FEAT-181's `TokenPage` (with its numbered guide) and
`TokenField`; step 4 reuses FEAT-182's `FollowList` and `SectionsOrder` over
`WIDE_HOME_CARDS`. What step 4 chooses is held in the screen and written only
by *Open Workbench*: the followed list, `HomeLayout`, then
`onboarded = { version: 1, at }`.

*Set up later* on steps 2 and 3 goes to Home and writes nothing.

### Launch

`onboard.svelte.ts` holds whether onboarding is on screen and resolves the
launch's wait when it finishes or is put off. The layout reads
`workbench.json` before the launch, and `resumeSession`'s `onboard` shows
`/welcome` and waits, so a command-line path loads after it. The launch
screen waits for the whole launch, so it gives way as onboarding starts.

### Run setup again

Settings › Behaviour, `RunSetupAgain.svelte`: clears `onboarded` and opens
`/welcome` with nothing waiting. Accounts are kept.

## Copy

The designs' wording is UI copy (proposal § 15.7), trimmed where it explained
rather than labelled (the author's 2026-10-28 note on chatter). The splash
line is `SPLASH_LINE`; the notice is `NOTICE`. No new brand copy.
