// SPDX-License-Identifier: MIT

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { click, render } from '../../testing/mount';

const goto = vi.fn(async (_path: string) => {});
vi.mock('$app/navigation', () => ({ goto: (path: string) => goto(path) }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/home') } }));
vi.mock('$lib/graph/store.svelte', async () => await import('../../testing/graph-store.svelte'));
vi.mock('$lib/repo.svelte', async () => await import('../../testing/repo-store.svelte'));
vi.mock('$lib/api', async (original) => ({ ...(await original<typeof import('$lib/api')>()), inTauri: () => false }));

import type { Run } from '$lib/pipelines/types';
import { control as repoControl } from '../../testing/repo-store.svelte';
import { EMPTY_HOME } from './home';
import { loader } from './loader.svelte';
import { workbench } from './state.svelte';
import type { WideHome as Home } from './types';
import WideHome from './WideHome.svelte';
import { wideHome } from './wide-home.svelte';

const run = (over: Partial<Run>): Run => ({
	id: 1, name: 'pipeline', branch: 'main', commit: '', event: 'push', state: 'completed', result: 'success', url: '', started: 1, ended: 2, duration: 75, ...over
});

const home: Home = {
	...EMPTY_HOME,
	requests: [
		{
			projectId: 7,
			project: 'payments/crx-transfers',
			item: { id: 1, iid: 412, title: 'Retry acks', author: 'omar', updated: 1, sourceBranch: 'b', targetBranch: 'main', draft: false, mine: false, reviewRequested: true, conflicts: false, url: 'https://g/412' }
		}
	],
	pipelines: [
		{ projectId: 7, project: 'payments/crx-transfers', item: run({ result: 'failed' }) },
		{ projectId: 9, project: 'cards/card-issuing-api', item: run({ id: 2 }) }
	],
	errors: [{ projectId: 11, project: 'channels/mobile-gateway', card: 'pipelines', detail: 'GitLab answered 500' }]
};

const followed = [
	{ host: 'h', id: 7, path: 'payments/crx-transfers' },
	{ host: 'h', id: 9, path: 'cards/card-issuing-api' },
	{ host: 'h', id: 11, path: 'channels/mobile-gateway' }
];

beforeEach(() => {
	goto.mockClear();
	repoControl.reset();
	loader.clear();
	workbench.clear();
	wideHome.present({ home: EMPTY_HOME, calls: [] });
});

describe('Home across followed repositories (FEAT-183)', () => {
	it('fills every card with nothing loaded, each row naming its project', () => {
		workbench.present({ login: 'n.hassan', followed });
		wideHome.present({ home, calls: [{ stem: 'core', name: 'Core banking sandbox', requests: 12, used: null, environment: null }] });
		const view = render(WideHome, {});

		expect(view.text()).toContain('Following 3 repositories on CIB GitLab');
		const needs = view.get('[data-card="needs"]').textContent ?? '';
		expect(needs).toContain('!412 Retry acks');
		expect(needs).toContain('crx-transfers');
		expect(view.get('[data-card="requests"] .chip').textContent).toBe('crx-transfers');
		expect(view.all('[data-card="pipelines"] .row')).toHaveLength(2);
		expect(view.get('[data-card="calls"]').textContent).toContain('Core banking sandbox');
		view.destroy();
	});

	it("shows one project's failure in its card, and the others still", () => {
		workbench.present({ login: 'n.hassan', followed });
		wideHome.present({ home });
		const view = render(WideHome, {});

		const pipelines = view.get('[data-card="pipelines"]');
		expect(pipelines.querySelector('.home-error')?.textContent).toBe('mobile-gateway: GitLab answered 500');
		expect(pipelines.querySelectorAll('.row')).toHaveLength(2);
		view.destroy();
	});

	it('opens Load repository with the project chosen for a row whose project is not loaded', () => {
		workbench.present({ login: 'n.hassan', followed });
		wideHome.present({ home });
		const view = render(WideHome, {});

		click(view.get('[data-card="requests"] .row'));
		expect(loader.open).toBe(true);
		expect(loader.selected).toBe(7);
		expect(loader.then).toBe('/requests');
		view.destroy();
	});

	it('says nothing is followed, with the two ways on, and still shows Calls', () => {
		workbench.present({ login: 'n.hassan', followed: [] });
		const view = render(WideHome, {});

		expect(view.text()).toContain("You're not following any repositories");
		expect(view.all('button').map((b) => b.textContent?.trim())).toEqual(
			expect.arrayContaining(['Choose repositories', 'Load a repository'])
		);
		expect(view.find('[data-card="calls"]')).not.toBeNull();
		expect(view.find('[data-card="pipelines"]')).toBeNull();
		view.destroy();
	});

	it('shows only Calls, and the way to connect, when CIB GitLab is not connected', () => {
		workbench.present({ login: null, followed });
		const view = render(WideHome, {});
		expect(view.text()).toContain('Not connected to CIB GitLab');
		expect(view.all('[data-card]').map((c) => c.dataset.card)).toEqual(['calls']);
		view.destroy();
	});

	it('draws the cards in the order chosen', () => {
		workbench.present({ login: 'n.hassan', followed });
		const view = render(WideHome, {});
		expect(view.all('[data-card]').map((c) => c.dataset.card)).toEqual(['needs', 'requests', 'pipelines', 'issues', 'calls', 'releases']);
		view.destroy();
	});
});
