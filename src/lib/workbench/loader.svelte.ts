// SPDX-License-Identifier: MIT

/**
 * Load repository (2E, FEAT-184): the dialog's state, and the loads it runs.
 *
 * One repository is loaded at a time. The order of a load lives in `load.ts`
 * behind a port; this supplies the real operations. A clone survives the
 * dialog closing, like the clone screen's, and loads when it finishes.
 */

import { goto } from '$app/navigation';
import { page } from '$app/state';
import { listen } from '@tauri-apps/api/event';
import * as api from '$lib/api';
import { graph } from '$lib/graph/store.svelte';
import { repo } from '$lib/repo.svelte';
import { workspace } from '$lib/workspace.svelte';
import {
	CLONE_DONE_EVENT,
	CLONE_PROGRESS_EVENT,
	type CloneDoneEvent,
	type CloneProgressEvent
} from '$lib/types';
import { loadRepository, unloadRepository, type LoadPort } from './load';
import type { Project } from './types';

/** What a row in the dialog is doing. */
export interface Busy {
	/** The row's key: a path, or `gitlab:<id>`. */
	key: string;
	text: string;
	percent: number | null;
}

let open = $state(false);
let then = $state<string | null>(null);
let selected = $state<number | null>(null);
let busy = $state<Busy | null>(null);
let error = $state<string | null>(null);
let unloading = $state(false);

export const realPort: LoadPort = {
	loaded: () => repo.info?.path ?? null,
	here: () => ({ route: page.url.pathname, selected: graph.selectedId }),
	remember: (path, place) => workspace.remember(path, place),
	placeOf: (path) => workspace.placeOf(path),
	open: (path) => repo.open(path),
	close: () => repo.close(),
	rest: () => workspace.rest(),
	route: () => page.url.pathname,
	goto: (route) => goto(route),
	want: (id) => graph.want(id)
};

export const loader = {
	get open(): boolean {
		return open;
	},
	/** The screen to go to once something is loaded. */
	get then(): string | null {
		return then;
	},
	/** A GitLab project to show first, from a Home row. */
	get selected(): number | null {
		return selected;
	},
	get busy(): Busy | null {
		return busy;
	},
	get error(): string | null {
		return error;
	},

	show(options: { then?: string | null; project?: number | null } = {}): void {
		then = options.then ?? null;
		selected = options.project ?? null;
		error = null;
		open = true;
	},

	/** Close the dialog. A clone that is running keeps running. */
	hide(): void {
		open = false;
		if (!busy) {
			then = null;
			selected = null;
		}
	},

	toggle(): void {
		if (open) this.hide();
		else this.show();
	},

	/** Load a repository on this machine. */
	async load(path: string, port: LoadPort = realPort): Promise<boolean> {
		if (busy) return false;
		busy = { key: path, text: 'Loading…', percent: null };
		error = null;
		try {
			const done = await loadRepository(port, path, then ?? undefined);
			if (done) {
				open = false;
				then = null;
				selected = null;
			} else {
				error = repo.error ?? 'Could not load this repository';
			}
			return done;
		} finally {
			busy = null;
		}
	},

	/** Ask for a folder, then load it. */
	async loadFolder(): Promise<boolean> {
		const { open: pick } = await import('@tauri-apps/plugin-dialog');
		const chosen = await pick({ directory: true, multiple: false, title: 'Load repository' });
		if (typeof chosen !== 'string') return false;
		return this.load(chosen);
	},

	/**
	 * Clone a CIB GitLab project into `parent`, then load it. A destination
	 * that already holds the clone is loaded as it is.
	 */
	async cloneAndLoad(project: Project, parent: string): Promise<boolean> {
		if (busy || !api.inTauri()) return false;
		const key = `gitlab:${project.id}`;
		busy = { key, text: 'Cloning…', percent: null };
		error = null;
		try {
			const plan = await api.clonePlan(project.httpUrl, parent);
			if (plan.problem?.kind === 'destinationNotEmpty' && plan.destination) {
				busy = { key, text: 'Loading…', percent: null };
				const done = await loadRepository(realPort, plan.destination, then ?? undefined);
				if (!done) error = plan.message ?? 'That folder is not empty';
				return done;
			}
			if (plan.problem || !plan.destination) {
				error = plan.message ?? 'Cannot clone here';
				return false;
			}
			const path = await new Promise<string>((resolve, reject) => {
				let token: number | null = null;
				const offs: Array<() => void> = [];
				const stop = () => offs.forEach((off) => off());
				void (async () => {
					offs.push(
						await listen<CloneProgressEvent>(CLONE_PROGRESS_EVENT, (event) => {
							if (event.payload.token !== token) return;
							busy = { key, text: 'Cloning…', percent: event.payload.percent };
						})
					);
					offs.push(
						await listen<CloneDoneEvent>(CLONE_DONE_EVENT, (event) => {
							if (event.payload.token !== token) return;
							stop();
							void api.cloneRelease();
							if (event.payload.ok) resolve(event.payload.path);
							else reject(event.payload.cancelled ? 'Cancelled' : (event.payload.error ?? 'The clone failed'));
						})
					);
					try {
						token = await api.cloneStart(project.httpUrl, parent);
					} catch (cause) {
						stop();
						reject(cause);
					}
				})();
			});
			busy = { key, text: 'Loading…', percent: null };
			const done = await loadRepository(realPort, path, then ?? undefined);
			if (done) {
				open = false;
				then = null;
				selected = null;
			}
			return done;
		} catch (cause) {
			error = String(cause);
			return false;
		} finally {
			busy = null;
		}
	},

	/** True while an unload is on its way to Home. */
	get unloading(): boolean {
		return unloading;
	},

	async unload(port: LoadPort = realPort): Promise<void> {
		unloading = true;
		try {
			await unloadRepository(port);
		} finally {
			unloading = false;
		}
	},

	/** For tests. */
	clear(): void {
		open = false;
		then = null;
		selected = null;
		busy = null;
		error = null;
		unloading = false;
	}
};
