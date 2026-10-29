// SPDX-License-Identifier: MIT

/**
 * Loading, replacing and unloading the one repository (FEAT-184), as
 * sequences a test can watch.
 *
 * The backend holds one repository (`AppState.session`). A load replaces it:
 * the place being left is remembered, the new one opens, and its own place
 * comes back. The failure that matters is a step that does not happen, so the
 * tests assert the calls, as `session.test.ts` does for a launch.
 */

import type { Place } from '$lib/workspace.svelte';
import { HOME_ROUTE } from '$lib/session';
import { isScoped } from './scoped';

export interface LoadPort {
	/** The loaded repository's path, or null. */
	loaded(): string | null;
	/** Where the loaded one is now: its route and selected commit. */
	here(): Place;
	remember(path: string, place: Place): void;
	placeOf(path: string): Place | null;
	/** Opens a repository; `false` when it did not resolve. */
	open(path: string): Promise<boolean>;
	close(): Promise<void>;
	/** Shows nothing loaded, keeping every place. */
	rest(): void;
	route(): string;
	goto(route: string): Promise<void>;
	want(id: string): void;
}

/**
 * Load `path`, replacing whatever is loaded.
 *
 * `then` is where to go afterwards: a dimmed rail row's screen. Without it,
 * the repository's own place comes back when it had one on a repository
 * screen; otherwise the window stays where it is.
 */
export async function loadRepository(port: LoadPort, path: string, then?: string): Promise<boolean> {
	const leaving = port.loaded();
	if (leaving === path) {
		if (then && then !== port.route()) await port.goto(then);
		return true;
	}
	if (leaving) port.remember(leaving, port.here());

	const place = port.placeOf(path);
	if (!(await port.open(path))) return false;

	const target = then ?? (place && isScoped(place.route) && isScoped(port.route()) ? place.route : null);
	if (target && target !== port.route()) await port.goto(target);
	if (place?.selected && (!then || then === place.route)) port.want(place.selected);
	return true;
}

/** Unload: nothing loaded, every place kept, and Home if on a repository screen. */
export async function unloadRepository(port: LoadPort): Promise<void> {
	const leaving = port.loaded();
	if (!leaving) return;
	port.remember(leaving, port.here());
	await port.close();
	port.rest();
	if (isScoped(port.route())) await port.goto(HOME_ROUTE);
}
