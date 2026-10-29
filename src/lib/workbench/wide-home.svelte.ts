// SPDX-License-Identifier: MIT

/**
 * What Workbench's Home shows across followed repositories (FEAT-183).
 *
 * One read, `home_workbench_wide`, for every Workbench-wide card. The last
 * answer paints first and is read again; after that Home rides the forge
 * watcher's clock while it is on screen and adds no timer of its own. A
 * project read within the watcher's interval is not read again (Rust decides).
 */

import { invoke } from '@tauri-apps/api/core';
import * as core from '$lib/api';
import type { HistoryEntry, Loaded } from '$lib/calls/types';
import { watcher } from '$lib/notifications/watcher.svelte';
import { describe } from '$lib/ui/notice.svelte';
import * as api from './api';
import { EMPTY_HOME, recentCollections, type RecentCollection } from './home';
import type { WideHome } from './types';

let home = $state<WideHome>(EMPTY_HOME);
let loading = $state(false);
let error = $state<string | null>(null);
let calls = $state<RecentCollection[]>([]);
let callsError = $state<string | null>(null);
let unsubscribe: (() => void) | null = null;
let asked = 0;

async function read(force: boolean): Promise<void> {
	if (!core.inTauri()) return;
	const ticket = ++asked;
	loading = true;
	try {
		const next = await api.homeWide(force);
		if (ticket !== asked) return;
		home = next;
		error = null;
	} catch (cause) {
		if (ticket === asked) error = describe(cause);
	} finally {
		if (ticket === asked) loading = false;
	}
}

/** Workbench-wide Calls, read without touching the scope the Calls screen is on. */
async function readCalls(): Promise<void> {
	if (!core.inTauri()) return;
	try {
		const [loaded, history] = await Promise.all([
			invoke<Loaded>('calls_load', { scope: 'global' }),
			invoke<HistoryEntry[]>('calls_history', { scope: 'global' }).catch(() => [])
		]);
		calls = recentCollections(loaded.collections, history);
		callsError = null;
	} catch (cause) {
		callsError = describe(cause);
	}
}

export const wideHome = {
	get home(): WideHome {
		return home;
	},
	get loading(): boolean {
		return loading;
	},
	get error(): string | null {
		return error;
	},
	get calls(): RecentCollection[] {
		return calls;
	},
	get callsError(): string | null {
		return callsError;
	},
	/** When the cards were last read from GitLab, in milliseconds. */
	get updated(): number | null {
		const times = [home.readWide, ...Object.values(home.read)].filter((t) => t > 0);
		return times.length ? Math.max(...times) * 1000 : null;
	},

	/** Home is shown: paint the last answer, then read again. */
	async open(connected: boolean): Promise<void> {
		if (!core.inTauri()) return;
		void readCalls();
		if (!connected) return;
		const saved = await api.homeWideCached().catch(() => null);
		if (saved && asked === 0) home = saved;
		await read(false);
	},

	/** Refresh: read everything now. */
	async refresh(connected: boolean): Promise<void> {
		void readCalls();
		if (connected) await read(true);
	},

	/** While Home is on screen, it reads on the watcher's clock. */
	follow(everyMinutes: number, connected: () => boolean): () => void {
		unsubscribe?.();
		unsubscribe = watcher.subscribe(() => (connected() ? read(false) : undefined), everyMinutes * 60_000);
		return () => {
			unsubscribe?.();
			unsubscribe = null;
		};
	},

	/** For tests. */
	present(next: { home?: WideHome; calls?: RecentCollection[]; error?: string | null }): void {
		if (next.home) home = next.home;
		if (next.calls) calls = next.calls;
		if (next.error !== undefined) error = next.error;
	}
};
