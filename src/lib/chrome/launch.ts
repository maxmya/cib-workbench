// SPDX-License-Identifier: MIT

/**
 * How long the launch screen stays (FEAT-168).
 *
 * It carries the internal-use notice, the trademark line and the licence, and
 * a fast open took it away before any of it could be read. So it stays at
 * least this long from when the window appeared, however soon the repository
 * is open. A click or a key once the repository is open lets it go at once:
 * somebody who has read it every morning need not wait for it.
 */
export const LAUNCH_MIN_MS = 2000;

/** How much longer to wait, from `startedAt`, at `now`. */
export function launchRemaining(startedAt: number, now: number, min = LAUNCH_MIN_MS): number {
	return Math.max(0, startedAt + min - now);
}

/**
 * Hold the launch screen: resolves once `opening` is settled and `min` has
 * passed since `startedAt`, or as soon as `skip` is called after `opening`
 * settled. `skip` before then is remembered and honoured when it settles.
 */
export function holdLaunch(
	opening: Promise<unknown>,
	startedAt: number,
	options: { now?: () => number; wait?: (ms: number) => Promise<void>; min?: number } = {}
): { done: Promise<void>; skip: () => void } {
	const now = options.now ?? (() => performance.now());
	const wait = options.wait ?? ((ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms)));
	let skipped = false;
	let release: () => void = () => {};
	const skippedNow = new Promise<void>((resolve) => (release = resolve));

	const done = (async () => {
		await opening.catch(() => {});
		if (skipped) return;
		const left = launchRemaining(startedAt, now(), options.min);
		if (left > 0) await Promise.race([wait(left), skippedNow]);
	})();

	return {
		done,
		skip() {
			skipped = true;
			release();
		}
	};
}
