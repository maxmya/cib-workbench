// SPDX-License-Identifier: MIT

/**
 * Write the `latest.json` an installed Workbench reads to update itself
 * (FEAT-187).
 *
 * Gate 6 runs this over the files gate 5 built, before anything is tagged, and
 * attaches the answer to the release. `tauri.conf.json` points the updater at
 * `releases/latest/download/latest.json`, which GitHub resolves to the newest
 * release that is neither a draft nor a pre-release — the same rule the update
 * notice follows — so an alpha never offers itself to anybody.
 *
 * Each platform's entry is the download and its signature. The signature is
 * the `.sig` the Tauri bundler writes beside an updater artefact when the
 * build has the signing key; a download without one is left out, because the
 * app refuses an update it cannot verify and an entry it would refuse is a
 * failure on somebody else's machine instead of here.
 *
 * Usage: bun tools/updater-manifest.mjs <artifacts-dir> <tag> <owner/repo> [notes-file]
 *   Prints the manifest. Exits 3, printing nothing, when no download is signed.
 */

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { basename, join } from 'node:path';
import { pathToFileURL } from 'node:url';

/**
 * Which download serves which platform. The updater looks for
 * `{os}-{arch}-{installer}` first and `{os}-{arch}` second, so each download
 * is listed under both; `.deb` and `.rpm` installs are not updated in place
 * and have no entry.
 */
export const PLATFORMS = [
	{ pattern: /_x64-setup\.exe$/, keys: ['windows-x86_64-nsis', 'windows-x86_64'] },
	{ pattern: /_amd64\.AppImage$/, keys: ['linux-x86_64-appimage', 'linux-x86_64'] },
	// `release-assets` names it for its runner: `Workbench-macos-arm64.app.tar.gz`.
	{ pattern: /-macos-arm64\.app\.tar\.gz$/, keys: ['darwin-aarch64-app', 'darwin-aarch64'] }
];

/** Every file under `dir`, at any depth: gate 6 downloads one folder per runner. */
export function walk(dir) {
	return readdirSync(dir).flatMap((name) => {
		const path = join(dir, name);
		return statSync(path).isDirectory() ? walk(path) : [path];
	});
}

/**
 * The manifest for `files`, or null when none of them is signed.
 *
 * `read` returns a file's text; it is a parameter so the tests need no disk.
 * Throws when two downloads claim one platform, because the updater would
 * take whichever came first and nobody would know which.
 */
export function manifest({ files, tag, repo, notes = '', date = new Date(), read }) {
	const names = new Set(files.map((file) => basename(file)));
	const platforms = {};
	for (const file of files) {
		const name = basename(file);
		const platform = PLATFORMS.find(({ pattern }) => pattern.test(name));
		if (!platform || !names.has(`${name}.sig`)) continue;
		const signature = read(`${file}.sig`).trim();
		const url = `https://github.com/${repo}/releases/download/${tag}/${encodeURIComponent(name)}`;
		for (const key of platform.keys) {
			if (platforms[key]) throw new Error(`${key} is claimed by ${platforms[key].url} and ${url}`);
			platforms[key] = { signature, url };
		}
	}
	if (Object.keys(platforms).length === 0) return null;
	return {
		version: tag.replace(/^v/, ''),
		notes: notes.trim(),
		pub_date: date.toISOString().replace(/\.\d{3}Z$/, 'Z'),
		platforms
	};
}

function main(argv) {
	const [dir, tag, repo, notesFile] = argv;
	if (!dir || !tag || !repo) {
		process.stderr.write('usage: updater-manifest.mjs <artifacts-dir> <tag> <owner/repo> [notes-file]\n');
		return 2;
	}
	const answer = manifest({
		files: walk(dir),
		tag,
		repo,
		notes: notesFile ? readFileSync(notesFile, 'utf8') : '',
		read: (path) => readFileSync(path, 'utf8')
	});
	if (!answer) {
		process.stderr.write('no download is signed, so there is no update to offer\n');
		return 3;
	}
	process.stdout.write(`${JSON.stringify(answer, null, 2)}\n`);
	return 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) {
	process.exit(main(process.argv.slice(2)));
}
