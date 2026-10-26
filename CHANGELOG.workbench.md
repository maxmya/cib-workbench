<!-- SPDX-License-Identifier: MIT -->

# Workbench changelog

All notable changes to Workbench, CIB's build of Spagitty, newest first.
Spagitty's own changes are in `CHANGELOG.md`, which arrives with every
upstream merge; this file records only what Workbench does differently.
Entries are written into `Unreleased` in the same change as the work, and
`node tools/release-notes.mjs <version> CHANGELOG.workbench.md` prints a
version's notes.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versions follow [Semantic Versioning 2.0.0](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Workbench 1.0.0, based on Spagitty 4.0.1.

### Added

- **The Workbench identity.** The window, the installer and the title bar say
  Workbench, with CIB's logo and the W🌐rkbench wordmark, whose globe is set as
  a lowercase o. The app has its own data folder and keychain group
  (`com.cibeg.workbench`), so it can sit beside Spagitty.
- **The CIB palette, Day and Night,** built on CIB's primary blue `#1858A8`.
  Primary buttons are CIB blue with white text in both modes.
- **A launch screen** while the first repository opens, with the internal-use
  notice, which Settings › About shows too.
- **`WORKBENCH_MULTIPLEXER_MIRROR`** points the build's tmux and psmux
  downloads at an internal mirror; the pinned checksums still apply.
- **Spagitty's six gates on this repository**: checks on every pull request and
  push, and builds and releases from `main` named Workbench, with notes from this
  file. The AppImage's update source is this repository.
- `tools/brand-spdx.mjs` and `tools/make-workbench-icon.py`, and
  `docs/workbench-sync.md` on bringing Spagitty's changes in.

### Changed

- **Licensed under MIT**, by Spagitty's sole author. Every first-party file,
  the manifests and Settings › About say so.
- Messages, labels and notices say Workbench; the forge, model and Calls
  clients identify as Workbench.

### Removed

- The update check. Workbench comes through CIB's internal software channel
  and asks nobody for a newer release.
- God mode from Settings.
