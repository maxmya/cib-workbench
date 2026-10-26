// SPDX-License-Identifier: MIT

/**
 * Workbench's brand seam is two files, `src/lib/brand.ts` for the frontend and
 * `crates/spagitty-core/src/brand.rs` for the backend, plus the identity in
 * `tauri.conf.json`. An upstream merge can move any of them; these say they
 * still agree.
 */

import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { BASED_ON, LICENCE, PRODUCT_NAME } from '../src/lib/brand';

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

	it('is based on a Spagitty release that CHANGELOG.md has', () => {
		expect(read('CHANGELOG.md')).toContain(`## [${BASED_ON.version}]`);
	});
});
