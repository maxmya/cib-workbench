<!-- SPDX-License-Identifier: MIT -->

# BUG-112 — Checkboxes are the platform's, not the theme's

**Status:** Fixed — in the working tree on `release/1.1.0`, uncommitted. A stylesheet change; not seen in the running application.
**Branch:** none yet.
**Screens:** Untrack (2B), and every screen with a checkbox, switch or radio.
**Raised by:** the author, 2026-10-28: "its perfect but checkboxes are not matching our design schema".

## Cause

The accent colour was all a checkbox took from the theme. Nothing told the webview that Night is dark, so on Windows an unticked box was a light square.

## Fix

- Checkboxes, switches and radios are drawn by `app.css` in the theme's colours, in Day and Night.
