<!-- SPDX-License-Identifier: MIT -->

# FEAT-187 — Automated tests

**Item:** [`agile/items/FEAT-187-workbench-installs-its-own-updates.md`](../items/FEAT-187-workbench-installs-its-own-updates.md)

| Where | Asserts |
| --- | --- |
| `src-tauri/src/self_update.rs` | Which installs can update in place (NSIS, AppImage, macOS app) and the reason each other kind is given: development build, no key, no installer, package manager, `.msi`. The answer's tagged shape. Restarting with nothing downloaded is an error. |
| `tools/installer.test.ts` | `latest.json` lists each signed download under both names the updater looks for, leaves out unsigned ones, is nothing when nothing is signed, refuses two downloads for one platform, and matches the names the bundler writes. The endpoint is this repository. Gate 5 signs only with the secret; gate 6 attaches the manifest; `release-assets` collects signatures and the macOS archive. NSIS is per user and the only Windows installer. The art exists where the config points, and the DMG icon positions match the art. |
| `src/lib/updates/store.test.ts` | Downloads then waits; installs only when asked; never downloads twice; up to date; falls back to the notice when unsupported; keeps a failure and retries; asks nothing outside the app. |
| `src/lib/settings/sections.test.ts` | Settings offers *Restart now* once downloaded and says when it is up to date; the notice cases run through the unsupported path. |
| `src/lib/requests/requests.test.ts` | `self_update.rs` is the only file in the Tauri layer that uses the updater. |
| `crates/spagitty-core/src/update.rs` | Workbench checks, and its fallback link is its own releases page. |
| gate 2 | `python3 tools/make-installer-art.py --check`. |

**What no test here can see:** the installer running, a signature verifying on
a real download, or the restart. Those are the sweep.
