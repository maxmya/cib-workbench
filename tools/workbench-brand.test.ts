// SPDX-License-Identifier: MIT

/**
 * Workbench's brand seam is two files, `src/lib/brand.ts` for the frontend and
 * `crates/spagitty-core/src/brand.rs` for the backend, plus the identity in
 * `tauri.conf.json`. An upstream merge can move any of them; these say they
 * still agree.
 */

import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { BASED_ON, FEATURES, GITLAB, LICENCE, PRODUCT_NAME, tokenPageUrl } from '../src/lib/brand';

const read = (path: string) => readFileSync(path, 'utf8');
const rust = read('crates/spagitty-core/src/brand.rs');
const constant = (name: string) => rust.match(new RegExp(`pub const ${name}: &str = "([^"]+)";`))?.[1];
const tauri = JSON.parse(read('src-tauri/tauri.conf.json'));

describe('the Workbench brand seam', () => {
	it('says the same in TypeScript and in Rust', () => {
		expect(constant('PRODUCT_NAME')).toBe(PRODUCT_NAME);
		expect(constant('LICENCE')).toBe(LICENCE);
		expect(constant('BASED_ON_SPAGITTY')).toBe(BASED_ON.version);
	});

	it('names the app as tauri.conf.json does', () => {
		expect(tauri.productName).toBe(PRODUCT_NAME);
		expect(tauri.app.windows[0].title).toBe(PRODUCT_NAME);
		expect(constant('APP_ID')).toBe(tauri.identifier);
	});

	it('builds in the same GitLab in TypeScript and in Rust (FEAT-181)', () => {
		expect(constant('HOST')).toBe(GITLAB.host);
		expect(constant('TOKEN_PATH')).toBe(GITLAB.tokenPath);
		expect(constant('TOKEN_NAME')).toBe(GITLAB.tokenName);
		const scopes = rust.match(/pub const SCOPES: \[&str; \d+\] = \[([^\]]+)\];/)?.[1];
		expect(scopes?.split(',').map((s) => s.trim().replace(/"/g, ''))).toEqual([...GITLAB.scopes]);
		expect(rust).toMatch(new RegExp(`pub const WORKBENCH_WIDE: bool = ${FEATURES.workbenchWide};`));
	});

	it('links to the token page with the name and scopes (FEAT-181)', () => {
		expect(tokenPageUrl()).toBe(
			'https://gitlab.apps.ocp-nonprod-01.hodomain.local/-/profile/personal_access_tokens?name=Workbench&scopes=api,read_repository,write_repository'
		);
		expect(
			tokenPageUrl({ host: 'gitlab.example.com', tokenPath: '/t', tokenName: 'A b', scopes: ['api'] })
		).toBe('https://gitlab.example.com/t?name=A%20b&scopes=api');
	});

	it('is based on a Spagitty release that CHANGELOG.md has', () => {
		expect(read('CHANGELOG.md')).toContain(`## [${BASED_ON.version}]`);
	});
});
