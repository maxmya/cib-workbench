<!-- SPDX-License-Identifier: MIT -->

# FEAT-188 — Continuous corners, and liquid glass on three things only

**Status:** Done — on `feature/workbench-2.0.0`, in Workbench 2.0.0. The manual sweep is still to run.
**Branch:** `feature/workbench-2.0.0`, with FEAT-180 to FEAT-184.
**Screens:** all (corners); the toolbar, the notice toast and every modal (glass).
**Raised by:** the author, 2026-10-29, with another project's write-up of how it draws its corners and its glass: "apply this criteria at workbench and add it to release 2.0.0".

## Change

### Corners

- Continuous corners (`corner-shape: superellipse(2)`) as progressive enhancement: a webview that draws them gets squircles, every other one the circular radii unchanged.
- A squircle looks tighter at the same radius, so each radius role has a larger, eye-calibrated value used only where squircles are drawn: field 7 → 11, button 9 → 14, row 4 → 6, panel 10 → 15, floating 16 → 24, pane 18 → 27, toast 22 → 30, window 12 → 18. `applyMetrics()` publishes them at every zoom (`CONTINUOUS_RADII`), and the `@supports` block in `src/lib/workbench/surfaces.css` paints them first.
- `corner-shape` is applied to an allow-list: the window, the pane, cards, floating surfaces, buttons, key caps, fields, the toast, and `.cc` for anything else. `[data-corner-shape='round']` opts out. `select` pickers have their own `@supports` rule.
- Pills, chips, the rail and toolbar capsules, switches, avatars and dots keep their round ends and take no squircle.
- A new radius role, `--r-toast`, replaces the toast's literal 22px. Workbench's own components write no literal radius.

### Glass

- Only on the **toolbar** (Workbench's dock, `.lrail`), the **notice toast** (`.lglass`) and the **scrim** behind a modal. Menus, dialogs, the palette, the command log and the rail are solid.
- `--lg-bg`, `--lg-rim`, `--lg-hi`, `--lg-lo`, `--lg-sheen`, `--lg-drop`, `--scrim` and `--scrim-blur` in `app.css`, redefined for Night; the glass classes name no colour of their own.
- The toolbar refracts what is behind it through `#lg-refract` (`GlassFilter.svelte`, mounted once), with the plain frost as the cascade fallback for a webview that refuses the `url()`, and a sheen behind its content.
- Every `backdrop-filter` is written with its `-webkit-` twin.
- The scrim is `var(--scrim)` with `blur(10px) saturate(120%)`, in all thirteen modals.

## Not taken

- **The lens** behind an active button: the toolbar has no selected result for one to sit behind, and so rule 5 (status not by colour alone) has nothing to apply to.
- **Reduced motion:** nothing here animates; the glass stays.

## Acceptance criteria

1. Where `corner-shape` is supported, surfaces and controls on the allow-list draw squircles at the calibrated radii; elsewhere they are unchanged. Pills, chips and capsules stay round.
2. Workbench's own components write no radius as a number.
3. Glass appears on the toolbar, the toast and modal scrims, and nowhere else.
4. The glass follows the theme in Day and Night.
5. On Windows (WebView2) the toolbar bends what is behind it; on macOS and Linux it is frosted.

## Records

Plan: [`FEAT-188-plan.md`](../plans/FEAT-188-plan.md). Tests: [`FEAT-188-automated.md`](../testing/FEAT-188-automated.md). Sweep: [`FEAT-188-sweep.md`](../testing/FEAT-188-sweep.md).
