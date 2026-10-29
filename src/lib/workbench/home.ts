// SPDX-License-Identifier: MIT

/**
 * Home across followed repositories (2A, FEAT-183), pure: which rows each
 * card shows, the *<repo> only* filter, and Needs you.
 */

import type { Collection, HistoryEntry, Item, Stored } from '$lib/calls/types';
import type { Need } from '$lib/home/needs';
import { NEED_KINDS, type NeedKind } from '$lib/home/needs';
import type { Followed, InProject, ProjectError, WideHome } from './types';

export const EMPTY_HOME: WideHome = {
	requests: [],
	pipelines: [],
	issues: [],
	releases: [],
	errors: [],
	read: {},
	readWide: 0
};

/**
 * Only `projectId`'s rows, for *<repo> only*. Null is every row. Errors
 * across the host stay: they are about every project.
 */
export function onlyProject(home: WideHome, projectId: number | null): WideHome {
	if (projectId === null) return home;
	const mine = <T>(rows: InProject<T>[]) => rows.filter((row) => row.projectId === projectId);
	return {
		...home,
		requests: mine(home.requests),
		pipelines: mine(home.pipelines),
		issues: mine(home.issues),
		releases: mine(home.releases),
		errors: home.errors.filter((e) => e.projectId === projectId || e.projectId === 0)
	};
}

/** Rows in followed projects only: the Merge requests and Issues cards. */
export function inFollowed<T>(rows: InProject<T>[], followed: Followed[]): InProject<T>[] {
	const ids = new Set(followed.map((f) => f.id));
	return rows.filter((row) => ids.has(row.projectId));
}

export function errorsOf(home: WideHome, card: ProjectError['card']): ProjectError[] {
	return home.errors.filter((e) => e.card === card);
}

const FAILED = ['failed', 'failure'];

/** A run's project chip: the path's last part, as the designs show it. */
export function shortName(path: string): string {
	return path.split('/').pop() || path;
}

/**
 * Needs you across the host (proposal § 15.3): review requests anywhere,
 * a failed latest run in a followed project, and issues assigned to the
 * person. Every row names its project.
 */
export function wideNeeds(home: WideHome): Need[] {
	const rows: Need[] = [];
	for (const { project, item: mr } of home.requests) {
		if (!mr.reviewRequested || mr.mine || mr.draft) continue;
		rows.push({
			key: `mr:${mr.id}`,
			kind: 'reviewRequested',
			title: `!${mr.iid} ${mr.title}`,
			meta: `${mr.author} asked you`,
			since: mr.updated * 1000,
			project: shortName(project),
			source: { kind: 'link', url: mr.url }
		});
	}
	for (const { projectId, project, item: run } of home.pipelines) {
		if (run.state !== 'completed' || !FAILED.includes(run.result)) continue;
		rows.push({
			key: `run:${projectId}:${run.branch}`,
			kind: 'runFailed',
			title: run.branch,
			meta: `${run.name} failed`,
			since: (run.ended || run.started) * 1000,
			project: shortName(project),
			source: { kind: 'run', run }
		});
	}
	for (const { project, item: issue } of home.issues) {
		rows.push({
			key: `issue:${issue.id}`,
			kind: 'assigned',
			title: `#${issue.number} ${issue.title}`,
			meta: ['Issue', ...issue.labels.slice(0, 2).map((l) => l.name)].join(' · '),
			since: Date.parse(issue.updated) || Date.parse(issue.created) || 0,
			project: shortName(project),
			source: { kind: 'issue', issue }
		});
	}
	const rank = (kind: NeedKind) => NEED_KINDS.indexOf(kind);
	return rows.sort((a, b) => rank(a.kind) - rank(b.kind) || a.since - b.since);
}

/** A Calls collection, as the Calls card shows it. */
export interface RecentCollection {
	stem: string;
	name: string;
	requests: number;
	/** Milliseconds, or null when it has never been sent from. */
	used: number | null;
	environment: string | null;
}

function count(items: Item[]): number {
	return items.reduce((n, item) => n + ('folder' in item ? count(item.items) : 1), 0);
}

/** The `n` Workbench-wide collections used most recently, then the rest by name. */
export function recentCollections(
	collections: Stored<Collection>[],
	history: HistoryEntry[],
	n = 3
): RecentCollection[] {
	const last = new Map<string, HistoryEntry>();
	for (const entry of history) {
		if (!entry.collection) continue;
		const seen = last.get(entry.collection);
		if (!seen || entry.at > seen.at) last.set(entry.collection, entry);
	}
	return collections
		.filter((stored) => stored.value)
		.map((stored) => {
			const used = last.get(stored.stem) ?? null;
			return {
				stem: stored.stem,
				name: stored.value!.name,
				requests: count(stored.value!.items),
				used: used ? used.at : null,
				environment: used?.environment ?? null
			};
		})
		.sort((a, b) => (b.used ?? -1) - (a.used ?? -1) || a.name.localeCompare(b.name))
		.slice(0, n);
}
