// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import type { DiffLine, Hunk } from '$lib/types';
import { gapsOf } from './whole';

const same = (old: number, now = old): DiffLine => ({ origin: 'context', old, new: now, text: `line ${old}` });
const gone = (old: number): DiffLine => ({ origin: 'removed', old, new: null, text: `old ${old}` });
const came = (now: number): DiffLine => ({ origin: 'added', old: null, new: now, text: `new ${now}` });

function hunk(lines: DiffLine[]): Hunk {
	return { oldStart: 0, oldLines: 0, newStart: 0, newLines: 0, header: '@@', lines };
}

const olds = (lines: DiffLine[]) => lines.map((line) => line.old);

describe('a file shown whole (FEAT-189)', () => {
	// Ten lines; line 2 replaced, line 8 removed, one context line either side.
	const whole = [
		same(1),
		gone(2),
		came(2),
		...[3, 4, 5, 6, 7].map((n) => same(n)),
		gone(8),
		same(9, 8),
		same(10, 9)
	];
	const hunks = [hunk([same(1), gone(2), came(2), same(3)]), hunk([same(7), gone(8), same(9, 8)])];

	it('fills the unchanged lines between, before and after the hunks', () => {
		const gaps = gapsOf(hunks, whole);

		expect(gaps.before.map(olds)).toEqual([[], [4, 5, 6]]);
		expect(olds(gaps.after)).toEqual([10]);
		expect(gaps.after[0].new).toBe(9);
	});

	it('leaves out nothing a hunk does not hold, and repeats nothing it does', () => {
		const gaps = gapsOf(hunks, whole);
		const shown = [...gaps.before.flat(), ...hunks.flatMap((h) => h.lines), ...gaps.after];

		expect(shown.filter((line) => line.old !== null).map((line) => line.old).sort((a, b) => a! - b!)).toEqual([
			1, 2, 3, 4, 5, 6, 7, 8, 9, 10
		]);
	});

	it('has nothing to fill for a file that was empty', () => {
		const gaps = gapsOf([hunk([came(1), came(2)])], [came(1), came(2)]);

		expect(gaps.before).toEqual([[]]);
		expect(gaps.after).toEqual([]);
	});
});
