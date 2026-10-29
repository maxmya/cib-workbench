// SPDX-License-Identifier: MIT

import { afterEach, describe, expect, it, vi } from 'vitest';
import { ProjectList, SEARCH_DEBOUNCE_MS } from './projects.svelte';
import { followedOf } from './state.svelte';
import type { Project, ProjectPage } from './types';

function project(id: number, path = `group/p${id}`): Project {
	return { id, path, name: path, description: '', lastActivity: 0, defaultBranch: 'main', httpUrl: `https://h/${path}.git`, webUrl: '' };
}

function page(ids: number[], n = 1, more = false): ProjectPage {
	return { projects: ids.map((id) => project(id)), page: n, more };
}

afterEach(() => vi.useRealTimers());

describe('the projects a person may follow (FEAT-182)', () => {
	it('paints the saved page first, then what GitLab says', async () => {
		let release!: (value: ProjectPage) => void;
		const read = vi.fn(() => new Promise<ProjectPage>((resolve) => (release = resolve)));
		const list = new ProjectList(read, vi.fn(async () => page([1, 2])));

		const done = list.reload();
		await vi.waitFor(() => expect(list.projects.map((p) => p.id)).toEqual([1, 2]));
		release(page([3]));
		await done;

		expect(list.projects.map((p) => p.id)).toEqual([3]);
		expect(list.loading).toBe(false);
	});

	it('pages on, and searches only once the typing stops', async () => {
		vi.useFakeTimers();
		const read = vi.fn(async (search: string | null, n: number) => (n === 1 ? page([1, 2], 1, true) : page([2, 3], 2)));
		const list = new ProjectList(read, vi.fn(async () => null));

		list.search('c');
		list.search('cr');
		list.search('crx');
		expect(read).not.toHaveBeenCalled();
		await vi.advanceTimersByTimeAsync(SEARCH_DEBOUNCE_MS);
		expect(read).toHaveBeenCalledTimes(1);
		expect(read).toHaveBeenLastCalledWith('crx', 1);

		await list.next();
		expect(read).toHaveBeenLastCalledWith('crx', 2);
		expect(list.projects.map((p) => p.id)).toEqual([1, 2, 3]);
		expect(list.more).toBe(false);
	});

	it('says what went wrong and keeps what it had', async () => {
		const list = new ProjectList(vi.fn(async () => Promise.reject('offline')), vi.fn(async () => page([5])));
		await list.reload();
		expect(list.error).toBe('offline');
		expect(list.projects.map((p) => p.id)).toEqual([5]);
	});

	it('follows a project by id, on CIB GitLab, with its path and default branch', () => {
		expect(followedOf(project(7, 'payments/crx-transfers'))).toEqual({
			host: 'gitlab.apps.ocp-nonprod-01.hodomain.local',
			id: 7,
			path: 'payments/crx-transfers',
			defaultBranch: 'main',
			httpUrl: 'https://h/payments/crx-transfers.git'
		});
	});
});
