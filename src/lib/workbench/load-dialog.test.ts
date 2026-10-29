// SPDX-License-Identifier: MIT

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { click, flushSync, press, render } from '../../testing/mount';

vi.mock('$app/navigation', () => ({ goto: vi.fn(async () => {}) }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/home') } }));
vi.mock('$lib/graph/store.svelte', async () => await import('../../testing/graph-store.svelte'));
vi.mock('$lib/repo.svelte', async () => await import('../../testing/repo-store.svelte'));

vi.mock('$lib/api', () => ({
	inTauri: vi.fn(() => true),
	recentRepos: vi.fn(async () => [
		{ path: 'C:\\Projects\\crx-transfers', name: 'crx-transfers', present: true, bare: false, branch: 'main' },
		{ path: 'C:\\Work\\notes', name: 'notes', present: true, bare: false, branch: 'trunk' },
		{ path: 'C:\\Gone', name: 'Gone', present: false, bare: false, branch: null }
	]),
	clonePlan: vi.fn(),
	cloneStart: vi.fn(),
	cloneRelease: vi.fn()
}));

vi.mock('./api', () => ({
	gitlabLocalClones: vi.fn(async () => [{ path: 'C:\\Projects\\crx-transfers', project: 'payments/crx-transfers' }]),
	gitlabProjects: vi.fn(async () => ({
		projects: [
			{ id: 7, path: 'payments/crx-transfers', name: 'crx-transfers', description: '', lastActivity: 0, defaultBranch: 'main', httpUrl: 'https://g/7.git', webUrl: '' },
			{ id: 8, path: 'core/customer-profile', name: 'customer-profile', description: 'Profiles', lastActivity: 0, defaultBranch: 'main', httpUrl: 'https://g/8.git', webUrl: '' }
		],
		page: 1,
		more: false
	})),
	gitlabProjectsCached: vi.fn(async () => null),
	setCloneInto: vi.fn()
}));

import { calls as repoCalls, control as repoControl } from '../../testing/repo-store.svelte';
import LoadDialog from './LoadDialog.svelte';
import { loader } from './loader.svelte';
import { workbench } from './state.svelte';

async function settle() {
	for (let i = 0; i < 8; i += 1) await new Promise((resolve) => setTimeout(resolve, 0));
	flushSync();
}

beforeEach(() => {
	vi.clearAllMocks();
	repoControl.reset();
	loader.clear();
	workbench.clear();
	workbench.present({ login: 'n.hassan', followed: [{ host: 'h', id: 9, path: 'cards/card-issuing-api', defaultBranch: 'main', httpUrl: 'https://g/9.git' }] });
});

describe('Load repository (2E, FEAT-184)', () => {
	it('lists this machine and CIB GitLab, a cloned project once, followed first', async () => {
		loader.show();
		const view = render(LoadDialog, {});
		await settle();

		expect(view.get('[role="dialog"]').getAttribute('aria-modal')).toBe('true');
		const titles = view.all('.row .title .mono').map((t) => t.textContent);
		expect(titles).toEqual(['payments/crx-transfers', 'notes', 'cards/card-issuing-api', 'core/customer-profile']);
		expect(view.all('.group-title').map((g) => g.textContent)).toEqual(['On this machine', 'On CIB GitLab']);
		expect(view.text()).toContain('Clones into C:\\Projects\\card-issuing-api');
		expect(view.text()).toContain('Followed');
		view.destroy();
	});

	it('moves with the arrows, loads with Enter, and closes with Esc', async () => {
		loader.show();
		const view = render(LoadDialog, {});
		await settle();
		const dialog = view.get('[role="dialog"]');

		press(dialog, 'ArrowDown');
		flushSync();
		expect(view.get('.row[data-active="true"] .title').textContent).toContain('notes');
		press(dialog, 'Enter');
		await settle();
		expect(repoCalls.opened).toEqual(['C:\\Work\\notes']);
		expect(loader.open).toBe(false);
		view.destroy();

		loader.show();
		const again = render(LoadDialog, {});
		await settle();
		press(again.get('[role="dialog"]'), 'Escape');
		expect(loader.open).toBe(false);
		again.destroy();
	});

	it('narrows to what the search matches', async () => {
		loader.show();
		const view = render(LoadDialog, {});
		await settle();
		const search = view.get('#load-search') as HTMLInputElement;
		search.value = 'note';
		search.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();

		expect(view.all('.row .title .mono').map((t) => t.textContent)).toEqual(['notes']);
		expect(view.text()).toContain('Nothing matches “note”.');
		view.destroy();
	});

	it('offers this machine only when nothing is connected', async () => {
		workbench.present({ login: null });
		loader.show();
		const view = render(LoadDialog, {});
		await settle();
		expect(view.all('.group-title').map((g) => g.textContent)).toEqual(['On this machine']);
		click(view.all('.row')[0]);
		await settle();
		expect(repoCalls.opened).toEqual(['C:\\Projects\\crx-transfers']);
		view.destroy();
	});
});
