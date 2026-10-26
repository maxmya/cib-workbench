<!-- SPDX-License-Identifier: MIT -->

# FEAT-171 — Hooks a manager runs, read, explained and edited

**Status:** Done — in the working tree on `release/1.0.1`, uncommitted. Covered by `hooks/builder.test.ts` › *hooks a manager runs*, `hooks/script.test.ts` and `hooks.rs`' tests; *Explain* not tried against a real agent.
**Branch:** none yet.
**Screens:** Hooks (1X), Settings › Hooks.
**Raised by:** the author, 2026-10-26: "hooks shows 'Husky manages this repository's hooks. Runs are available; the builder is read-only.' we should be able to read it here and maybe a button to make agent explain what it do or even edit it !"; "in this hooks thingy we shall color its code here or on settings"; "maybe hooks screen i didn't undersand it well so if you can explain it better".

## Change

- **The screen says what it is.** A line under the tabs: hooks are scripts git runs when you commit or push; *Runs* lists every time they ran and what each step printed; *Hooks* shows what runs, or builds hooks from blocks.
- **What a manager runs is on screen.** With Husky, lefthook or pre-commit in charge, *Hooks* shows every script it runs and its configuration, where it said only that the builder was read-only. The banner says who runs them, from files committed with the code, and offers *Import as blocks*. The catalogue still offers the import (BUG-099).
- **Explain.** An agent this repository allows reads the file and says, in short sentences, when it runs, what each command does, what stops a commit or push, and how to skip it once. It is asked through `agents::ask`, read-only, as Recipes asks. With several agents, a choice says which.
- **Edit.** A file is edited in place and saved; its permissions stay, so a script that ran still runs. Commit it to share the change.
- **Colour.** Scripts are coloured, numbered lines, here and in Settings › Hooks: the language from the file's name, else from its first line (`#!/usr/bin/env node`), else shell.

## Safety

`hooks_explain` and `hooks_write` accept only a path that is one of the repository's listed hook files or its manager's configuration, and a write of at most 256 KiB to a file that is there.
