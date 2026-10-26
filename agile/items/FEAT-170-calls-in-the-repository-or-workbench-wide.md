<!-- SPDX-License-Identifier: MIT -->

# FEAT-170 — Calls in the repository, or Workbench-wide

**Status:** Done — in the working tree on `release/1.0.1`, uncommitted. Covered by `CallsScreen.test.ts` › *where collections are kept*, the Calls store and import tests, and `calls.rs`' scope tests; not looked at in the running application.
**Branch:** none yet.
**Screens:** Calls (2C), Calls › Import, Settings › Calls.
**Raised by:** the author, 2026-10-26: "make calls optional to be global or project specific"; "make calls import page … cover the scoap (global or project based)"; "info about opened calles can be saved per repo or globally … even if no project is opend".

## Change

- **Two places for collections.** *The repository*, as before: `.spagitty/calls/`, beside the code, to be committed. Or *Workbench*: application data under `calls/global/`, laid out as a working tree is, the same whichever repository is open, and never committed.
- **One switch.** Calls' head, Import's head (*into …*) and Settings › Calls › *Keep collections* show and set the same choice, `callsScope` in the application's settings. It is not in `calls`, which a repository may override: whether to use the repository is not the repository's choice.
- **No repository needed.** With none open, Calls and Import are Workbench-wide. The open calls, the active environment, what is collapsed and hidden, and history are kept per place, so each comes back as it was left.
- **What only a repository has** stays with it: scanning its code, the collection files found in it, and *Share › Commit*, which says why it is refused for a Workbench-wide collection.

## Design

Every Calls command takes an optional `scope` and resolves it to a root directory: the open repository, or the Workbench folder. The folder, the state, the history and the keychain key are then found from that root as they always were, so the two places cannot drift apart in behaviour. `api.ts` holds the scope the store last set, so no caller can write one place's collection into the other's.
