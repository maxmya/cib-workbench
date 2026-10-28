<!-- SPDX-License-Identifier: MIT -->

# FEAT-187 — Workbench installs its own updates, from its own installer

**Status:** Done — on `ccr-ca3b004c-ol8a0t`. Installing needs the one-time key setup in [`docs/self-update.md`](../../docs/self-update.md).
**Branch:** `ccr-ca3b004c-ol8a0t`.
**Screens:** Settings → Behaviour → Updates (1K), the status strip.
**Raised by:** the author, 2026-10-28: "make workbench have its own installer, not just a generic extractor installer … make the app self updating without uninstall and reinstall every time … using GitHub releases as source of updates".

## Decisions

The author answered four questions before the work began:

- **Feed:** this repository's GitHub releases, read without a token. The repository is private today; the author will make the releases readable ("act as it's public").
- **Windows install mode:** per user, with no administrator prompt, so an update installs silently.
- **Flow:** download quietly, then ask to restart. Nothing is installed mid-work.
- **Installer language:** English only.

This reverses FEAT-054's non-scope ("Spagitty does not replace its own binary") and TASK-037's, for Workbench only, and turns `brand::CHECKS_FOR_UPDATES` back on: Workbench no longer comes only through CIB's internal software channel.

## Change

- `tauri-plugin-updater` (~2.12, held below 2.13 because that needs tauri 2.12), called from Rust only. `self_update.rs` has two commands: `update_prepare` checks `latest.json`, downloads, verifies and holds the update; `update_restart` installs it and restarts.
- `blocker` decides, with no request, which builds cannot update in place: a development build, a build with no public key, and anything no Workbench installer put there (the bare `.exe`, `.deb`, `.rpm`, `.msi`). They keep FEAT-054's notice and link, and say why.
- The endpoint and the public key are in `tauri.conf.json` (`plugins.updater`), compiled in. The key is empty until the author generates one.
- Gate 5 signs the updater's downloads only when `TAURI_SIGNING_PRIVATE_KEY` is set (`--config tauri.updater.conf.json` turns on `createUpdaterArtifacts`), so a local build needs no key. Gate 6 writes `latest.json` with `tools/updater-manifest.mjs`, and warns instead of failing when nothing is signed.
- The installer: NSIS, per user, with CIB's header and sidebar and the Workbench icon; NSIS-only on Windows (`tauri.windows.conf.json`). The macOS disk image has a laid-out window. The art is `tools/make-installer-art.py`, checked by gate 2.
- The UI: `src/lib/updates/store.svelte.ts`; *Restart to update* in the status strip; Settings shows downloading, ready, up to date, or the notice.
- The notice's fallback link was Spagitty's releases page; it is `brand::RELEASES_PAGE` now.

## Non-scope

- **Code signing the installers.** Windows SmartScreen and macOS Gatekeeper still warn on first install; the updater's signature is a different key and does not quiet them.
- **Updating `.deb` and `.rpm` installs in place.** Tauri 2.11's bundler signs no package.
- **Pre-release channels.** `latest` skips alphas, as the notice always has.
- **Retro-fitting 1.3.1 and earlier.** They have no updater; people install the first release with one over the top, once.

## Acceptance criteria

- With the key set, a gate 6 release carries `latest.json` naming the setup `.exe`, the `.AppImage` and the macOS `.app.tar.gz`, each with its signature.
- An installed Workbench finds it at startup, downloads it, and installs it only on *Restart to update*, without an administrator prompt on Windows.
- A download that does not verify is refused.
- A build that cannot update in place says why and links to the release.
