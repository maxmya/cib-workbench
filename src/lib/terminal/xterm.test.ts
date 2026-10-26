// SPDX-License-Identifier: MIT

/**
 * xterm.js behind the emulator's interface (FEAT-130).
 *
 * There is no canvas here, so nothing is drawn and nothing in this file says
 * what a terminal looks like. What can be said without one: that an emulator
 * is made and kept before it is in the page, that it takes output meanwhile,
 * and what it is set up with.
 */

import { afterEach, describe, expect, it, vi } from 'vitest';
import type { EmulatorHooks } from './emulator';
import { makeXterm, multiplexerStyle, SCROLLBACK, SYMBOLS_FONT, terminalFont } from './xterm';

const hooks = (): EmulatorHooks => ({ onData: vi.fn(), links: vi.fn(async () => []), open: vi.fn() });
const made: { dispose(): void }[] = [];

function make(given: EmulatorHooks = hooks()) {
	const emulator = makeXterm(given);
	made.push(emulator);
	return emulator;
}

afterEach(() => {
	for (const emulator of made.splice(0)) emulator.dispose();
});

describe('the emulator', () => {
	it('keeps ten thousand lines above the screen', () => {
		expect(SCROLLBACK).toBe(10_000);
	});

	it('is made with an element of its own that fills what it is put in', () => {
		const emulator = make();

		expect(emulator.element.className).toBe('terminal-host');
		expect(emulator.element.style.width).toBe('100%');
		expect(emulator.element.style.height).toBe('100%');
		expect(emulator.element.isConnected).toBe(false);
	});

	/** A terminal is made when its program is asked for, which is before the screen has put it anywhere. */
	it('says a terminal’s usual size until it is in the page, and draws nothing there', () => {
		const emulator = make();

		expect(emulator.fit()).toEqual({ cols: 80, rows: 24 });
		expect(emulator.element.childElementCount).toBe(0);
	});

	it('takes output before it is in the page', () => {
		const emulator = make();

		expect(() => emulator.write(new TextEncoder().encode('built in 4.2s\r\n'))).not.toThrow();
		expect(emulator.selection()).toBe('');
	});

	it('takes its element out of the page when it is let go', () => {
		const emulator = makeXterm(hooks());
		document.body.appendChild(emulator.element);

		emulator.dispose();

		expect(emulator.element.isConnected).toBe(false);
	});
});

/** FEAT-157: the multiplexer is sent nothing but colours it can write. */
describe("the multiplexer's colours", () => {
	it('are every one a #rrggbb', () => {
		const style = multiplexerStyle();

		expect(Object.keys(style).sort()).toEqual(
			['accent', 'danger', 'ground', 'ink', 'line', 'muted', 'ok', 'onAccent', 'selection'].sort()
		);
		for (const [name, colour] of Object.entries(style))
			expect(colour, name).toMatch(/^#[0-9a-f]{6}$/);
	});

	it('are taken again by an open terminal when the theme changes', () => {
		const emulator = make();

		expect(() => emulator.retheme()).not.toThrow();
	});
});

/** BUG-103: a prompt's icons are in no code face, so the symbols come after it. */
describe('the terminal font', () => {
	it('puts the Nerd Font symbols behind the code face', () => {
		expect(terminalFont("'JetBrains Mono', monospace")).toBe(
			`'JetBrains Mono', monospace, '${SYMBOLS_FONT}'`
		);
	});

	it('has a face when the theme gives none, and names the symbols once', () => {
		expect(terminalFont('  ')).toBe(`monospace, '${SYMBOLS_FONT}'`);
		const once = terminalFont('monospace');
		expect(terminalFont(once)).toBe(once);
	});

	it('is bundled under the name the terminal asks for', async () => {
		const { readFileSync } = await import('node:fs');
		const css = readFileSync('src/app.css', 'utf8');
		expect(css).toContain(`font-family: '${SYMBOLS_FONT}'`);
		expect(css).toContain('assets/fonts/symbols-nerd-font/SymbolsNerdFontMono-Regular.ttf');
	});
});
