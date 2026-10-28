// SPDX-License-Identifier: MIT

/**
 * Workbench's installer and its self-update, checked where they are decided
 * (FEAT-187).
 *
 * Like `updates.test.ts`, most of this is joins between files no compiler
 * reads together: the update endpoint against the repository the in-app
 * check reads, the download names gate 5 produces against the patterns
 * `updater-manifest.mjs` looks for, and the installer art against the
 * configuration that points at it.
 */

import { describe, expect, it } from 'vitest';
import { existsSync, readFileSync } from 'node:fs';
import { manifest, PLATFORMS } from './updater-manifest.mjs';

const read = (path: string) => readFileSync(path, 'utf8');
const config = JSON.parse(read('src-tauri/tauri.conf.json'));

const signatures: Record<string, string> = {
	'a/Workbench_1.4.0_x64-setup.exe.sig': 'sig-windows\n',
	'b/Workbench_1.4.0_amd64.AppImage.sig': 'sig-linux',
	'c/Workbench-macos-arm64.app.tar.gz.sig': 'sig-mac'
};

const built = [
	'a/Workbench_1.4.0_x64-setup.exe',
	'a/Workbench_1.4.0_x64-setup.exe.sig',
	'a/cib-workbench-windows-x86_64.exe',
	'b/Workbench_1.4.0_amd64.AppImage',
	'b/Workbench_1.4.0_amd64.AppImage.sig',
	'b/Workbench_1.4.0_amd64.AppImage.zsync',
	'b/Workbench_1.4.0_amd64.deb',
	'c/Workbench_1.4.0_aarch64-macos-arm64.dmg',
	'c/Workbench-macos-arm64.app.tar.gz',
	'c/Workbench-macos-arm64.app.tar.gz.sig'
];

const compose = (files: string[], notes = '') =>
	manifest({
		files,
		tag: 'v1.4.0',
		repo: 'maxmya/cib-workbench',
		notes,
		date: new Date('2026-10-28T12:00:00.123Z'),
		read: (path: string) => signatures[path]
	});

describe('the update manifest', () => {
	it('lists each signed download under both of the names the updater looks for', () => {
		const found = compose(built, '### Added\n\n- Updates.\n');

		expect(found).toEqual({
			version: '1.4.0',
			notes: '### Added\n\n- Updates.',
			pub_date: '2026-10-28T12:00:00Z',
			platforms: expect.any(Object)
		});
		expect(Object.keys(found!.platforms).sort()).toEqual([
			'darwin-aarch64',
			'darwin-aarch64-app',
			'linux-x86_64',
			'linux-x86_64-appimage',
			'windows-x86_64',
			'windows-x86_64-nsis'
		]);
		expect(found!.platforms['windows-x86_64']).toEqual({
			signature: 'sig-windows',
			url: 'https://github.com/maxmya/cib-workbench/releases/download/v1.4.0/Workbench_1.4.0_x64-setup.exe'
		});
		expect(found!.platforms['darwin-aarch64'].url).toMatch(/\/Workbench-macos-arm64\.app\.tar\.gz$/);
	});

	it('leaves out a download with no signature, which the app would refuse', () => {
		const found = compose(built.filter((file) => !file.endsWith('AppImage.sig')));

		expect(found!.platforms['linux-x86_64']).toBeUndefined();
		expect(found!.platforms['windows-x86_64']).toBeDefined();
	});

	it('is nothing at all when no download is signed', () => {
		expect(compose(built.filter((file) => !file.endsWith('.sig')))).toBeNull();
	});

	it('refuses two downloads for one platform rather than picking one', () => {
		signatures['d/Workbench_1.4.1_x64-setup.exe.sig'] = 'other';
		expect(() =>
			compose([...built, 'd/Workbench_1.4.1_x64-setup.exe', 'd/Workbench_1.4.1_x64-setup.exe.sig'])
		).toThrow(/windows-x86_64/);
	});

	it('matches the names the bundler writes for this product', () => {
		// Composed the way `updates.test.ts` composes the AppImage's name, so a
		// renamed product fails here and not on somebody's machine.
		const { productName, version } = config;
		const names = [`${productName}_${version}_x64-setup.exe`, `${productName}_${version}_amd64.AppImage`];
		for (const name of names) {
			expect(PLATFORMS.some(({ pattern }) => pattern.test(name)), name).toBe(true);
		}
	});
});

describe('the updater configuration', () => {
	const updater = config.plugins.updater;

	it('reads this repository and nothing else', () => {
		// The same repository as the in-app notice, in the same letters.
		expect(updater.endpoints).toEqual([
			'https://github.com/maxmya/cib-workbench/releases/latest/download/latest.json'
		]);
		expect(read('crates/spagitty-core/src/update.rs')).toContain('repos/maxmya/cib-workbench/');
	});

	it('carries a public key slot, so the plugin starts', () => {
		// The plugin refuses to start without `pubkey`; empty means "cannot
		// verify", which the app answers by not installing anything.
		expect(typeof updater.pubkey).toBe('string');
	});

	it('installs on Windows with a progress bar and no questions', () => {
		expect(updater.windows.installMode).toBe('passive');
	});

	it('is signed by gate 5 only when the key is there', () => {
		const gates = read('.github/workflows/gates.yml');
		expect(gates).toContain('TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}');
		expect(gates).toContain('/src-tauri/tauri.updater.conf.json');
		expect(JSON.parse(read('src-tauri/tauri.updater.conf.json')).bundle.createUpdaterArtifacts).toBe(true);
		// Never in the base configuration: a local build has no key to sign with.
		expect(config.bundle.createUpdaterArtifacts).toBeUndefined();
		expect(gates).toContain('bun tools/updater-manifest.mjs artifacts');
		expect(gates).toContain('${{ steps.manifest.outputs.file }}');
	});

	it('collects the signatures and the macOS archive with the downloads', () => {
		const collect = read('.github/actions/release-assets/action.yml');
		for (const name of ["'*-setup.exe.sig'", "'*.AppImage.sig'", "'*.app.tar.gz'", "'*.app.tar.gz.sig'"]) {
			expect(collect).toContain(`-name ${name}`);
		}
	});
});

describe('the installer', () => {
	const nsis = config.bundle.windows.nsis;

	it('installs for the current user, so an update needs no administrator', () => {
		expect(nsis.installMode).toBe('currentUser');
	});

	it('is the only Windows installer built', () => {
		const windows = JSON.parse(read('src-tauri/tauri.windows.conf.json'));
		expect(windows.bundle.targets).toEqual(['nsis']);
	});

	it.each([
		['header', () => nsis.headerImage],
		['sidebar', () => nsis.sidebarImage],
		['uninstaller header', () => nsis.uninstallerHeaderImage],
		['disk image background', () => config.bundle.macOS.dmg.background]
	])('points at %s art that tools/make-installer-art.py wrote', (_, path) => {
		expect(existsSync(`src-tauri/${path()}`)).toBe(true);
	});

	it('places the disk image icons where the art draws its arrow', () => {
		const art = read('tools/make-installer-art.py');
		const { appPosition, applicationFolderPosition, windowSize } = config.bundle.macOS.dmg;
		expect(art).toContain(`DMG_SIZE = (${windowSize.width}, ${windowSize.height})`);
		expect(art).toContain(`DMG_APP = (${appPosition.x}, ${appPosition.y})`);
		expect(art).toContain(`DMG_APPLICATIONS = (${applicationFolderPosition.x}, ${applicationFolderPosition.y})`);
	});
});
