<!-- SPDX-License-Identifier: MIT -->

# Handoff: onboarding, a Workbench-wide Home, and Load repository

What it should look like. The proposal, `docs/proposals/workbench-wide-home-and-load-repository.md`, says what to build. Where the two disagree, the proposal wins.

| File | What it is |
| --- | --- |
| `source/*.dc.html` | Design Component sources. **Treat the markup as a layout spec only.** Throw away the DC runtime, and build with the repo's own components and tokens. |
| `source/canvas.json` | How the artboards sit on the canvas. |

`Main.dc.html` is the whole onboarding as one component, because its steps share state. The `step` prop (1–4) and `page` prop (0–2) set where it starts. `OnboardGitLab`, `OnboardToken` and `OnboardHome` only mount `Main` at steps 2, 3 and 4.

`Home.dc.html` is Home, the rail, the title bar slot and the Load repository dialog, again as one component. The `start` prop is `following`, `empty`, `picker` or `loaded`. `HomeEmpty`, `LoadRepo` and `HomeLoaded` mount it in those states.

## Behaviour shown in the prototypes

- **Onboarding.**
  - Next / Back / dots on the Welcome pages.
  - *Open token page* and *Copy link*.
  - The token field: Check is disabled while the field is empty; it says *Checking…*, then shows the account, or an error for a short token.
  - *Next* is disabled on the Token step until the check passes.
  - Repository toggles, search, *Start empty*.
  - Section up/down and on/off.
  - *Open Workbench* goes to Home, or to the empty Home when nothing is followed.
- **Home.**
  - The title bar slot opens the dialog.
  - Dimmed repository rows on the rail open the dialog.
  - Picking a row shows *Loading…* or *Cloning…*, then the loaded state: chip, Switch, Unload, the Loaded band, the rail no longer dimmed.
  - *All followed* / *<repo> only* filters every card.
  - The empty state's *Choose repositories* fills Home.

## Images

The designs reference images as `/_blob/<id>`. Use the repository's files through `BRAND_ASSETS` in `src/lib/brand.ts`:

| Id | File |
| --- | --- |
| `8ed2904807391f8815df61b310da6e3b` | `assets/brand/cib/cib-globe-round.png` (the wordmark's "o") |
| `a3e65eef4d83f5b2b8873571038ea10e` | `assets/brand/cib/cib-logo-reversed.png` |

## Fidelity

High for structure, copy and the meaning of every colour. Night only; Day follows from the existing tokens. Every repository, merge request, person, count and time is made up. `[COMPANY GITLAB URL]` is a placeholder for `GITLAB.host`. The palette follows the 2026-10-26 corrections in `design_handoff_cib_workbench/README.md`: CIB blue `#0C529A`, a black Night, and accent text `#4D8FD6`.
