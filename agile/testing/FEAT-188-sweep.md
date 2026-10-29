<!-- SPDX-License-Identifier: MIT -->

# FEAT-188 — Manual sweep

**Item:** [`agile/items/FEAT-188-continuous-corners-and-liquid-glass.md`](../items/FEAT-188-continuous-corners-and-liquid-glass.md)

| Ticket | Preconditions | Steps | Expected result | Priority | Pass/Fail |
| --- | --- | --- | --- | --- | --- |
| SWEEP-001 | Windows, Night | Look at Home's cards, a button, a field and the window corner at 500% screen zoom | Continuous corners, about as soft as the circular design; chips and the toolbar stay capsules. | High | |
| SWEEP-002 | Windows | Scroll the graph under the toolbar | What is behind the toolbar bends gently and is lightly frosted; text on it stays readable. | High | |
| SWEEP-003 | SWEEP-002 | Resize the window and scroll quickly | No stutter from the toolbar (compare with its glass switched off in devtools). | High | |
| SWEEP-004 | Any | Trigger a notice (a fetch) | The toast is frosted glass with a bright rim, in Day and Night. | Medium | |
| SWEEP-005 | Any | Open a menu, the palette and a confirmation | Menus and panels are solid; behind a modal the screen is dimmed and blurred. | High | |
| SWEEP-006 | macOS or Linux | SWEEP-002 and SWEEP-004 | Frosted, with no refraction; nothing broken. | Medium | |
| SWEEP-007 | Any | Ctrl+= to 150% | Corners grow with the zoom and stay squircles. | Low | |
