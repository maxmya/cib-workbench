// SPDX-License-Identifier: MIT

/**
 * Whether onboarding is on screen, and the launch waiting on it (FEAT-180).
 *
 * A launch that finds `onboarded` unset shows `/welcome` and waits here before
 * it opens anything, so a path on the command line loads after setup. *Run
 * setup again* shows it with nothing waiting.
 */

let showing = $state(false);
let waiting: (() => void) | null = null;

export const onboard = {
	/** True while `/welcome` owns the window: no rail, no title bar tools. */
	get showing(): boolean {
		return showing;
	},

	/** Show onboarding; resolves once it is finished or put off. */
	run(show: () => Promise<void> | void): Promise<void> {
		showing = true;
		const done = new Promise<void>((resolve) => (waiting = resolve));
		void show();
		return done;
	},

	/** Show it again from Settings, with no launch waiting. */
	again(): void {
		showing = true;
	},

	/** Finished, or *Set up later*: the window gets its chrome back. */
	done(): void {
		showing = false;
		waiting?.();
		waiting = null;
	}
};
