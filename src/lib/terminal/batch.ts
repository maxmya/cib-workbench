// SPDX-License-Identifier: MIT

/**
 * Output, gathered and written once a frame (FEAT-130).
 *
 * `yes | head -n 1000000` arrives as thousands of small pieces in a second.
 * Handing each to the emulator as it arrives makes it parse and schedule a
 * draw for each, and the window stops answering. Gathered, a frame's worth is
 * one write: the emulator parses it in one go and draws once.
 *
 * Nothing is dropped and nothing is reordered. A piece that ends in the
 * middle of a character or an escape sequence is joined to the next before
 * the emulator sees either, which is also the only correct way to hand them
 * over.
 *
 * Given a clock, the first piece after a quiet frame is written at once
 * (BUG-111): a keystroke's echo is one small piece, and holding it for the
 * frame was a frame of lag on every key. Only a piece that ends on a whole
 * character goes early; the rest of a busy frame is gathered as before.
 */

export type Schedule = (work: () => void) => void;

/** A frame, in milliseconds: quieter than this, and the next piece goes at once. */
export const QUIET_MS = 16;

/** True when `bytes` does not end part-way through a UTF-8 character. */
export function endsWhole(bytes: Uint8Array): boolean {
	// Back over the continuation bytes to the character's first byte.
	let back = 0;
	while (back < 4 && back < bytes.length && (bytes[bytes.length - 1 - back] & 0xc0) === 0x80) back++;
	if (back === bytes.length) return back === 0;
	const lead = bytes[bytes.length - 1 - back];
	const length = lead < 0x80 ? 1 : lead >= 0xf0 ? 4 : lead >= 0xe0 ? 3 : lead >= 0xc0 ? 2 : 1;
	return back + 1 >= length;
}

const nextFrame: Schedule = (work) => {
	if (typeof requestAnimationFrame === 'function') requestAnimationFrame(() => work());
	else setTimeout(work, 16);
};

export class Batcher {
	private pieces: Uint8Array[] = [];
	private size = 0;
	private waiting = false;
	private last = Number.NEGATIVE_INFINITY;

	constructor(
		private readonly write: (bytes: Uint8Array) => void,
		private readonly schedule: Schedule = nextFrame,
		/** Milliseconds, for the first piece after a quiet frame to go at once. None: every piece waits. */
		private readonly now?: () => number
	) {}

	/** Take a piece. It is written with everything else that arrives this frame. */
	push(bytes: Uint8Array): void {
		if (bytes.length === 0) return;
		this.pieces.push(bytes);
		this.size += bytes.length;
		if (this.waiting) return;
		if (this.now && this.pieces.length === 1 && this.now() - this.last >= QUIET_MS && endsWhole(bytes)) {
			this.flush();
			return;
		}
		this.waiting = true;
		this.schedule(() => this.flush());
	}

	/** Write what has been gathered, now. Called by the frame, and before a terminal is closed. */
	flush(): void {
		this.waiting = false;
		if (this.pieces.length === 0) return;
		if (this.now) this.last = this.now();
		const all = this.pieces.length === 1 ? this.pieces[0] : join(this.pieces, this.size);
		this.pieces = [];
		this.size = 0;
		this.write(all);
	}

	/** How many bytes are waiting for the frame. */
	get pending(): number {
		return this.size;
	}
}

function join(pieces: Uint8Array[], size: number): Uint8Array {
	const all = new Uint8Array(size);
	let at = 0;
	for (const piece of pieces) {
		all.set(piece, at);
		at += piece.length;
	}
	return all;
}
