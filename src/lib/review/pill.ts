// SPDX-License-Identifier: MIT

/**
 * Where the review controls sit (BUG-104).
 *
 * The pill floats in the diff column, and the column is off the window's
 * centre whenever the file list and the conversation differ in width. Centred
 * in the column, it sat beside the toolbar under the window rather than over
 * it, and looked misplaced. It is moved onto the window's centre line, as far
 * as the column allows: never past `margin` from either of the column's edges.
 *
 * Answers the move in pixels, positive to the right.
 */
export function centreOffset(
	column: { left: number; width: number },
	windowWidth: number,
	pillWidth: number,
	margin = 8
): number {
	const wanted = windowWidth / 2 - (column.left + column.width / 2);
	const room = Math.max(0, (column.width - pillWidth) / 2 - margin);
	return Math.round(Math.min(room, Math.max(-room, wanted)));
}

/**
 * How tight the review controls are drawn (BUG-109): the first of `last + 1`
 * levels whose width fits in `room`, or the tightest. The pill used to wrap
 * onto a second row when the column narrowed — zoomed in, or with a wide
 * conversation beside it — and covered the diff it controls. It keeps one row
 * and draws its labels shorter instead, down to a fixed smallest size.
 */
export function fitLevel(room: number, widthAt: (level: number) => number, last: number): number {
	for (let level = 0; level < last; level++) {
		if (widthAt(level) <= room) return level;
	}
	return last;
}
