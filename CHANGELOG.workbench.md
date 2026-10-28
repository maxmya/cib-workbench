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

## [1.3.1] - 2026-10-28

### Fixed

- **Calls tabs have no scroll bar.** The row of open calls still scrolls, by
  the mouse wheel or a trackpad, but Windows' bar with arrows no longer shows
  under it (BUG-114).

### Downloads

- **Windows, Linux and macOS on Apple silicon.** The Mac build is signed ad hoc
  and is not notarized: on first open macOS says the developer cannot be
  verified. Allow it under **System Settings › Privacy & Security ›
  Open Anyway**, and macOS remembers the choice. If macOS says the app is
  **damaged** instead, compare the download with `SHA256SUMS-*.txt`.

## [1.3.0] - 2026-10-28

### Added

- **Discard a review.** Finish review has *Discard*, which throws away the
  pending comments, the summary and the viewed files after asking. Nothing
  has reached the host by then (FEAT-186).

### Changed

- **The screens say less.** Sentences that explained how things work are
  gone across the application, empty screens say only what is missing, and
  long tooltips are shorter. Errors and warnings stay as they were
  (TASK-077).

### Fixed

- **Calls' open calls stay on one row.** When more calls are open than the
  header can hold, the strip scrolls sideways instead of wrapping onto a
  second row, and the call shown scrolls into view (BUG-114).
- **Recipes' panels resize, and the right one goes away.** Both sides drag
  wider or narrower, the run panel can be put away and brought back, and it
  no longer scrolls sideways (BUG-115).
- **The launch screen's globe is clean, and "CIB" is centred.** The orange
  glow that filled the globe's gaps is gone, and "CIB" sits over the
  wordmark (BUG-116).

### Downloads

- **Windows, Linux and macOS on Apple silicon.** The Mac build is signed ad hoc
  and is not notarized: on first open macOS says the developer cannot be
  verified. Allow it under **System Settings › Privacy & Security ›
  Open Anyway**, and macOS remembers the choice. If macOS says the app is
  **damaged** instead, compare the download with `SHA256SUMS-*.txt`.

## [1.2.0] - 2026-10-28

### Added

- **What a call expects and answers, from its code.** A scanned Spring,
  JAX-RS or NestJS call carries its body's field rules (required, format,
  lengths, bounds, allowed values) and what it answers with each status,
  each with an example and where it was read from. A *Docs* tab shows them.
  On Import, an agent you allow can read the chosen controllers for what the
  scan missed; it adds and never overwrites (FEAT-179).
- **Export as OpenAPI.** Share writes a collection as OpenAPI 3.1, JSON or
  YAML, with those rules and answers; it is also in each collection's `⋯`
  (FEAT-179).
- **Endpoints say which service they are in.** Import finds each Maven or
  Gradle module, `package.json` and Nest monorepo app, and shows controllers
  under their service (FEAT-177).
- **Hooks in two tabs.** *Current hooks* lists every step the repository's
  hooks run, numbered in the order git runs them, with a category filter.
  *Create hook* has 18 explained templates to add (FEAT-176).
- **Close others, close all.** A right-click on a tab in Files (FEAT-174).
- **Calls' side panels widen,** and the variables can be put away
  (FEAT-175).

### Changed

- **Import is a sheet over Calls,** in three steps with *Back* and *Cancel*,
  opened from the Collections `+` (FEAT-178).
- **Home's *Fetch all* is *Refresh*:** it fetches and reloads every card.
  Latest commits say who made each (FEAT-173).
- **Recipes' right panel says less:** the level, *Run*, and *Agent help*
  only when it is on (TASK-076).

### Fixed

- **A GitLab merge request's files load** when the server's stored diffs
  fail with 500 (BUG-108).
- **Review's bar stays on one row,** tighter as room shrinks (BUG-109).
- **A pipeline's lines run between its cards,** never through one; jobs that
  need one job share a trunk (BUG-110).
- **The Terminal shows output sooner:** a key's echo at once, and a flood in
  few messages (BUG-111).
- **Checkboxes, switches and radios are drawn in the theme,** not as light
  boxes in Night (BUG-112).
- **Calls' fields and panes are the card's colour,** and the `{{` hint is
  gone (BUG-113).

### Downloads

- **Windows, Linux and macOS on Apple silicon.** The Mac build is signed ad hoc
  and is not notarized: on first open macOS says the developer cannot be
  verified. Allow it under **System Settings › Privacy & Security ›
  Open Anyway**, and macOS remembers the choice. If macOS says the app is
  **damaged** instead, compare the download with `SHA256SUMS-*.txt`.

## [1.1.0] - 2026-10-26

### Added

- **Calls without a repository.** Collections can be kept in the repository,
  beside the code, or in Workbench for every repository. The switch is in
  Calls' head, on Import and in Settings › Calls; with no repository open,
  Calls works Workbench-wide. Open calls, environments and history are kept
  for each place (FEAT-170).
- **Hooks you can read.** With Husky, lefthook or pre-commit in charge, the
  Hooks tab shows every script they run, coloured, with *Explain* (an agent
  you allow says what it does) and *Edit*. Scripts are coloured in Settings ›
  Hooks too, and the screen says in a line what Runs and Hooks are for
  (FEAT-171).
- **Open on All repositories.** An option in Settings › Behaviour, off by
  default, starts Workbench with nothing open (FEAT-172).

### Changed

- **The rail in groups.** Repository, Collaboration, Workspace, History and
  Tools, each below a line you can see with the rail closed (FEAT-169).
- **The launch screen stays two seconds,** so it can be read. A click or a key
  skips it once the repository is open (FEAT-168).
- **git's failures in plain words.** A refused push, pull, fetch, stash or
  switch says what went wrong and what to do; git's own lines are behind
  *Show git's output*. The toolbar says *Push failed · why?* instead of git's
  whole text (BUG-107).

### Fixed

- **Closing a tab shows the next repository.** The closed one stayed loaded
  behind it until you changed screens (BUG-102).
- **Nerd Font icons draw in the Terminal.** Prompt icons were boxes; a symbols
  font is bundled behind the code face (BUG-103).
- **Review's controls line up with the toolbar,** and keep their width
  whatever the file is called (BUG-104).
- **Importing into Calls says it is reading,** then how many calls came and
  what to do next (BUG-105).
- **A Calls collection can be hidden or deleted** from its `⋯` menu
  (BUG-106).

### Downloads

- **Windows, Linux and macOS on Apple silicon.** The Mac build is signed ad hoc
  and is not notarized: on first open macOS says the developer cannot be
  verified. Allow it under **System Settings › Privacy & Security ›
  Open Anyway**, and macOS remembers the choice. If macOS says the app is
  **damaged** instead, compare the download with `SHA256SUMS-*.txt`.

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
