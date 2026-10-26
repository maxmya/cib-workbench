// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { holdLaunch, LAUNCH_MIN_MS, launchRemaining } from './launch';

/** FEAT-168: the launch screen stays long enough to be read. */
describe('holding the launch screen', () => {
	it('stays two seconds from when the window appeared', () => {
		expect(LAUNCH_MIN_MS).toBe(2000);
		expect(launchRemaining(100, 600)).toBe(1500);
		expect(launchRemaining(100, 2100)).toBe(0);
		expect(launchRemaining(100, 9000)).toBe(0);
	});

	it('waits out the rest of the two seconds after a fast open', async () => {
		const waited: number[] = [];
		const held = holdLaunch(Promise.resolve(), 0, {
			now: () => 300,
			wait: async (ms) => void waited.push(ms)
		});
		await held.done;
		expect(waited).toEqual([1700]);
	});

	it('does not wait at all after a slow open', async () => {
		const waited: number[] = [];
		await holdLaunch(Promise.resolve(), 0, { now: () => 5000, wait: async (ms) => void waited.push(ms) }).done;
		expect(waited).toEqual([]);
	});

	it('gives way at once to a skip, but never before the repository is open', async () => {
		let open: () => void = () => {};
		const opening = new Promise<void>((resolve) => (open = resolve));
		let ended = false;
		const held = holdLaunch(opening, 0, { now: () => 100, wait: () => new Promise(() => {}) });
		void held.done.then(() => (ended = true));

		held.skip();
		await Promise.resolve();
		expect(ended).toBe(false);

		open();
		await held.done;
		expect(ended).toBe(true);
	});

	it('gives way even when the open failed', async () => {
		await holdLaunch(Promise.reject(new Error('gone')), 0, { now: () => 5000 }).done;
	});
});
