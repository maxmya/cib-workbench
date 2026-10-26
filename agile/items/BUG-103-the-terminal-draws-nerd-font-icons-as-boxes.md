<!-- SPDX-License-Identifier: MIT -->

# BUG-103 — The Terminal draws Nerd Font icons as boxes

**Status:** Fixed — in the working tree on `release/1.0.1`, uncommitted. Covered by font-stack tests; not looked at in the running application.
**Branch:** none yet.
**Screens:** Terminal (1T).
**Raised by:** the author, 2026-10-26: "terminal not rendering some nerd font related glyphs".

## Cause

The terminal is drawn in the code face chosen in Settings › Reading. None of the bundled faces has the Private Use Area glyphs Nerd Fonts put prompt icons at (powerline arrows, branch marks, file icons), and nothing behind them did either.

## Fix

- **Symbols Nerd Font Mono 3.4.0** (MIT, unmodified) is bundled in `assets/fonts/symbols-nerd-font/` and declared in `app.css`. It holds only the symbols, so it is a fallback, not a face.
- `terminalFont` puts it behind the code face; `rescaleOverlappingGlyphs` keeps a wide icon in its cell; once the font has loaded, the WebGL atlas is cleared and redrawn, so a glyph first drawn as a box does not stay one.
- `NOTICE` records the font and where its glyphs come from.

## Dependency

A bundled font, not a package. Why: no face the application ships has these glyphs, and a prompt that draws boxes is the usual case for anyone using starship, oh-my-posh or powerlevel10k.
