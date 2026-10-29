// SPDX-License-Identifier: MIT

/**
 * Workbench's chrome with `FEATURES.workbenchWide` on (FEAT-180, FEAT-184).
 * Spagitty's — tabs, a rail that opens a folder — is `chrome/chrome.test.ts`,
 * run with the switch off.
 */

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { click, flushSync, render } from '../../testing/mount';
import type { RepoInfo } from '$lib/types';

const goto = vi.fn();
vi.mock('$app/navigation', () => ({ goto: (path: string) => goto(path) }));

const { page } = vi.hoisted(() => ({ page: { url: new URL('http://localhost/home') } }));
vi.mock('$app/state', () => ({ page }));
vi.mock('$lib/graph/store.svelte', async () => await import('../../testing/graph-store.svelte'));
vi.mock('$lib/repo.svelte', async () => await import('../../testing/repo-store.svelte'));
vi.mock('$lib/branches/store.svelte', async () => await import('../../testing/branches-store.svelte'));

const { appWindow } = vi.hoisted(() => ({
	appWindow: {
		close: vi.fn(),
		minimize: vi.fn(),
		toggleMaximize: vi.fn(),
		isMaximized: vi.fn(() => Promise.resolve(false)),
		startDragging: vi.fn(),
		startResize: vi.fn()
	}
}));
vi.mock('$lib/chrome/window', () => ({ appWindow }));

import { FEATURES, PRODUCT_LONG } from '$lib/brand';
import NavRail from '$lib/chrome/NavRail.svelte';
import TitleBar from '$lib/chrome/TitleBar.svelte';
import { workspace } from '$lib/workspace.svelte';
import { control as repoControl } from '../../testing/repo-store.svelte';
import { loader } from './loader.svelte';

function info(path = '/repos/crx-transfers'): RepoInfo {
	return {
		path,
		name: path.split('/').pop() ?? '',
		bare: false,
		lastFetched: null,
		head: { branch: 'main', detached: false, id: 'a'.repeat(40), short: 'aaaaaaa' }
	};
}

beforeEach(() => {
	goto.mockClear();
	repoControl.reset();
	loader.clear();
	workspace.clear();
	page.url = new URL('http://localhost/home');
});

describe('the switch', () => {
	it('is on in this build', () => {
		expect(FEATURES.workbenchWide).toBe(true);
	});
});

describe('the title bar slot (FEAT-184)', () => {
	it('offers Load repository when nothing is loaded, and no tabs', () => {
		workspace.opened('/repos/a');
		workspace.rest();
		const view = render(TitleBar, {});

		expect(view.all('.tab')).toHaveLength(0);
		const slot = view.get('.slot button.empty');
		expect(slot.textContent).toMatch(/No repository loaded\s*Load repository/);
		click(slot);
		expect(loader.open).toBe(true);
		view.destroy();
	});

	it('names the loaded repository and its branch, with Switch and Unload', () => {
		repoControl.setInfo(info());
		const view = render(TitleBar, {});

		expect(view.get('.repo-name').textContent).toBe('crx-transfers');
		expect(view.get('.branch').textContent).toBe('main');
		expect(view.get('.unload').getAttribute('aria-label')).toBe('Unload crx-transfers');
		click(view.get('.switch'));
		expect(loader.open).toBe(true);
		expect(view.get('.name').getAttribute('aria-label')).toBe(PRODUCT_LONG);
		view.destroy();
	});

	it('is bare while onboarding: the window buttons and nothing else (FEAT-180)', () => {
		const view = render(TitleBar, { bare: true });

		expect(view.find('.slot')).toBeNull();
		expect(view.find('.name')).toBeNull();
		expect(view.find('.bell')).toBeNull();
		view.destroy();
	});
});

describe('the rail with nothing loaded (FEAT-184)', () => {
	it('dims the repository screens, keeps them focusable, and says why', () => {
		const view = render(NavRail, {});
		const row = (label: string) => view.all('.item').find((i) => i.getAttribute('aria-label') === label)!;

		const graph = row('Graph');
		expect(graph.getAttribute('aria-disabled')).toBe('true');
		expect(graph.getAttribute('title')).toBe('Graph — load a repository first');
		expect(graph.hasAttribute('disabled')).toBe(false);
		for (const open of ['Home', 'Calls', 'Farm', 'Settings']) {
			expect(row(open).getAttribute('aria-disabled'), open).toBeNull();
		}
		view.destroy();
	});

	it('opens Load repository from a dimmed row, then goes to that screen', () => {
		const view = render(NavRail, {});
		click(view.all('.item').find((i) => i.getAttribute('aria-label') === 'Branches')!);

		expect(goto).not.toHaveBeenCalled();
		expect(loader.open).toBe(true);
		expect(loader.then).toBe('/branches');
		view.destroy();
	});

	it('offers Load repository at the top, and goes straight to a screen once loaded', () => {
		let view = render(NavRail, {});
		click(view.get('.open-repository'));
		expect(loader.open).toBe(true);
		view.destroy();

		repoControl.setInfo(info());
		flushSync();
		view = render(NavRail, {});
		expect(view.find('.open-repository')).toBeNull();
		click(view.all('.item').find((i) => i.getAttribute('aria-label') === 'Branches')!);
		expect(goto).toHaveBeenCalledWith('/branches');
		view.destroy();
	});
});

describe('upgrading from tabs (FEAT-184 § 9.6)', () => {
	it('reads an old workspace without loss: places kept, tabs kept for the switch off', () => {
		const tabs = ['a', 'b', 'c', 'd', 'e'].map((n) => ({ path: `/repos/${n}`, name: n }));
		localStorage.setItem(
			'spagitty.workspace',
			JSON.stringify({ tabs, active: '/repos/c', places: { '/repos/c': { route: '/branches', selected: 'x' } } })
		);
		workspace.init();

		expect(workspace.active).toBe('/repos/c');
		expect(workspace.placeOf('/repos/c')).toEqual({ route: '/branches', selected: 'x' });
		expect(workspace.tabs.map((t) => t.name)).toEqual(['a', 'b', 'c', 'd', 'e']);
		localStorage.removeItem('spagitty.workspace');
	});
});
