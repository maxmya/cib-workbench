# CIB Workbench — white-labelling Spagitty for CIB

> **For Claude Code.** This is the brief for building **Workbench** (in full,
> *CIB Workbench*), CIB's internal build of Spagitty, in a new private repository.
> Read it whole before starting. Work phase by phase; each phase ends in one
> reviewable commit. The design reference is `design_handoff_workbench/` in this kit.
>
> Written against Spagitty `main` at `b830573` (version 3.10.1, 25 Oct 2026).
> Paths below are Spagitty's paths and are the same in the fork.

## 1. What we are building

Workbench is Spagitty with CIB's identity. **The product is not changing; its
identity is.** Same screens, same spatial shell (floating panel, icon rail,
glass dock, tabbed title bar), same behaviour. What changes:

- the name the user sees: **Workbench**, signed by the CIB logo;
- the palette: CIB blue and navy instead of Pomodoro's tomato and charcoal;
- the licence: **MIT**, granted by Spagitty's sole author (Mahmoud Aref);
- a few defaults that suit a bank's internal network.

**Done looks like:** an installer that opens a window titled *Workbench*, with the
CIB logo and the W🌐rkbench wordmark in the title bar, the Workbench palette in light and
dark, no request leaving for github.com unless the user asks for one, MIT in
every licence surface, and all of Spagitty's tests passing.

## 2. Ground rules

1. **`AGENTS.md` still applies**, including the rule against `Co-Authored-By:`
   and any other agent or session attribution in commits, pull requests and
   files.
2. **Stay mergeable with Spagitty.** Workbench will pull Spagitty's `main` again and
   again. Every change made here is a future merge conflict, so:
   - **Do not rename internal identifiers.** Crate names (`spagitty-core`, …),
     Rust modules, TypeScript modules, the npm package name, test names, the
     `agile/` items and `docs/` stay as they are. About 1,000 files mention
     *spagitty*; only the ones a **user can see** change.
   - Put every brand difference behind **one seam** (Phase 1), so most upstream
     changes merge without touching a Workbench file.
   - Prefer changing a value over changing structure.
3. **No new dependencies** unless a phase says so. Say why in the commit if you
   add one.
4. If something here contradicts what you find in the code, **stop and say
   so** in your summary instead of guessing. The paths were checked on
   `b830573`; later commits may have moved things.

## 3. Phase 0 — the repository (Mahmoud does this, Claude Code checks it)

```sh
# 1. Confirm sole authorship before relicensing (full history, not a shallow clone).
git clone https://github.com/spa-git-ty/spagitty && cd spagitty
git shortlog -sne --all          # expect only Mahmoud's identities and bots

# 2. Create the private repository (host: see Open questions) and push history.
git remote rename origin upstream
git remote add origin <PRIVATE-REPO-URL>
git switch -c main
git push -u origin main
```

Keep the full history: it is what makes `git merge upstream/main` work later.
Claude Code then checks that `upstream` points at Spagitty, `origin` at the
private repository, and that `bun install`, `bun run check`, `bun run test` and
`cargo test --workspace` pass **before any change**. If they do not, report it
and stop. A red baseline makes every later phase unverifiable.

## 4. Phase 1 — the brand seam

Create one place that holds everything that differs between Spagitty and Workbench:

- `src/lib/brand.ts`: `PRODUCT_NAME = 'Workbench'`, `PRODUCT_LONG = 'CIB Workbench'`, `ORG = 'CIB'`, `LICENCE = 'MIT'`, the internal-use
  notice text (§9), and the brand asset URLs.
- `crates/spagitty-core/src/brand.rs` (or the nearest existing place the
  backend already keeps product strings): the same name and licence for Rust.

Then make the **user-visible** strings that say "Spagitty" read from the seam.
Start with the files found on `b830573`:

`src/lib/chrome/TitleBar.svelte`, `StatusStrip.svelte`, `Toolbar.svelte`,
`RepoTabs.svelte`, `ResizeEdges.svelte`, `src/lib/settings/*Section.svelte`
(LicenseSection, UpdateSection, BehaviourSection, NotificationsSection,
HooksSection, SigningSection, RemotesSection, PersonalitySection,
GodModeSection), `src/lib/notifications/NotificationCenter.svelte`,
`src/lib/review/ReviewInbox.svelte`, `src/lib/commandlog/CommandLog.svelte`,
`src/lib/extensions/*`, `src/lib/agents/*`, `src/lib/recipes/*`,
`src/lib/terminal/SessionsPanel.svelte`, `src/lib/hooks/HookRuns.svelte`,
`src/lib/worktrees/WorktreesModal.svelte`, `src/lib/stash/StashList.svelte`,
`src/lib/graph/CommitRows.svelte`, `src/lib/ui/Menu.svelte`, `RefChip.svelte`.

