// SPDX-License-Identifier: MIT

import { beforeEach, describe, expect, it, vi } from 'vitest';

const goto = vi.fn(async (_path: string) => {});
vi.mock('$app/navigation', () => ({ goto: (path: string) => goto(path) }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/home') } }));
vi.mock('$lib/graph/store.svelte', async () => await import('../../testing/graph-store.svelte'));
vi.mock('$lib/repo.svelte', async () => await import('../../testing/repo-store.svelte'));

const { handlers } = vi.hoisted(() => ({ handlers: new Map<string, (event: { payload: unknown }) => void>() }));
vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
		handlers.set(name, handler);
		return () => handlers.delete(name);
	})
}));

vi.mock('$lib/api', () => ({
	inTauri: vi.fn(() => true),
	clonePlan: vi.fn(),
	cloneStart: vi.fn(async () => 5),
	cloneRelease: vi.fn(async () => {})
}));

import * as api from '$lib/api';
import { CLONE_DONE_EVENT, CLONE_PROGRESS_EVENT, type ClonePlan } from '$lib/types';
import { calls as repoCalls, control as repoControl } from '../../testing/repo-store.svelte';
import { loader } from './loader.svelte';
import type { Project } from './types';

const project: Project = {
	id: 7, path: 'payments/crx-transfers', name: 'crx-transfers', description: '', lastActivity: 0,
	defaultBranch: 'main', httpUrl: 'https://g/payments/crx-transfers.git', webUrl: ''
};

function plan(over: Partial<ClonePlan> = {}): ClonePlan {
	return { name: 'crx-transfers', destination: 'C:\\P\\crx-transfers', createsDestination: true, problem: null, message: null, ...over };
}

async function until(check: () => boolean) {
	for (let i = 0; i < 50 && !check(); i += 1) await new Promise((resolve) => setTimeout(resolve, 0));
}

beforeEach(() => {
	vi.clearAllMocks();
	handlers.clear();
	repoControl.reset();
	loader.clear();
});

describe('the Load repository dialog (FEAT-184)', () => {
	it('opens, toggles and remembers where to go after', () => {
		loader.show({ then: '/branches', project: 7 });
		expect([loader.open, loader.then, loader.selected]).toEqual([true, '/branches', 7]);
		loader.toggle();
		expect([loader.open, loader.then, loader.selected]).toEqual([false, null, null]);
		loader.toggle();
		expect(loader.open).toBe(true);
	});

	it('loads, closes, and goes to the screen it was opened for', async () => {
		loader.show({ then: '/branches' });
		expect(await loader.load('/repos/a')).toBe(true);
		expect(repoCalls.opened).toEqual(['/repos/a']);
		expect(goto).toHaveBeenCalledWith('/branches');
		expect(loader.open).toBe(false);
		expect(loader.busy).toBeNull();
	});

	it('stays open and says so when the repository does not open', async () => {
		loader.show();
		repoControl.failNextOpen();
		expect(await loader.load('/repos/gone')).toBe(false);
		expect(loader.open).toBe(true);
		expect(loader.error).toBeTruthy();
	});

	it('clones with progress in the row, then loads the clone', async () => {
		vi.mocked(api.clonePlan).mockResolvedValueOnce(plan());
		loader.show();
		const done = loader.cloneAndLoad(project, 'C:\\P');
		await until(() => handlers.has(CLONE_DONE_EVENT) && vi.mocked(api.cloneStart).mock.calls.length > 0);
		await Promise.resolve();

		handlers.get(CLONE_PROGRESS_EVENT)!({ payload: { token: 5, phase: 'Receiving', percent: 40, line: '' } });
		expect(loader.busy).toEqual({ key: 'gitlab:7', text: 'Cloning…', percent: 40 });
		handlers.get(CLONE_PROGRESS_EVENT)!({ payload: { token: 9, phase: 'x', percent: 99, line: '' } });
		expect(loader.busy?.percent).toBe(40);

		handlers.get(CLONE_DONE_EVENT)!({ payload: { token: 5, ok: true, cancelled: false, error: null, path: 'C:\\P\\crx-transfers' } });
		expect(await done).toBe(true);
		expect(api.cloneStart).toHaveBeenCalledWith('https://g/payments/crx-transfers.git', 'C:\\P');
		expect(api.cloneRelease).toHaveBeenCalled();
		expect(repoCalls.opened).toEqual(['C:\\P\\crx-transfers']);
		expect(loader.open).toBe(false);
	});

	it('says why a clone failed and loads nothing', async () => {
		vi.mocked(api.clonePlan).mockResolvedValueOnce(plan());
		loader.show();
		const done = loader.cloneAndLoad(project, 'C:\\P');
		await until(() => handlers.has(CLONE_DONE_EVENT) && vi.mocked(api.cloneStart).mock.calls.length > 0);
		await Promise.resolve();
		handlers.get(CLONE_DONE_EVENT)!({ payload: { token: 5, ok: false, cancelled: false, error: 'Authentication failed', path: '' } });

		expect(await done).toBe(false);
		expect(loader.error).toBe('Authentication failed');
		expect(repoCalls.opened).toEqual([]);
	});

	it('loads a folder that already holds the clone, rather than cloning again', async () => {
		vi.mocked(api.clonePlan).mockResolvedValueOnce(plan({ problem: { kind: 'destinationNotEmpty' }, message: 'not empty' }));
		expect(await loader.cloneAndLoad(project, 'C:\\P')).toBe(true);
		expect(api.cloneStart).not.toHaveBeenCalled();
		expect(repoCalls.opened).toEqual(['C:\\P\\crx-transfers']);
	});

	it('refuses a folder it cannot clone into, saying why', async () => {
		vi.mocked(api.clonePlan).mockResolvedValueOnce(plan({ problem: { kind: 'missingParent' }, destination: null, message: 'C:\\Nope does not exist' }));
		expect(await loader.cloneAndLoad(project, 'C:\\Nope')).toBe(false);
		expect(loader.error).toBe('C:\\Nope does not exist');
	});

	it('unloads to nothing', async () => {
		repoControl.setInfo({ path: '/repos/a', name: 'a', bare: false, lastFetched: null, head: { branch: 'main', detached: false, id: 'a', short: 'a' } });
		await loader.unload();
		expect(repoCalls.closed).toBe(1);
		expect(loader.unloading).toBe(false);
	});
});
