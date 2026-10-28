// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { centreOffset, fitLevel } from './pill';

/** BUG-104: the review controls line up with the toolbar under the window. */
describe('the review pill', () => {
	it('stays put when the column is centred in the window', () => {
		expect(centreOffset({ left: 200, width: 800 }, 1200, 500)).toBe(0);
	});

	it('moves onto the window’s centre line when the column is off it', () => {
		// Column centre 700, window centre 600: a hundred to the left.
		expect(centreOffset({ left: 300, width: 800 }, 1200, 400)).toBe(-100);
	});

	it('goes no further than the column allows', () => {
		// Room either side: (800 - 600) / 2 - 8 = 92.
		expect(centreOffset({ left: 400, width: 800 }, 1200, 600)).toBe(-92);
		expect(centreOffset({ left: 0, width: 800 }, 1600, 600)).toBe(92);
	});

	it('does not move a pill as wide as its column', () => {
		expect(centreOffset({ left: 300, width: 500 }, 1200, 520)).toBe(0);
	});
});

/** BUG-109: the pill draws itself tighter rather than wrap. */
describe('how tight the review pill is', () => {
	const widths = [820, 690, 520, 380];
	const widthAt = (level: number) => widths[level];

	it('is drawn whole when the column holds it', () => {
		expect(fitLevel(900, widthAt, 3)).toBe(0);
	});

	it('takes the first level that fits', () => {
		expect(fitLevel(700, widthAt, 3)).toBe(1);
		expect(fitLevel(520, widthAt, 3)).toBe(2);
	});

	it('stops at its smallest, whatever the room', () => {
		expect(fitLevel(100, widthAt, 3)).toBe(3);
	});
});
