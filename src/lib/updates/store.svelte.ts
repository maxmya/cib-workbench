// SPDX-License-Identifier: MIT

import * as api from '$lib/api';
import { settings } from '$lib/settings/store.svelte';
import type { Prepared } from '$lib/types';

/**
 * Workbench updating itself (FEAT-187).
 *
 * One check, at startup when the preference allows it and whenever Settings'
 * *Check now* is pressed. A newer release is downloaded in the background and
 * verified, then waits: the status strip and Settings offer *Restart*, and
 * nothing is installed until somebody presses it.
 *
 * A build that cannot install updates — a development build, one with no
 * update key, one no installer put there — falls back to the notice and link
 * `settings.checkForUpdate` has always given.
 */

export type Phase = 'idle' | 'checking' | 'downloading' | 'current' | 'ready' | 'unsupported' | 'restarting';

let phase = $state<Phase>('idle');
let received = $state(0);
let total = $state<number | null>(null);
let version = $state<string | null>(null);
let reason = $state<string | null>(null);
let error = $state<string | null>(null);

function settle(prepared: Prepared): void {
	phase = prepared.state;
	version = prepared.state === 'unsupported' ? null : prepared.version;
	reason = prepared.state === 'unsupported' ? prepared.reason : null;
}

export const updates = {
	get phase(): Phase {
		return phase;
	},
	/** The version downloaded, or the one running when it is current. */
	get version(): string | null {
		return version;
	},
	/** Why this build cannot install updates, when it cannot. */
	get reason(): string | null {
		return reason;
	},
	get error(): string | null {
		return error;
	},
	/** Whole percent downloaded, or null while the size is unknown. */
	get percent(): number | null {
		return total ? Math.min(100, Math.floor((received * 100) / total)) : null;
	},
	get busy(): boolean {
		return phase === 'checking' || phase === 'downloading' || phase === 'restarting';
	},

	/** Look for a newer release, and download it if there is one. */
	async check(): Promise<void> {
		if (!api.inTauri() || this.busy || phase === 'ready') return;
		phase = 'checking';
		error = null;
		received = 0;
		total = null;
		try {
			const prepared = await api.prepareUpdate((bytes, size) => {
				phase = 'downloading';
				received = bytes;
				total = size;
			});
			settle(prepared);
			if (prepared.state === 'unsupported') await settings.checkForUpdate();
		} catch (e) {
			phase = 'idle';
			error = String(e);
		}
	},

	/** Install what was downloaded and start the new version. */
	async restart(): Promise<void> {
		if (phase !== 'ready') return;
		phase = 'restarting';
		error = null;
		try {
			await api.restartToUpdate();
		} catch (e) {
			// The download is spent on a failed install; the next check
			// fetches it again.
			phase = 'idle';
			error = String(e);
		}
	}
};

/** Back to a fresh launch, for tests. */
export function resetUpdates(): void {
	phase = 'idle';
	received = 0;
	total = null;
	version = null;
	reason = null;
	error = null;
}
