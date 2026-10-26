<!-- SPDX-License-Identifier: MIT -->

# FEAT-172 — Open on All repositories

**Status:** Done — in the working tree on `release/1.0.1`, uncommitted. Covered by `session.test.ts` › *starting on All repositories*, the Behaviour section and `settings.rs`.
**Branch:** none yet.
**Screens:** Settings › Behaviour, the launch.
**Raised by:** the author, 2026-10-26: "option on options disabled by deafult to open spagitty ( or workbench ) on all repos screen with no repo select or open last opened repo".

## Change

- Settings › Behaviour › *Open on All repositories, with no repository open*. Off by default: a launch reopens the tab the last session was on, as before.
- On, a launch opens nothing and lands on All repositories. The tabs stay in the strip with none shown, and which one the session was on is not forgotten, so turning it off again resumes it.
- A repository named on the command line is opened either way.
