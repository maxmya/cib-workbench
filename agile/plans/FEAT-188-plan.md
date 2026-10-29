<!-- SPDX-License-Identifier: MIT -->

# FEAT-188 — Plan

**Item:** [`agile/items/FEAT-188-continuous-corners-and-liquid-glass.md`](../items/FEAT-188-continuous-corners-and-liquid-glass.md)
**Branch:** `feature/workbench-2.0.0`.
**Status:** implemented.

## Approach

**Radii go through `applyMetrics()`.** Every `--r-*` is rewritten inline at zoom time, so a stylesheet `@supports` block alone would be overridden after the first paint. `metrics.ts` keeps the circular `RADII` and adds `CONTINUOUS_RADII`, chosen when `CSS.supports('corner-shape', 'superellipse(2)')`. `surfaces.css` holds the same values for the first paint, and `metrics.test.ts` reads both.

**Glass comes off through tokens, not component by component.** Twelve components and two classes (`.ornament`, `.floating`) painted with `--glass-thick` and `--blur-thick`. Setting the three grades to their solid surfaces and the three blurs to `none` makes every one of them solid with no change to Spagitty's components, which keeps merges clean. The toolbar keeps `.ornament` (its shape) and adds `.lrail` (its glass); the toast swaps `.ornament` for `.lglass`.

**The scrim** is the one change made in each modal: its dimmed backdrop takes `var(--scrim)` and the scrim blur. Thirteen files, one rule each.

**Workbench-only file.** The classes, the `@supports` blocks and the calibrated radii are in `src/lib/workbench/surfaces.css`, loaded by the layout after `app.css`. The tokens are in `app.css` with the others, where `flat.test.ts` reads them.

## Risk

TASK-022 measured the old SVG lens at 196 ms a frame on WebKitGTK. The refraction here is a `backdrop-filter: url()` on the toolbar alone, which WebKit refuses, so Linux and macOS get the 14px frost and only Chromium (WebView2 on Windows) refracts. The sweep measures it there.
