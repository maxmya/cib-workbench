<!-- SPDX-License-Identifier: MIT -->

# Staying in sync with Spagitty

Workbench is CIB's build of Spagitty. It takes in Spagitty's changes again and
again, so every Workbench difference is kept small and in known places.

## Bringing Spagitty in

Workbench's history starts on its own (`git@github.com:maxmya/cib-workbench.git`),
so Spagitty is not merged: what changed in Spagitty between the commit
Workbench is based on and a newer one is applied as a diff. `BASED_ON.commit`
in `src/lib/brand.ts` is that base.

```sh
git remote add upstream git@github.com:spa-git-ty/spagitty.git   # once
git fetch upstream                 # Spagitty's objects, kept locally, never pushed here
git switch -c sync/spagitty-<version> main
base=<BASED_ON.commit>; next=<the Spagitty commit to come up to>
git diff --binary "$base" "$next" | git apply --3way --index
node tools/brand-spdx.mjs          # new upstream files arrive with GPL headers
node tools/brand-spdx.mjs --check
bun run sweep:index                # if upstream added sweeps
bun run check && bun run test && cargo test --workspace
```

`--3way` uses the fetched Spagitty blobs, so a hunk that meets a Workbench
change becomes conflict markers to resolve rather than a refusal. Then set
**`BASED_ON`** to the new version and commit, in both `src/lib/brand.ts`
(`version`, `commit`) and `crates/spagitty-core/src/brand.rs`.
`tools/workbench-brand.test.ts` fails until both say the same and
`CHANGELOG.md` has that version. Commit the result as one change,
`chore(sync): Spagitty <version>`, with the range in its message.

Resolve conflicts **towards upstream's structure with Workbench's values**.

## Where Workbench differs, and the conflicts to expect

| What | Where | On a conflict |
| --- | --- | --- |
| Name, licence, notice, CIB's images, feature switches | `src/lib/brand.ts`, `crates/spagitty-core/src/brand.rs` | Workbench-only files: no conflict. |
| User-visible strings | ~50 frontend and ~20 Rust files read `PRODUCT_NAME` | Take upstream's sentence, put `PRODUCT_NAME` back where it said Spagitty. A new upstream string that says Spagitty is found with `grep -rn "Spagitty" src --include=*.svelte`. |
| Identity and version | `src-tauri/tauri.conf.json` (+ linux, macos), `package.json`, `Cargo.toml`, `Cargo.lock` | Keep Workbench's `productName`, `mainBinaryName`, `identifier`, title, descriptions and version (Workbench's own, from 1.0.0). |
| Palette | `src/lib/themes.ts` (`GIORNO`, `NOTTE`), the boot values in `src/app.css` | Keep Workbench's values in upstream's structure. The two must match; `src/lib/themes.test.ts` checks contrast. |
| Primary button | `src/lib/ui/Btn.svelte` (`.btn.primary` re-points `--accent` to `--primary`) | Keep the three custom-property lines. |
| Title bar, wordmark, mark | `TitleBar.svelte`, `Wordmark.svelte`, `BrandMark.svelte` | Keep Workbench's lockup. |
| Launch screen | `src/lib/chrome/Launch.svelte`, the `launching` state in `src/routes/+layout.svelte` | Keep the `.finally(() => (launching = false))` on the startup block. |
| Licence | `LICENSE`, `NOTICE` top, `deny.toml`, `version.ts`, `about.rs`, `LicenseSection.svelte`, SPDX headers | Run `tools/brand-spdx.mjs`. Third-party sections of `NOTICE` follow upstream. |
| Update check | `crates/spagitty-core/src/update.rs` (`CHECKS_FOR_UPDATES`), `src-tauri/src/settings.rs` default | Keep the early return; it makes no request. |
| Multiplexer mirror | `tools/multiplexer.ts` (`sourceOf`, `WORKBENCH_MULTIPLEXER_MIRROR`) | Keep `sourceOf` round the download. |
| Extensions' Spagitty version | `src-tauri/src/extensions.rs` passes `BASED_ON_SPAGITTY` | Extensions name Spagitty's version in `engines`, so they are checked against it, not against Workbench's. |
| App data and keychain | `com.cibeg.workbench` in `brand.rs` (`APP_ID`), `tools/extensions/cli.ts` | Keep. |
| Changelog | `CHANGELOG.workbench.md` is Workbench's, and the release lanes read it; `CHANGELOG.md` is Spagitty's and comes in as it is. | No conflict. |
| Brand copy | The Workbench section at the top of `docs/branding.md` | Keep it first. |
| Icons | `assets/brand/cib/`, `src-tauri/icons/` (bundle icons only) | Keep Workbench's. Regenerate with `python3 tools/make-workbench-icon.py` and `bun tauri icon assets/brand/cib/workbench-icon-1024.png`, never `tools/make-icons.py`. |

What stays Spagitty's on purpose: crate, module and package names, test
names, `agile/` and `docs/` (except this file and the branding section), the
`.spagitty/` folder in repositories, settings ids (`pomodoro`, `fullSpagitty`),
and agent prompts.

## Pipelines

The GitHub workflows are Spagitty's six gates, run on this repository:
`pull-request.yml` (gates 0–4 on a pull request), `gates.yml` (0–4 on a push
to `main` or `dev`; 5 build and 6 release on `main` when the version is new),
`prerelease.yml` (an alpha from `dev`, by hand) and `draft-release.yml` (a
draft from a `draft/**` branch). Workbench's differences in them:

- gate 2 checks `tools/make-workbench-icon.py` instead of Spagitty's
  `tools/make-icons.py`, and runs `node tools/brand-spdx.mjs --check`;
- release notes come from `CHANGELOG.workbench.md`;
- releases, tags and artefacts are named Workbench, the bare Windows binary is
  `cib-workbench-windows-x86_64.exe`, and the AppImage's update source is
  this repository, never Spagitty's;
- `.gitleaksignore` pins its fingerprints to this repository's commits.
