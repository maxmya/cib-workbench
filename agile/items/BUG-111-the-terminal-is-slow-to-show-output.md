<!-- SPDX-License-Identifier: MIT -->

# BUG-111 — The Terminal is slow to show output

**Status:** Fixed — in the working tree on `release/1.1.0`, uncommitted. Covered by `terminal.rs`'s three outbox tests, `terminal/batch.test.ts` and `terminal/store.test.ts`. How fast it feels on the reporter's machine has not been checked.
**Branch:** none yet.
**Screens:** Terminal (1T).
**Raised by:** the author, 2026-10-28: "terminal render is slow and it takes time to output anything ! not snapy and fast enough".

## Cause

Each piece the PTY read was a message to the webview, so a flood was hundreds of messages. The webview then held every piece for the next frame, a keystroke's echo included.

## Fix

- The host gathers output into few messages, in order and with nothing lost.
- The first output after a pause, such as a key's echo, is written at once. What follows in the same frame waits for the frame. A character cut in two waits for its other half.
- If it still feels slow on Windows, the bundled multiplexer is an extra layer, and can be turned off in Settings.
