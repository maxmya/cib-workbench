<!-- SPDX-License-Identifier: MIT -->

# Self-update

Workbench installs its own updates from this repository's GitHub releases
(FEAT-187). Nobody uninstalls or reinstalls: Workbench finds a newer release,
downloads it in the background, checks its signature, and shows
**Restart to update** in the status strip and in Settings → Behaviour →
Updates. Nothing is installed until somebody presses it.

## How it works

| Step | Where |
| --- | --- |
| Gate 5 builds each platform and, when the signing key is set, signs the updater's downloads: the Windows `*-setup.exe`, the Linux `.AppImage` and the macOS `.app.tar.gz`. Each gets a `.sig`. | `.github/workflows/gates.yml` (`TAURI_SIGNING_PRIVATE_KEY`, `src-tauri/tauri.updater.conf.json`) |
| `release-assets` collects the downloads, their `.sig` files and the macOS archive. | `.github/actions/release-assets/action.yml` |
| Gate 6 writes `latest.json` from the signed downloads and attaches it to the release. | `tools/updater-manifest.mjs` |
| An installed Workbench reads `releases/latest/download/latest.json`. GitHub serves it from the newest release that is not a draft or a pre-release, so alphas never offer themselves. | `src-tauri/tauri.conf.json` (`plugins.updater.endpoints`) |
| If the version there is newer, Workbench downloads it and verifies it against the public key compiled into it. A download that does not verify is refused. | `src-tauri/src/self_update.rs` |
| **Restart to update** installs it. On Windows the installer runs with a progress bar, for the current user, with no administrator prompt, and starts Workbench again. On macOS and Linux the app is replaced and restarts. | `src/lib/updates/store.svelte.ts` |

The endpoint and the key are part of the build, not settings. An update source
that can be pointed elsewhere is a way to hand somebody a different program.

## One-time setup

Until both halves of the key are in place, releases go out as before and
installed copies only say that a newer release exists. Gate 6 warns about it.

1. **Make the repository's releases readable without signing in.** The app
   downloads without a token, so the repository (or wherever the releases are
   published) must be public.

2. **Generate the signing key**, on your own machine, never in CI:

   ```sh
   bun tauri signer generate -w ~/.tauri/workbench.key
   ```

   Give it a password. It writes `~/.tauri/workbench.key` (private) and
   `~/.tauri/workbench.key.pub` (public). Keep the private key backed up
   somewhere safe: **losing it means installed copies can never update again**,
   because every release after that is signed by a key they do not trust.

3. **Put the private key in the repository's secrets** (Settings → Secrets and
   variables → Actions):

   | Secret | Value |
   | --- | --- |
   | `TAURI_SIGNING_PRIVATE_KEY` | the contents of `~/.tauri/workbench.key` |
   | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | its password |

4. **Commit the public key** as `plugins.updater.pubkey` in
   `src-tauri/tauri.conf.json`: the one line in `~/.tauri/workbench.key.pub`.
   The bundler refuses a private key that does not match it.

5. **Release as usual.** The first release built with the key is the first
   that can update itself. People on 1.3.1 or earlier install it once over the
   top of what they have, with its `*-setup.exe`, `.dmg` or `.AppImage`; no
   uninstall is needed, and everything after that arrives on its own.

## What does not update itself

The app says why in Settings → Behaviour → Updates, and links to the release.

- **A development build**, compiled without `SPAGITTY_RELEASE`.
- **A build with no public key** in `tauri.conf.json`.
- **The bare `cib-workbench-windows-x86_64.exe`**: no installer put it there,
  so there is nothing an installer can replace.
- **A `.deb` or `.rpm` install.** The package manager owns it; install the new
  release's package.
- **An `.msi` install** from before Windows releases became NSIS-only. Install
  the new `*-setup.exe` once.

## The installer

The Windows installer is NSIS, branded with CIB's files: the header strip on
every page, the sidebar on the welcome and finish pages, and Workbench's icon.
It installs for the current user, under `%LOCALAPPDATA%`, which is what lets an
update run without an administrator. The macOS disk image opens on a white
window with the app, an arrow and Applications.

The art is generated, never edited by hand:

```sh
python3 tools/make-installer-art.py           # writes src-tauri/installer/
python3 tools/make-installer-art.py --check   # gate 2 runs this
```

## Rotating the key

Do it only if the private key leaks. Ship one release signed with the **old**
key whose `tauri.conf.json` carries the **new** public key; once people are on
it, switch the secret to the new private key.
