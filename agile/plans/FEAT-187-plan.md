<!-- SPDX-License-Identifier: MIT -->

# FEAT-187 — Plan

**Item:** [`agile/items/FEAT-187-workbench-installs-its-own-updates.md`](../items/FEAT-187-workbench-installs-its-own-updates.md)
**Branch:** `ccr-ca3b004c-ol8a0t`
**Status:** implemented.

## Approach

### Tauri's updater, not a second one

The plugin already knows how to run the NSIS installer in passive mode and
restart, replace an AppImage on the same filesystem, and swap a macOS bundle;
it verifies minisign signatures before installing. Writing those again over
`forge::http` would be a larger promise about integrity than this item makes.
It brings no new HTTP client: `reqwest` is already linked by `tauri`.
`requests.test.ts` names `self_update.rs` as the one place requests leave
from outside the core.

### Fixed endpoint, fixed key

Both in `tauri.conf.json`, for the reason `update.rs` gives for its
endpoint. The plugin will not start without a `pubkey` field, so the field is
present and empty until the author commits the real one; `blocker` reads an
empty key as "cannot verify" and installs nothing.

### Signing only where the key is

`--config src-tauri/tauri.updater.conf.json` turns `createUpdaterArtifacts`
on in gate 5 only when the secret exists. A base config with it on would break every local
`bun tauri build`, which has no key.

### Ask before installing

The download happens at startup, in the background; the install happens on
*Restart to update*. The bytes stay in memory between the two, so nothing on
disk changes until then.

### One installer per platform

Windows builds NSIS only: an `.msi` install would be updated by a different
installer and end up installed twice. NSIS installs per user, which is what
lets an update run without an administrator.