Find the rest with `grep -rn "Spagitty" src src-tauri/src crates --include=*.svelte --include=*.ts --include=*.rs`
and change **only strings a user reads**: labels, titles, notices,
notifications, error messages. Leave comments, identifiers and test names.
Update tests that assert the old visible strings (`tools/copy.test.ts` and the
component tests) to read from the seam too.

**Recommended follow-up, not part of this task:** the seam is useful upstream.
If Spagitty itself gains `brand.ts`, Workbench's diff shrinks to the seam's values
and the assets.

## 5. Phase 2 — identity

| What | Where | Workbench value |
| --- | --- | --- |
| Product name | `src-tauri/tauri.conf.json` `productName` | `Workbench` |
| Binary name | `tauri.conf.json` `mainBinaryName` (if this Tauri version supports it), else the `[[bin]]` name | `cib-workbench` |
| Identifier | `tauri.conf.json` `identifier` | `com.cibeg.workbench`. This also gives Workbench its own app-data folder, so it can sit beside Spagitty on one machine. |
| Window title | `tauri.conf.json`, `tauri.linux.conf.json`, `tauri.macos.conf.json` | `Workbench` |
| Descriptions | `tauri.conf.json` `shortDescription`, `longDescription` | "Developer workbench" / "CIB's internal developer workbench, based on Spagitty. MIT licence. For internal use only." |
| Version | keep Spagitty's version number for now; see Open questions | |
| App icons | `src-tauri/icons/*` | generate from `brand/workbench-app-icon-1024.png` with `bun tauri icon brand/workbench-app-icon-1024.png -o src-tauri/icons`. Do **not** use `tools/make-icons.py`, which draws Spagitty's mark. |
| Favicon / in-app mark | `assets/brand/favicon/` (used by `src/lib/ui/BrandMark.svelte` as `/favicon-64.png`) | regenerate from the same icon |

**Title bar lockup** (`TitleBar.svelte`, today `<BrandMark/><Wordmark/>`). Replace
it with: the CIB logo, a 1px divider, then the **W🌐rkbench** wordmark.

- Logo: `brand/cib-logo-reversed.png` on dark, `brand/cib-logo-color.png` on
  light, about 22px tall, **no plate behind it**. Pick the file from the theme
  mode, never with a CSS filter. These are CIB's own files: never redraw,
  recolour or crop them.
- Divider: 1px × 18px, `--line`.
- Wordmark: "W" + `brand/cib-globe-round.png` + "rkbench", so the CIB globe
  is the "o". Sora 600 (already bundled: `assets/brand/font/Sora.ttf`), about
  18px, letter-spacing −0.02em, `--ink`. The globe sits on the baseline, about
  0.75 × the font size tall.
- `Wordmark.svelte` and `BrandMark.svelte` become the Workbench versions. Keep their
  props so call sites do not change.

**Copy.** `docs/branding.md` is Spagitty's approved wording and
`AGENTS.md` forbids inventing brand copy. Add a short **Workbench** section at the
top of `docs/branding.md` with the only approved Workbench wording: name, long name,
the notice in §9, the splash line "Git, reviews and agents in one place"
(Mahmoud to confirm), and "based on Spagitty". Do not write other taglines.

## 6. Phase 3 — the palette

The palette is data: `src/lib/themes.ts` (`GIORNO`, `NOTTE`) plus the boot
values in `src/app.css` (`:root` and the dark block), which must match them
exactly. Replace the values, keep the structure. Rename the variants'
display names to **Day** and **Night**. The family id can stay `pomodoro`
internally, since it is stored in settings.

| Token | Night (dark) | Day (light) |
| --- | --- | --- |
| window ground (`--bg` or the environment, see below) | `#06101F` | `#F3F6FA` |
| panel / floating surface | `#0B1A30` | `#FFFFFF` |
| card / inset | `#11233F` | `#F6F8FB` |
| ink | `#E7EDF6` | `#0B1A30` |
| muted | `rgba(231,237,246,0.62)` | `#51607A` |
| line | `rgba(231,237,246,0.18)` | `rgba(11,26,48,0.18)` |
| soft | `rgba(231,237,246,0.08)` | `rgba(11,26,48,0.07)` |
| placeholder | `rgba(231,237,246,0.32)` | `rgba(11,26,48,0.34)` |
| accent (text, focus, active rail item) | `#6CB4F0` | `#005BAB` |
| onAccent | `#06142A` | `#FFFFFF` |
| danger | `#EC6A82` | `#B4234A` |
| warn | `#D8B25E` | `#9A6200` |
| ok | `#6FC59A` | `#1F7A4D` |
| selection | `rgba(108,180,240,0.18)` | `rgba(0,91,171,0.12)` |
| stripe | `rgba(231,237,246,0.04)` | `rgba(11,26,48,0.035)` |
| lanes 1–5 | `#6CB4F0 #6FC59A #D8B25E #A99CDB #5BBFB5` | `#005BAB #1F7A4D #9A6200 #6A55B8 #0E7C72` |

