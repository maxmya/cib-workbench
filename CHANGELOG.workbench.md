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

## [1.0.1] - 2026-10-26

### Changed

- **CIB's own blue, and a black Night.** The palette is built on `#0C529A`, the
  blue of CIB's logo lockups, in place of a brighter web blue. Day is neutral
  greys with near-black text; Night is black with neutral text and a soft
  CIB-blue glow, rather than navy with sky-blue text. The app icon's plate is
  the same blue.

### Fixed

- **The URL shows in Calls.** Its field was filled over the layer that draws
  the text, so a typed or loaded URL showed as a selection with no letters.
- **Fields and wells are not black holes in Night.** They are a shade under
  the pane instead of mixed toward black.
- **Settings › Calls lines up.** Cards in a row are now the same height; each
  stopped at its own content, so the rows looked scattered.

### Changed (pipeline)

- **Releases take about half as long.** The four checks run side by side, the
  installers build while they run, and the Linux packages the builds need are
  kept between runs instead of downloaded each time.

### Removed

- **The Intel Mac download.** macOS builds are for Apple silicon only.

### Downloads

- **Windows, Linux and macOS on Apple silicon.** The Mac build is signed ad hoc
  and is not notarized: on first open macOS says the developer cannot be
  verified. Allow it under **System Settings › Privacy & Security ›
  Open Anyway**, and macOS remembers the choice. If macOS says the app is
  **damaged** instead, compare the download with `SHA256SUMS-*.txt`.

## [1.0.0] - 2026-10-26

The first Workbench, based on Spagitty 4.0.1. INTERNAL USE ONLY: in
development and provided as is.

### Added

- **The Workbench identity.** The window, the installer and the title bar say
  Workbench, with CIB's logo and the W🌐rkbench wordmark, whose globe is set as
  a lowercase o. Its app icon is CIB's globe with a tools badge, so it is never
  taken for CIB's own mobile app. The app has its own data folder and keychain group
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


### Downloads

- **Windows, Linux and macOS** (Apple silicon `*-macos-arm64.dmg` and Intel
  `*-macos-x86_64.dmg`), each with its terminal multiplexer. The Windows builds
  are unsigned, so SmartScreen warns. The Mac builds are signed ad hoc and are
  not notarized: on first open macOS says the developer cannot be verified.
  Open the app once from the right-click **Open** menu, or allow it under
  **System Settings › Privacy & Security › Open Anyway**, and macOS remembers
  the choice. If macOS says the app is **damaged** instead, compare the
  download with `SHA256SUMS-*.txt` and download it again if it differs.
