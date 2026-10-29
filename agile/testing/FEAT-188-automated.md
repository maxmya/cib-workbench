<!-- SPDX-License-Identifier: MIT -->

# FEAT-188 — Automated tests

**Item:** [`agile/items/FEAT-188-continuous-corners-and-liquid-glass.md`](../items/FEAT-188-continuous-corners-and-liquid-glass.md)

| Where | Asserts |
| --- | --- |
| `src/lib/metrics.test.ts` | Circular radii agree with `app.css`; continuous radii agree with `surfaces.css`, scale with the zoom, and leave pills and ornaments round. |
| `src/lib/ui/glass.test.ts` | The thick, regular and thin materials are solid with no blur in both themes; the `--lg-*` tokens and the scrim exist in both themes; the toast frosts and the toolbar refracts, with the frost as the fallback and both prefixes, naming no colour; glass classes are on the toolbar and the toast only; every modal blurs behind with the scrim; one fixed refraction filter, mounted once; `corner-shape` only inside `@supports`, never on pills, chips, ornaments or switches; every squircle radius larger than its circular one. |
| `src/lib/ui/flat.test.ts` | Unchanged: no component names a colour or a pixel size of its own, and every token read is defined. |

**What no test here can see:** the corners and the glass on a screen. That is the sweep.
