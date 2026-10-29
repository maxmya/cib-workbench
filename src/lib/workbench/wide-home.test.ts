// SPDX-License-Identifier: MIT

import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({ inTauri: vi.fn(() => true) }));
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));
vi.mock('./api', () => ({ homeWide: vi.fn(), homeWideCached: vi.fn() }));

import { invoke } from '@tauri-apps/api/core';
import * as api from './api';
import { EMPTY_HOME } from './home';
import { wideHome } from './wide-home.svelte';

const saved = { ...EMPTY_HOME, readWide: 100, read: { 7: 200 } };
const fresh = { ...EMPTY_HOME, readWide: 300, read: { 7: 300 } };

beforeEach(() => {
	vi.clearAllMocks();
	vi.mocked(invoke).mockImplementation(async (command: string) =>
		command === 'calls_load'
			? { collections: [{ stem: 'core', file: 'core.json', error: null, value: { v: 1, name: 'Core', items: [] } }] }
			: [{ id: '1', at: 5, collection: 'core', environment: 'UAT' }]
	);
});

describe("Home's reads (FEAT-183)", () => {
	it('paints the saved answer, then reads again; Calls from the Workbench-wide place', async () => {
		vi.mocked(api.homeWideCached).mockResolvedValueOnce(saved);
		let release!: (value: typeof fresh) => void;
		vi.mocked(api.homeWide).mockReturnValueOnce(new Promise((resolve) => (release = resolve)));

		const done = wideHome.open(true);
		await vi.waitFor(() => expect(wideHome.loading).toBe(true));
		expect(wideHome.updated).toBe(200_000);
		release(fresh);
		await done;

		expect(api.homeWide).toHaveBeenCalledWith(false);
		expect(wideHome.home).toEqual(fresh);
		expect(wideHome.updated).toBe(300_000);
		expect(invoke).toHaveBeenCalledWith('calls_load', { scope: 'global' });
		await vi.waitFor(() => expect(wideHome.calls).toEqual([{ stem: 'core', name: 'Core', requests: 0, used: 5, environment: 'UAT' }]));
	});

	it('reads only Calls when nothing is connected', async () => {
		await wideHome.open(false);
		expect(api.homeWide).not.toHaveBeenCalled();
		expect(invoke).toHaveBeenCalledWith('calls_load', { scope: 'global' });
	});

	it('reads everything on Refresh, and says why a read failed', async () => {
		vi.mocked(api.homeWide).mockRejectedValueOnce('GitLab is down');
		await wideHome.refresh(true);
		expect(api.homeWide).toHaveBeenCalledWith(true);
		expect(wideHome.error).toBe('GitLab is down');
	});

	it('says why Calls could not be read', async () => {
		vi.mocked(invoke).mockRejectedValue('no folder');
		await wideHome.refresh(false);
		await vi.waitFor(() => expect(wideHome.callsError).toBe('no folder'));
	});

	it('follows the watcher clock and lets go', () => {
		const stop = wideHome.follow(2, () => true);
		expect(typeof stop).toBe('function');
		stop();
	});
});
