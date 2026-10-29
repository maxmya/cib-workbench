// SPDX-License-Identifier: MIT

/**
 * A working-copy file shown whole (FEAT-189): its hunks as they are, since
 * they are what is staged, and the unchanged lines between them taken from
 * the file read whole.
 *
 * Both come from the same diff of the same two sides, so an unchanged line of
 * the whole file is either inside a hunk, as one of its context lines, or in
 * a gap between two. The gaps are found by the old side's numbers: an
 * unchanged line has one, and so does every hunk that has an old side at all.
 */

import type { DiffLine, Hunk } from '$lib/types';

export interface Gaps {
	/** The unchanged lines before each hunk, by the hunk's index. */
	before: DiffLine[][];
	/** The unchanged lines after the last hunk. */
	after: DiffLine[];
}

export function gapsOf(hunks: readonly Hunk[], whole: readonly DiffLine[]): Gaps {
	const unchanged = whole.filter((line) => line.old !== null && line.origin === 'context');
	/** The next unchanged line not yet placed, and the last old line a hunk holds. */
	let at = 0;
	let held = 0;

	const before = hunks.map((hunk) => {
		const olds = hunk.lines.flatMap((line) => (line.old === null ? [] : [line.old]));
		// A hunk with no old side is a file that was empty: nothing to fill.
		if (olds.length === 0) return [];
		const gap: DiffLine[] = [];
		for (; at < unchanged.length && (unchanged[at].old as number) < olds[0]; at += 1) {
			if ((unchanged[at].old as number) > held) gap.push(unchanged[at]);
		}
		held = olds[olds.length - 1];
		return gap;
	});

	const after = unchanged.slice(at).filter((line) => (line.old as number) > held);
	return { before, after };
}
