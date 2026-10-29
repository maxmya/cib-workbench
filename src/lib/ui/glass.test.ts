// SPDX-License-Identifier: MIT

/**
 * The thick glass material, guarded at the source.
 *
 * **Superseded in Workbench by FEAT-188** (the author, 2026-10-29): the thick
 * material is solid, and glass is on the toolbar, the toast and the modal
 * scrim only. The history below is why the edge is still asserted.
 *
 * TASK-024: the author reported that a floating pane "still looks just
 * transparent". Three things had made it so, and each of them is asserted here
 * because each is one edit away from coming back:
 *
 * 1. `saturate(0)` drained the backdrop to grey, so what showed through the
 *    pane carried none of the colour that makes a blur read as a material.
 * 2. The tint covered 82% of the backdrop, leaving the frost almost nothing to
 *    be made of.
 * 3. TASK-023 switched off `--glass-rim` and `--glass-sheen` when it flattened
 *    the interface, and nothing replaced them, so a pane had no edge at all.
 *
 * And one thing that must *not* come back: TASK-022 measured the blur radius
 * at 10px costing 16.0ms a frame against the 196ms of the lens it replaced.
 * Widening it is the way this material becomes expensive again, so the radius
 * is pinned rather than merely commented.
 *
 * These assertions read the stylesheets rather than a rendered pane, for the
 * reason `btn.test.ts` gives: the test environment mounts components without
 * applying any CSS, so `getComputedStyle` here would pass whatever the rules
 * say.
 */

import { readdirSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';

const css = readFileSync('src/app.css', 'utf8');

/** The value of a custom property declared on a `:root` rule. */
function token(name: string, scope = ':root {'): string {
	const from = css.indexOf(scope);
	if (from === -1) throw new Error(`no ${scope} rule`);
	const at = css.indexOf(`${name}:`, from);
	if (at === -1) throw new Error(`no ${name} under ${scope}`);
	return css.slice(at + name.length + 1, css.indexOf(';', at)).trim();
}

/** Every `.svelte` file under a directory, recursively. */
function componentsUnder(dir: string): string[] {
	return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
		const path = join(dir, entry.name);
		if (entry.isDirectory()) return componentsUnder(path);
		return entry.name.endsWith('.svelte') ? [path] : [];
	});
}

/** Every component that paints itself with the thick glass material. */
function thickGlassPanes(): { path: string; source: string }[] {
	return [...componentsUnder('src/lib'), ...componentsUnder('src/routes')]
		.map((path) => ({ path, source: readFileSync(path, 'utf8') }))
		.filter(({ source }) => source.includes('var(--blur-thick)'));
}

/**
 * Workbench (FEAT-188, the author, 2026-10-29): glass on the toolbar, the
 * toast and the scrim behind a modal, and nowhere else. The thick material the
 * menus, dialogs, the palette, the command log and the rail paint with is
 * solid, so TASK-024's frost, saturation and blur radius no longer apply to
 * them; the edge (below) still does.
 */
describe('the thick material', () => {
	it('is solid, and blurs nothing, in both themes', () => {
		for (const scope of [':root {', ":root[data-theme='dark'] {"]) {
			expect(token('--glass-thick', scope)).toBe('var(--surface-2)');
			expect(token('--glass', scope)).toBe('var(--surface)');
			expect(token('--glass-thin', scope)).toBe('var(--panel)');
		}
		for (const blur of ['--blur-thick', '--blur', '--blur-thin']) expect(token(blur)).toBe('none');
	});
});