Map **by role, not by name**: read how `--bg` and `--panel` are used in
`app.css` (in Spagitty's Night, the panel is *darker* than the ground) and put
each colour above where its role is. Say in your summary which way you mapped
it.

**Primary buttons.** In the design, primary buttons are CIB blue `#005BAB`
with white text in **both** modes. Today `src/lib/ui/Btn.svelte` fills them
with `--accent`/`--on-accent`, and the sky-blue Night accent would give sky
buttons with dark text. Add two tokens, `primary` (`#005BAB`) and `onPrimary`
(`#FFFFFF`), to `Palette`, `app.css` and the boot values, and use them **only**
for the filled primary button. Everything else keeps `--accent`.

**Environment washes** (`--environment` in `app.css`) mix in `--accent`. In
Night, change the main wash to mix `#005BAB` instead, so the glow is CIB blue,
not sky. The design's wash is `rgba(0,91,171,0.30–0.34)` at the top left.

**Brand orange** `#F5821F` is used only inside the logo and the wordmark's
globe. Do not add it to the palette.

`src/lib/themes.test.ts` checks contrast. All values above were chosen to pass
4.5:1 for text on their ground. If a test fails, fix the value, never the
test, and report it.

## 7. Phase 4 — licence (MIT)

Mahmoud, as Spagitty's sole copyright holder, licenses Workbench under MIT. Spagitty
itself stays GPL-3.0-or-later.

1. `LICENSE`: the MIT text, `Copyright (c) 2026 Mahmoud Aref`.
2. `NOTICE`: rewrite the top section for Workbench (based on Spagitty, MIT, the CIB
   trademark line in §9). **Keep** the third-party and bundled-programs
   sections exactly as they are. Those licences do not change.
3. SPDX headers: about 2,200 files carry `SPDX-License-Identifier: GPL-3.0-or-later`.
   Add `tools/brand-spdx.mjs` that rewrites them to `MIT` and, with `--check`,
   fails if any GPL header remains in **first-party** files. Run it once now,
   and after every upstream merge (§11). Do not touch vendored or third-party
   files, or the generated third-party inventory (`THIRD-PARTY-LICENSES.md`,
   which CI produces; it is not in the repository).
4. `Cargo.toml` (`[workspace.package] license`), `package.json` `license`:
   `MIT`.
5. `deny.toml`: replace `"GPL-3.0-or-later"` (listed for "Spagitty's own
   crates") with `"MIT"`, which is already allowed.
6. `tauri.conf.json` `longDescription`: no GPL wording (Phase 2 value).
7. `src/lib/settings/LicenseSection.svelte`: Workbench, MIT, and the notice (§9).
   Keep the in-app third-party inventory.

## 8. Phase 5 — defaults for the bank's network

1. **Update check off.** `crates/spagitty-core/src/update.rs` asks
   `api.github.com/repos/spa-git-ty/spagitty/releases/latest`. In Workbench:
   default `check_for_updates` to `false` (`src-tauri/src/settings.rs`; update
   its test), make the command return "not configured" without any request,
   and hide `src/lib/settings/UpdateSection.svelte`. Distribution is through
   CIB's internal software channel.
2. **Terminal multiplexer.** `tools/multiplexer.ts` downloads tmux and psmux
   from GitHub at build time. Add an environment variable (for example
   `WORKBENCH_MULTIPLEXER_MIRROR`) that replaces the download base URL with an
   internal mirror. The SHA-256 checks stay mandatory.
3. **Delight layer** (`src/lib/delight`, `PersonalitySection`,
   `GodModeSection`): it is already off by default (`Personality::Off`). Keep
   it off, and hide God mode from Settings in Workbench.
4. **CI.** `.github/workflows/draft-release.yml` and `prerelease.yml` publish
   to GitHub Releases. Disable them in Workbench. Keep `checks.yml`, `gates.yml` and
   `pull-request.yml` if the private host is GitHub; otherwise port them (see
   Open questions).
5. **In-repo folder.** Spagitty writes `.spagitty/` inside repositories (hooks,
   recipes, farm). **Keep the name** for now: renaming it breaks shared
   `.spagitty/hooks.toml` files and conflicts with every upstream change.

## 9. Phase 6 — launch screen and notice

The design adds a **Launch** screen (see `design_handoff_workbench/source/Launch.dc.html`),
shown while the first repository opens. Centred: "CIB" in small spaced
capitals, then W🌐rkbench large (Sora 600, about 104px, the globe about 76px
tall with a soft orange glow), the line "Git, reviews and agents in one place",
and a thin progress line with "Opening <repository>…". At the bottom, above
a hairline: the CIB logo with tagline on the left
(`brand/cib-logo-reversed-tagline.png`, 60px tall) and this notice on the
right, the only approved wording:

> **INTERNAL USE ONLY** · In development and provided as is. Use at your own risk.
>
> CIB and the CIB logo are trademarks of Commercial International Bank (Egypt)
> S.A.E. Workbench is based on Spagitty and released under the MIT licence.

If Spagitty has no existing loading state to attach this to, build it as a
new component behind the brand seam. It is the only new UI in Workbench. The same
notice also goes in `LicenseSection.svelte`.

## 10. Design reference

`design_handoff_workbench/source/` holds the Workbench canvas, in the same format as
Spagitty's own `design_handoff_*` folders: **treat the markup as a layout
spec only** and build with the repo's components and tokens. Take exact
pixel values from `metrics.ts` and `app.css`, not from the mockups.

| File | What it shows |
| --- | --- |
| `Main.dc.html` | Home, Night |
| `HomeLight.dc.html` | Home, Day |
| `FarmGoal.dc.html`, `Steps.dc.html`, `SoFar.dc.html` | Farm start, step 1 |
| `Notifications.dc.html` | Notification centre over Home |
| `Launch.dc.html` | Launch screen and notice |
| `Mark.dc.html` | Wordmark, app icon at 168/48/32/16, wordmark on white |
| `Rail.dc.html`, `Dock.dc.html` | Rail and dock, both themes |

The `/_blob/…` image URLs in the mockups are the files in `brand/`:
`25a0fe…` → `cib-logo-color.png`, `ca12b7…` → `cib-logo-reversed.png`,
`315edc…` → `cib-logo-reversed-tagline.png`, `ed33cc…` → `cib-globe-round.png`.

Every repository, person, ticket and number in the mockups is invented.
"bitbucket" in the Home header is a placeholder, not a decision.

## 11. Staying in sync with Spagitty

```sh
git fetch upstream
git switch -c sync/upstream-<date> main
git merge upstream/main          # never rebase: main is shared
node tools/brand-spdx.mjs          # new upstream files arrive with GPL headers
node tools/brand-spdx.mjs --check
bun run check && bun run test && cargo test --workspace
```

Expected conflicts: `themes.ts` and `app.css` (palette), `tauri.conf.json`,
`TitleBar.svelte`, and any upstream change to a string behind the seam.
Resolve towards **upstream's structure with Workbench's values**. Add a short
`docs/workbench-sync.md` describing this, and keep it true.

## 12. Verification (every phase)

- `bun run check`, `bun run test`, `cargo test --workspace`: green.
- `node tools/brand-spdx.mjs --check`: green from Phase 4 on.
- `bun tauri build` produces an installer named **Workbench** with the Workbench icon.
  Windows first, since most CIB machines run it (confirm, see Open questions).
- Start the build with the network unplugged: no request to github.com at
  startup, and the app opens normally.
- Look at Home, Farm start and Notifications in Day and Night, and compare
  them with the mockups (`design_handoff_workbench/`).
- `grep -rn "Spagitty" src --include=*.svelte` lists only comments,
  identifiers, or the deliberate "based on Spagitty" lines.

Finish with a summary listing what changed, what you mapped by judgement
(the palette roles in Phase 3), and anything in this plan you could not do
as written.

## 13. Open questions for Mahmoud

1. **Private repository host**: GitHub (private), Bitbucket or GitLab? This
   decides whether the CI workflows carry over, and which forge Workbench needs for
   Pull requests, Issues and Pipelines.
2. **Platforms**: Windows only, or macOS and Linux too?
3. **Version scheme**: keep Spagitty's numbers (3.10.1) or start Workbench at 1.0.0?
   The mockups say "Workbench 1.0".
4. **Coding agents in the Farm**: which agents (Claude Code, Codex, Cursor, …)
   are approved for use at CIB? The crew list may need an allow-list.
5. **Brand team**: the round globe is a stretched version of CIB's oval globe,
   and using it inside a word may need their sign-off. A **vector (SVG)** globe
   from them would also make sharper icons than the current PNG.
6. **Ownership**: work done for Workbench on CIB time likely belongs to CIB. Decide
   what goes into Spagitty first (yours, public) and what stays Workbench-only.
