// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import type { Place } from '$lib/workspace.svelte';
import { loadRepository, unloadRepository, type LoadPort } from './load';
import { isScoped } from './scoped';

/** A port that records what it was asked, in order. */
function recorder(start: { loaded?: string | null; route?: string; places?: Record<string, Place>; fails?: string[] } = {}) {
	const calls: string[] = [];
	let loaded = start.loaded ?? null;
	let route = start.route ?? '/home';
	const places: Record<string, Place> = { ...(start.places ?? {}) };
	const port: LoadPort = {
		loaded: () => loaded,
		here: () => ({ route, selected: loaded ? `sel-${loaded}` : null }),
		remember: (path, place) => {
			calls.push(`remember ${path} ${place.route} ${place.selected}`);
			places[path] = place;
		},
		placeOf: (path) => places[path] ?? null,
		open: async (path) => {
			calls.push(`open ${path}`);
			if (start.fails?.includes(path)) return false;
			loaded = path;
			return true;
		},
		close: async () => {
			calls.push('close');
			loaded = null;
		},
		rest: () => calls.push('rest'),
		route: () => route,
		goto: async (next) => {
			calls.push(`goto ${next}`);
			route = next;
		},
		want: (id) => calls.push(`want ${id}`)
	};
	return { port, calls, places, get loaded() { return loaded; }, get route() { return route; } };
}

describe('loading a repository (FEAT-184)', () => {
	it('replaces the loaded one, and loading it again comes back to its route and commit', async () => {
		const run = recorder({ loaded: '/a', route: '/branches' });

		await loadRepository(run.port, '/b');
		expect(run.loaded).toBe('/b');
		expect(run.calls).toEqual(['remember /a /branches sel-/a', 'open /b']);

		run.calls.length = 0;
		await loadRepository(run.port, '/a');
		expect(run.loaded).toBe('/a');
		expect(run.calls).toEqual(['remember /b /branches sel-/b', 'open /a', 'want sel-/a']);
	});

	it("goes back to the repository's own screen from another repository screen", async () => {
		const run = recorder({ loaded: '/b', route: '/', places: { '/a': { route: '/stash', selected: null } } });
		await loadRepository(run.port, '/a');
		expect(run.route).toBe('/stash');
	});

	it('stays on Home when loaded from Home', async () => {
		const run = recorder({ route: '/home', places: { '/a': { route: '/stash', selected: 'c1' } } });
		await loadRepository(run.port, '/a');
		expect(run.route).toBe('/home');
		expect(run.calls).toEqual(['open /a', 'want c1']);
	});

	it('goes to the screen a dimmed rail row asked for, once the load finishes', async () => {
		const run = recorder({ route: '/home' });
		await loadRepository(run.port, '/a', '/branches');
		expect(run.calls).toEqual(['open /a', 'goto /branches']);
	});

	it('goes nowhere when the repository does not open', async () => {
		const run = recorder({ route: '/home', fails: ['/gone'] });
		expect(await loadRepository(run.port, '/gone', '/branches')).toBe(false);
		expect(run.calls).toEqual(['open /gone']);
	});

	it('loading the loaded one only navigates', async () => {
		const run = recorder({ loaded: '/a', route: '/home' });
		await loadRepository(run.port, '/a', '/changes');
		expect(run.calls).toEqual(['goto /changes']);
	});

	it('unloads to nothing, keeps the place, and leaves a repository screen for Home', async () => {
		const run = recorder({ loaded: '/a', route: '/changes' });
		await unloadRepository(run.port);
		expect(run.calls).toEqual(['remember /a /changes sel-/a', 'close', 'rest', 'goto /home']);
		expect(run.places['/a'].route).toBe('/changes');
	});

	it('unloads in place on a Workbench-wide screen', async () => {
		const run = recorder({ loaded: '/a', route: '/calls' });
		await unloadRepository(run.port);
		expect(run.calls).toEqual(['remember /a /calls sel-/a', 'close', 'rest']);
	});
});

describe('which screens need a repository', () => {
	it('dims the repository tools and never Home, Calls, Farm or Settings', () => {
		for (const path of ['/', '/changes', '/branches/', '/review', '/files', '/terminal', '/history/x']) {
			expect(isScoped(path), path).toBe(true);
		}
		for (const path of ['/home', '/calls', '/farm', '/settings', '/repos', '/welcome', '/requests', '/pipelines']) {
			expect(isScoped(path), path).toBe(false);
		}
	});
});
