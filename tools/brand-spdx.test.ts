// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { remaining } from './brand-spdx.mjs';

/**
 * Workbench is MIT. An upstream merge brings new Spagitty files with GPL
 * headers; `node tools/brand-spdx.mjs` rewrites them, and this fails until it
 * has been run.
 */
describe('the licence of every first-party file', () => {
	it('is MIT: no tracked file says GPL-3.0-or-later', () => {
		expect(remaining()).toEqual([]);
	});

	it('is MIT in every manifest that states one', () => {
		expect(JSON.parse(readFileSync('package.json', 'utf8')).license).toBe('MIT');
		expect(JSON.parse(readFileSync('packages/extension-sdk/package.json', 'utf8')).license).toBe('MIT');
		expect(readFileSync('Cargo.toml', 'utf8')).toMatch(/^license = "MIT"$/m);
		expect(readFileSync('LICENSE', 'utf8')).toMatch(/^MIT License/);
	});
});
