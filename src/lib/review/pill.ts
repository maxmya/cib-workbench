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
