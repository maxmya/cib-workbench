// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';

/**
 * A VarField's letters are drawn by a mirror behind a transparent input, so
 * anything that fills the input hides what is typed. The URL bar did exactly
 * that: `RequestEditor` gave its input `--sunken`, and the field showed a
 * selection with no letters in it.
 */
const read = (path: string) => readFileSync(path, 'utf8');
const style = (path: string) => read(path).split('<style>')[1] ?? '';

describe('a field with variables in it', () => {
	/** BUG-113: flat, the card's own colour; a well a shade under it read as a dark slab. */
	it('paints nothing behind the mirror, so the field is the card’s colour', () => {
		const field = /\.field \{([^}]*)\}/.exec(style('src/lib/calls/VarField.svelte'))?.[1] ?? '';
		expect(field).not.toMatch(/background/);
	});

	it('keeps its input transparent, focused or not', () => {
		const css = style('src/lib/calls/VarField.svelte');
		expect(css).toMatch(/input:focus,\s*textarea:focus \{\s*background-color: transparent;/);
		expect(css).toMatch(/color: transparent;/);
	});

	it('is never filled from outside', () => {
		for (const path of ['src/lib/calls/RequestEditor.svelte', 'src/lib/calls/SoapEditor.svelte']) {
			expect(style(path), path).not.toMatch(/:global\(input\)\s*\{[^}]*background/);
		}
	});
});
