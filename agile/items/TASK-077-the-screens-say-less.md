<!-- SPDX-License-Identifier: MIT -->

# TASK-077 — The screens say less

**Status:** Done — in 1.3.0, on `release/1.3.0`, in two passes: the visible lines and empty states, then the long tooltips. Copy only; the screens' existing tests follow the new words.
**Branch:** `release/1.3.0`.
**Screens:** all.
**Raised by:** the author, 2026-10-28: "why we have this non needed chatter labels … simplify them and don't put labels unless they are needed", and "مش عايز رغي كتير في الابلكيشن كله لو الليبل ملوش لازمه نشيله" (no chatter anywhere in the app; a label that is not needed goes).

## The rule

Keep headings, field labels, errors, and empty states that say what is missing and what to do. Drop sentences that explain where something is kept, how it works, or what a control does. Where the reassurance matters, it goes in the control's `title`.

## Change

- **Calls.** The empty list is "No collections yet". "Found in this repository" shows only over found collections. The scan card asks "Scan this repository for endpoints?", and what a scan reads is in *Scan*'s tooltip.
- **Calls' editors.** "No body" and "No auth" replace two sentences each. The Content-Type, keep-rule, WS-Security and Docs lines are gone.
- **Share and Import.** Share says only "SOAP calls are left out" for OpenAPI, which is a real loss. The Postman line and Import's line on changed calls are gone.
- **Environments.** "Values Calls fills in for you after a call…" is gone; the switch's tooltip says what it does.
- **Agents › OMP.** The two lines under Model and Profile are gone.
- **Farm's wizard.** The lines on steps 2 and 4 and "Crew and rules are remembered" are gone.
- **Hooks.** The empty editor says "Choose a block".
- **Releases.** The line on hooks and confirmations under the changelog branch is gone.

The second pass:

- **Screen leads** went from Merger, Rebase, Log, Untrack and Settings › Reading. So did the line under the restore points and lines that repeated what a control says.
- **Empty states** say what is missing: "No call open", "No environments yet", "No hooks yet", "No open pull requests". Farm's empty steps say "No output yet", "No checks yet", and so on.
- **Shorter sentences:**
  - "Rewrites the last commit"
  - "Matched by name"
  - "Files stay on disk"
  - Import's three sources, each in a few words
  - the Read-only and Full run levels in one line each
- **Tooltips** over about 60 characters are cut to their point. For example, Home's Refresh is "Fetch and reload everything".

Kept on purpose: "Nothing can be committed until these are resolved", "Connect an account…", "No agent is set up…", Merger's stopped-commit line, and the hook blocks' descriptions, which say what each block checks.