describe("Workbench's liquid glass (FEAT-188)", () => {
	const surfaces = readFileSync('src/lib/workbench/surfaces.css', 'utf8');
	const rule = (selector: string) => {
		const at = surfaces.indexOf(`${selector} {`);
		if (at === -1) throw new Error(`no ${selector}`);
		return surfaces.slice(at, surfaces.indexOf('\n}', at));
	};

	it('is themed by tokens both themes define', () => {
		for (const name of ['--lg-bg', '--lg-rim', '--lg-hi', '--lg-lo', '--lg-sheen', '--lg-drop', '--scrim']) {
			expect(token(name), name).toBeTruthy();
		}
		const dark = css.slice(css.indexOf("/* Night: smoked rather than frosted, and black, not navy. */"));
		for (const name of ['--lg-bg', '--lg-rim', '--lg-hi', '--lg-lo', '--lg-sheen', '--lg-drop', '--scrim']) {
			expect(dark, `${name} in the dark`).toContain(`${name}:`);
		}
	});

	it('frosts the toast and refracts the toolbar, with the frost as the fallback, both prefixes', () => {
		const glass = rule('.lglass');
		expect(glass).toContain('-webkit-backdrop-filter: blur(16px) saturate(180%)');
		expect(glass).toContain('\tbackdrop-filter: blur(16px) saturate(180%)');
		const rail = rule('.ornament.lrail');
		const lines = rail.split('\n').filter((line) => /^\tbackdrop-filter:/.test(line));
		expect(lines).toEqual([
			'\tbackdrop-filter: blur(14px) saturate(180%);',
			'\tbackdrop-filter: url(#lg-refract) blur(2.5px) saturate(185%) brightness(1.04);'
		]);
		expect(rail).toContain('-webkit-backdrop-filter: url(#lg-refract)');
		for (const body of [glass, rail]) expect(body, 'a literal colour').not.toMatch(/#[0-9a-f]{3,8}\b|rgba?\(/i);
	});

	it('puts glass on the toolbar, the toast and the scrims only', () => {
		const toolbar = readFileSync('src/lib/chrome/Toolbar.svelte', 'utf8');
		expect(toolbar).toMatch(/class="toolbar ornament lrail"/);
		expect(readFileSync('src/lib/ui/NoticeToast.svelte', 'utf8')).toMatch(/class="notice lglass"/);
		expect(readFileSync('src/lib/chrome/NavRail.svelte', 'utf8')).not.toMatch(/lglass|lrail/);
		const users = [...componentsUnder('src/lib'), ...componentsUnder('src/routes')]
			.filter((path) => /class="[^"]*\b(lglass|lrail)\b/.test(readFileSync(path, 'utf8')))
			.map((path) => path.replaceAll('\\', '/'))
			.sort();
		expect(users).toEqual(['src/lib/chrome/Toolbar.svelte', 'src/lib/ui/NoticeToast.svelte']);
	});

	it('blurs behind every modal with the scrim, never a glass rim', () => {
		const scrims = [...componentsUnder('src/lib'), ...componentsUnder('src/routes')].filter((path) =>
			readFileSync(path, 'utf8').includes('background: var(--scrim)')
		);
		expect(scrims.length).toBeGreaterThanOrEqual(10);
		for (const path of scrims) {
			const source = readFileSync(path, 'utf8');
			expect(source, path).toContain('-webkit-backdrop-filter: var(--scrim-blur)');
			expect(source, path).toMatch(/\tbackdrop-filter: var\(--scrim-blur\)/);
		}
		expect(token('--scrim-blur')).toBe('blur(10px) saturate(120%)');
	});

	it('refracts through one fixed filter, mounted once', () => {
		const filter = readFileSync('src/lib/workbench/GlassFilter.svelte', 'utf8');
		expect(filter).toContain('id="lg-refract"');
		expect(filter).toContain('seed="11"');
		expect(readFileSync('src/routes/+layout.svelte', 'utf8').match(/<GlassFilter \/>/g)).toHaveLength(1);
	});
});

describe('continuous corners (FEAT-188)', () => {
	const surfaces = readFileSync('src/lib/workbench/surfaces.css', 'utf8');
	const block = surfaces.slice(surfaces.indexOf('@supports (corner-shape: superellipse(2)) {'), surfaces.indexOf('/* `select` pickers'));

	it('is progressive: squircles and their radii only where the webview draws them', () => {
		expect(block).toContain('corner-shape: superellipse(2)');
		expect(block).toContain("[data-corner-shape='round']");
		// Outside an `@supports` block, nothing asks for a squircle.
		const outside = surfaces.split(/@supports[^{]*\{[\s\S]*?\n\}/).join('');
		expect(outside).not.toMatch(/^\s+corner-shape:/m);
		expect(css).not.toMatch(/^\s+corner-shape:/m);
	});

	it('never gives a pill, a chip, an ornament or a switch a squircle', () => {
		const selectors = block.slice(block.indexOf('.app,'), block.indexOf('{', block.indexOf('.app,')));
		for (const shape of ['.chip', '.ornament', 'r-pill', '.toolbar', '.rail']) expect(selectors).not.toContain(shape);
		expect(selectors).toContain(":not([role='switch'])");
		expect(block).not.toContain('--r-pill');
		expect(block).not.toContain('--r-ornament');
	});

	it('makes every squircle radius larger than its circular one', () => {
		for (const [, name, value] of block.matchAll(/--(r-[a-z]+):\s*(\d+)px/g)) {
			const circular = token(`--${name}`);
			expect(Number(value), name).toBeGreaterThan(parseInt(circular, 10));
		}
	});
});

describe('the edge of a floating pane', () => {
	it('gives every floating pane an edge', () => {
		const panes = thickGlassPanes();
		// If this drops to nothing the rest of the suite is asserting over an
		// empty list, which is the failure mode a scan like this has.
		expect(panes.length).toBeGreaterThanOrEqual(5);

		for (const { path, source } of panes) {
			expect(source, `${path} floats with no edge to catch the light`).toMatch(
				/border(-top)?:[^;]*var\(--glass-edge/
			);
		}
	});

	it('draws that edge as a border rather than as the shadow TASK-023 removed', () => {
		// The author asked for a flat interface with no shadows. The edge is
		// what gives a pane thickness; a rim shadow is what was taken away.
		// Both rim tokens stay off, in both themes.
		expect([...css.matchAll(/--glass-(rim|rim-thick|sheen):\s*([^;]+);/g)].length).toBe(6);
		for (const [, , value] of css.matchAll(/--glass-(rim|rim-thick|sheen):\s*([^;]+);/g)) {
			expect(value.trim()).toBe('none');
		}
	});
});

describe('the stylesheet itself', () => {
	it('is brace-balanced', () => {
		// A stray `}` shipped in FEAT-056 and sat in the file until TASK-024.
		// Browsers recover from one, which is exactly why nothing caught it.
		let depth = 0;
		let line = 0;
		for (const text of css.split('\n')) {
			line += 1;
			for (const character of text) {
				if (character === '{') depth += 1;
				if (character === '}') depth -= 1;
				expect(depth, `unbalanced closing brace at src/app.css:${line}`).toBeGreaterThanOrEqual(
					0
				);
			}
		}
		expect(depth, 'src/app.css has an unclosed rule').toBe(0);
	});
});
