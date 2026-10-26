<!-- SPDX-License-Identifier: MIT -->

# BUG-105 — Importing a collection shows nothing while it reads

**Status:** Fixed — in the working tree on `release/1.0.1`, uncommitted. Covered by `ImportScreen.test.ts` › *reading a file*.
**Branch:** none yet.
**Screens:** Calls › Import (2C).
**Raised by:** the author, 2026-10-26: "calls when i import a collection no loading animation or feedback shown".

## Cause

Choosing a file, a Bruno folder, a WSDL address or a pasted cURL awaited the backend with nothing on screen. A large collection took seconds, and when it came back its calls only appeared in the list, not yet in any collection, with nothing saying so.

## Fix

- `importer.read` holds what is being read: the head says *Reading big.json…* with the strands, the endpoint list shows the loader instead of saying it is empty, and the source buttons wait.
- After it, a line says what came: *2 calls read from big.json. Choose where they go, then bring them in.*
- *Bring in* says *Bringing them in…* while it writes.
