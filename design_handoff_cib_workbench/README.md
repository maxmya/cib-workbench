# Handoff: Workbench, CIB's build of Spagitty

## Corrections from the author (2026-10-26)

These override the mockups and `WORKBENCH_PLAN.md` wherever they disagree.

- **CIB blue is `#0C529A`**, the plate of CIB's own logo lockups (the author's
  Business Banking lockup), not `#005BAB` and not the web blue `#1858A8`. Every
  `#005BAB` and `rgba(0,91,171,…)` in the mockups means `#0C529A` /
  `rgba(12,82,154,…)`.
- **Night is black, not navy.** Surfaces are neutral near-black (`#111214`, the
  ground `#0B0B0C`) with neutral text, and the only colour in the ground is a soft
  CIB-blue ambient glow. Night's text accent is `#4D8FD6`, CIB blue lifted until
  it reads on black, not the sky blue `#6CB4F0`. Day is neutral greys too.
- **The globe in W🌐rkbench is a lowercase o.** The mockups size it to the capital
  W and stand it on the baseline, so it reads as a capital. It is as tall as the
  lowercase letters, with the small overshoot round letters have above and below.
- **The app icon carries a tools badge.** The mockups' icon, the globe alone on
  CIB blue, is CIB's own mobile-app icon. Workbench's adds a white badge with a
  wrench and a screwdriver at the bottom right (`tools/make-workbench-icon.py`).
- The images the mockups call `/_blob/…` are in `assets/brand/cib/`.

The design for Workbench. `WORKBENCH_PLAN.md` (in this folder) says what to build. This
folder shows what it should look like.

| File | What it is |
| --- | --- |
| `source/*.dc.html` | Design Component sources. **Treat the markup as a layout spec only.** Throw away the DC runtime, and build with the repo's own components and tokens. |
| `source/canvas.json` | How the artboards sit on the canvas. |

`Main.dc.html` is Home in Night; the canvas's entry file has to be called Main.
`Rail`, `Dock`, `Steps` and `SoFar` are pieces the screens import; Rail and
Dock take `theme="light"` for Day.

Images are referenced as `/_blob/<id>`. Those are the files in `../brand/`:

| Id starts | File |
| --- | --- |
| `25a0fe` | `cib-logo-color.png` |
| `ca12b7` | `cib-logo-reversed.png` |
| `315edc` | `cib-logo-reversed-tagline.png` |
| `ed33cc` | `cib-globe-round.png` |

**Fidelity: high** for structure, copy, and the meaning of every colour.
Every repository, pull request, ticket, person, count and time is made up.
