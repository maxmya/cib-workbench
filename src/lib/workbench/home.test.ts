// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import type { Collection, HistoryEntry, Stored } from '$lib/calls/types';
import type { Issue } from '$lib/issues/types';
import type { Run } from '$lib/pipelines/types';
import { EMPTY_HOME, inFollowed, onlyProject, recentCollections, wideNeeds } from './home';
import type { Followed, MergeRequest, WideHome } from './types';

function mr(id: number, over: Partial<MergeRequest> = {}): MergeRequest {
	return {
		id, iid: id, title: `MR ${id}`, author: 'omar', updated: 1_000, sourceBranch: 'b', targetBranch: 'main',
		draft: false, mine: false, reviewRequested: true, conflicts: false, url: `https://g/mr/${id}`, ...over
	};
}

function run(over: Partial<Run> = {}): Run {
	return { id: 1, name: 'pipeline', branch: 'main', commit: '', event: 'push', state: 'completed', result: 'failed', url: '', started: 10, ended: 20, duration: 10, ...over };
}

function issue(id: number): Issue {
	return {
		id: String(id), number: id, title: `Issue ${id}`, body: '', state: 'open', author: 'a', created: '2026-10-01T00:00:00Z',
		updated: '2026-10-02T00:00:00Z', url: '', replies: 0, labels: [{ name: 'bug', color: '' } as never], assignees: [],
		milestone: null, locked: false, branchSuggestion: ''
	};
}

const home: WideHome = {
	...EMPTY_HOME,
	requests: [
		{ projectId: 7, project: 'payments/crx-transfers', item: mr(1) },
		{ projectId: 99, project: 'other/thing', item: mr(2) },
		{ projectId: 7, project: 'payments/crx-transfers', item: mr(3, { mine: true, reviewRequested: false }) },
		{ projectId: 7, project: 'payments/crx-transfers', item: mr(4, { draft: true }) }
	],
	pipelines: [
		{ projectId: 7, project: 'payments/crx-transfers', item: run() },
		{ projectId: 9, project: 'cards/card-issuing-api', item: run({ result: 'success' }) }
	],
	issues: [{ projectId: 9, project: 'cards/card-issuing-api', item: issue(88) }],
	releases: [],
	errors: [
		{ projectId: 9, project: 'cards/card-issuing-api', card: 'releases', detail: 'down' },
		{ projectId: 0, project: '', card: 'issues', detail: 'offline' }
	]
};

const followed: Followed[] = [
	{ host: 'h', id: 7, path: 'payments/crx-transfers' },
	{ host: 'h', id: 9, path: 'cards/card-issuing-api' }
];

describe('Home across followed repositories (FEAT-183)', () => {
	it('needs you for reviews anywhere, failed runs and assigned issues, each with its project', () => {
		const rows = wideNeeds(home);
		expect(rows.map((r) => [r.kind, r.title, r.project])).toEqual([
			['reviewRequested', '!1 MR 1', 'crx-transfers'],
			['reviewRequested', '!2 MR 2', 'thing'],
			['runFailed', 'main', 'crx-transfers'],
			['assigned', '#88 Issue 88', 'card-issuing-api']
		]);
		expect(rows[0].source).toEqual({ kind: 'link', url: 'https://g/mr/1' });
	});

	it('lists merge requests and issues of followed projects only', () => {
		expect(inFollowed(home.requests, followed).map((r) => r.item.id)).toEqual([1, 3, 4]);
		expect(inFollowed(home.issues, [])).toEqual([]);
	});

	it('filters every card to the loaded project, keeping errors about the whole host', () => {
		const only = onlyProject(home, 9);
		expect(only.requests).toEqual([]);
		expect(only.pipelines.map((r) => r.projectId)).toEqual([9]);
		expect(only.issues.length).toBe(1);
		expect(only.errors.map((e) => e.projectId)).toEqual([9, 0]);
		expect(onlyProject(home, null)).toBe(home);
	});

	it('shows the three Calls collections used last, then the rest by name', () => {
		const stored = (stem: string, name: string, items = 1): Stored<Collection> => ({
			stem, file: `${stem}.json`, error: null,
			value: { v: 1, name, items: Array.from({ length: items }, () => ({ folder: 'f', items: [{} as never, {} as never] })) }
		});
		const used = (collection: string, at: number, environment: string | null = null) =>
			({ id: `${collection}${at}`, at, collection, environment } as unknown as HistoryEntry);
		const rows = recentCollections(
			[stored('a', 'Alpha'), stored('b', 'Beta', 2), stored('c', 'Core banking'), stored('d', 'Delta')],
			[used('b', 5, 'UAT'), used('c', 9), used('b', 3)]
		);
		expect(rows.map((r) => [r.name, r.requests, r.used, r.environment])).toEqual([
			['Core banking', 2, 9, null],
			['Beta', 4, 5, 'UAT'],
			['Alpha', 2, null, null]
		]);
	});
});
