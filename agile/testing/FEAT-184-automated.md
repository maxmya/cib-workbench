<!-- SPDX-License-Identifier: MIT -->

# FEAT-184 — Automated tests

**Item:** [`agile/items/FEAT-184-load-repository-instead-of-tabs.md`](../items/FEAT-184-load-repository-instead-of-tabs.md)

| Where | Asserts |
| --- | --- |
| `src/lib/workbench/load.test.ts` | Loading B over A leaves B loaded, and loading A again comes back to A's route and commit; from a repository screen, the repository's own screen; from Home, stays on Home; a dimmed row's screen after the load; nothing when the open fails; loading the loaded one only navigates; unload keeps the place and leaves a repository screen for Home, or stays on a Workbench-wide one. Which routes are repository screens. |
| `src/lib/workbench/chrome.test.ts` | The switch is on; no tab strip; the slot offers Load repository, or names the loaded one with Switch and Unload; the title bar is bare while onboarding; dimmed rail rows are focusable, titled and open Load repository, then the screen; an old `spagitty.workspace` with five tabs reads without loss. |
| `src/lib/session.test.ts` | Under the switch: reload the last loaded repository on Home with its commit; a command-line path after onboarding; *Open with nothing loaded* lands on Home; the original sixteen cases unchanged. |
| `src/lib/chrome/chrome.test.ts` | Spagitty's chrome with the switch off: the tab strip and the rail's folder picker, unchanged. |
