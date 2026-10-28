// SPDX-License-Identifier: MIT

/**
 * Calls › Import's state (FEAT-162, proposal §8).
 *
 * A scan runs in the backend on a thread of its own and says how far it
 * has got; what it found is kept there too, so *Bring in* names endpoints
 * by id and the backend builds them with the choices made here. Files from
 * outside and a pasted cURL are added to what the scan found.
 *
 * *New* and *Changed* endpoints are ticked when they arrive; ones already
 * in a collection are not (§8.1).
 */

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { describe, notice } from '$lib/ui/notice.svelte';
import * as api from './api';
import type { Scope } from './api';
import { calls } from './store.svelte';
import type { Askable, BroughtIn, Endpoint, Found, ScanDone, ScanProgress, SourceKind } from './types';

export type Filter = 'all' | 'new' | 'changed' | 'in';

/** Sources whose endpoints are declared by code rather than held in a file. */
export const CODE_KINDS: SourceKind[] = ['spring', 'jaxRs', 'nest', 'jaxWs', 'springWs', 'wsdl', 'openApi'];

/** Controllers an agent can be asked to read (FEAT-179). */
export const AGENT_KINDS: SourceKind[] = ['spring', 'jaxRs', 'nest'];

export interface BringChoices {
	/** Into an existing collection, or a new one. */
	as: 'into' | 'new';
	into: string | null;
	name: string;
	folders: 'perGroup' | 'byPath';
	bodies: boolean;
	pathVars: boolean;
	serverAddress: boolean;
}

let found = $state<Found | null>(null);
let scanning = $state<{ id: string; files: number } | null>(null);
let error = $state<string | null>(null);
let selected = $state<string[]>([]);
let filter = $state<Filter>('all');
let source = $state<string | null>(null);
let bringing = $state(false);
let choices = $state<BringChoices>({
	as: 'into',
	into: null,
	name: '',
	folders: 'perGroup',
	bodies: true,
	pathVars: true,
	serverAddress: true
});
/** The Calls card's question was answered for this repository. */
let dismissed = $state(false);

let repoPath: string | null = null;
/** The scope and repository what is found belongs to (FEAT-170). */
let place: string | null = null;
let importScope = $state<Scope>('repository');
/**
 * What is being read right now, said on screen (BUG-105): a file, a folder,
 * a WSDL or a pasted command. Reading a large collection took seconds and
 * nothing moved, so it looked as if the click had done nothing.
 */
let reading = $state<string | null>(null);
/** What the last read found, said until the next one: how many calls, from where. */
let lastRead = $state<string | null>(null);
/** The agents that may read the code, and the one chosen (FEAT-179). */
let agents = $state<Askable[]>([]);
let agent = $state<string | null>(null);
/** An agent is reading the chosen controllers. */
let agentReading = $state(false);
/** What its last read added, said until the next. */
let agentSaid = $state<string | null>(null);
let listening: Promise<void> | null = null;
let unlisten: UnlistenFn[] = [];
/** Scanned once on arrival when Settings says *Always*. */
let autoScanned = new Set<string>();

const wanted = (endpoint: Endpoint) => endpoint.status.kind === 'new' || endpoint.status.kind === 'changed';

function take(next: Found | null, from: string | null = null): void {
	const before = new Set(found?.endpoints.map((endpoint) => endpoint.id) ?? []);
	found = next;
	const ids = new Set(next?.endpoints.map((endpoint) => endpoint.id) ?? []);
	// What is still there stays as chosen; what arrived is ticked by the rule.
	const kept = selected.filter((id) => ids.has(id));
	const arrived = (next?.endpoints ?? [])
		.filter((endpoint) => !before.has(endpoint.id) || (from !== null && endpoint.source === from))
		.filter(wanted)
		.map((endpoint) => endpoint.id);
	selected = [...new Set([...kept, ...arrived])];
	if (choices.into === null || !calls.collections.some((stored) => stored.stem === choices.into)) {
		const counts = new Map<string, number>();
		for (const endpoint of next?.endpoints ?? []) {
			if (endpoint.status.kind !== 'new') counts.set(endpoint.status.collection, (counts.get(endpoint.status.collection) ?? 0) + 1);
		}
		const best = [...counts].sort((a, b) => b[1] - a[1])[0]?.[0];
		const first = calls.collections.find((stored) => stored.value)?.stem ?? null;
		choices.into = best ?? first;
		if (!choices.into) choices.as = 'new';
	}
	if (!choices.name) {
		const name = (importScope === 'repository' ? repoPath?.split(/[\\/]/).filter(Boolean).pop() : null) ?? 'calls';
		choices.name = name.toLowerCase().replace(/[^a-z0-9._-]+/g, '-') || 'calls';
	}
}

async function listenOnce(): Promise<void> {
	listening ??= (async () => {
		unlisten = [
			await listen<ScanProgress>(api.SCAN_EVENT, (event) => {
				if (scanning && event.payload.scanId === scanning.id) scanning = { ...scanning, files: event.payload.files };
			}),
			await listen<ScanDone>(api.SCAN_DONE_EVENT, (event) => {
				const done = event.payload;
				if (!scanning || done.scanId !== scanning.id) return;
				scanning = null;
				if (done.cancelled) return;
				if (done.error) {
					error = done.error;
					return;
				}
				error = null;
				take(done.found);
			})
		];
	})();
	await listening;
}

export const importer = {
	get found(): Found | null {
		return found;
	},
	get scanning(): { id: string; files: number } | null {
		return scanning;
	},
	get error(): string | null {
		return error;
	},
	get selected(): string[] {
		return selected;
	},
	get filter(): Filter {
		return filter;
	},
	get source(): string | null {
		return source;
	},
	get choices(): BringChoices {
		return choices;
	},
	get bringing(): boolean {
		return bringing;
	},
	get dismissed(): boolean {
		return dismissed;
	},
	/** Where what is brought in goes (FEAT-170). */
	get scope(): Scope {
		return importScope;
	},
	/** What is being read now, or null (BUG-105). */
	get reading(): string | null {
		return reading;
	},
	/** What the last read found. */
	get lastRead(): string | null {
		return lastRead;
	},
	get agents(): Askable[] {
		return agents;
	},
	get agent(): string | null {
		return agent;
	},
	get agentReading(): boolean {
		return agentReading;
	},
	get agentSaid(): string | null {
		return agentSaid;
	},

	/** The chosen endpoints an agent could read: ones a controller declares. */
	get forAgent(): Endpoint[] {
		const code = new Set((found?.sources ?? []).filter((s) => AGENT_KINDS.includes(s.kind)).map((s) => s.id));
		return (found?.endpoints ?? []).filter((endpoint) => selected.includes(endpoint.id) && code.has(endpoint.source) && endpoint.file);
	},

	/** The endpoints the filters leave. */
	get visible(): Endpoint[] {
		return (found?.endpoints ?? []).filter(
			(endpoint) => (source === null || endpoint.source === source) && (filter === 'all' || endpoint.status.kind === filter)
		);
	},

	/** Endpoints code declares that no collection has: what the Calls card counts. */
	get notInACollection(): Endpoint[] {
		const code = new Set(
			(found?.sources ?? []).filter((s) => CODE_KINDS.includes(s.kind)).map((s) => s.id)
		);
		return (found?.endpoints ?? []).filter((endpoint) => code.has(endpoint.source) && endpoint.status.kind === 'new');
	},

	counts(): Record<Filter, number> {
		const all = (found?.endpoints ?? []).filter((endpoint) => source === null || endpoint.source === source);
		return {
			all: all.length,
			new: all.filter((e) => e.status.kind === 'new').length,
			changed: all.filter((e) => e.status.kind === 'changed').length,
			in: all.filter((e) => e.status.kind === 'in').length
		};
	},

	/** What this repository's last scan found; a scan when Settings says *Always*. */
	async load(path: string | null, scope: Scope = 'repository'): Promise<void> {
		// What was found is kept per place: a file read into Workbench-wide
		// calls is not offered in a repository's, nor the other way round.
		const key = scope === 'global' ? 'global' : `repository\u0000${path}`;
		if (key !== place) {
			place = key;
			repoPath = path;
			importScope = scope;
			found = null;
			selected = [];
			source = null;
			filter = 'all';
			error = null;
			dismissed = false;
			choices = { ...choices, into: null, name: '' };
		}
		await listenOnce();
		try {
			take(await api.found());
		} catch (cause) {
			error = describe(cause);
		}
		// Only a repository has code to scan.
		if (path && scope === 'repository' && !found && calls.settings?.scanOnOpen === 'always' && !autoScanned.has(path)) {
			autoScanned.add(path);
			await this.scan();
		}
	},

	async scan(): Promise<void> {
		if (scanning) return;
		const id = `scan_${Date.now().toString(36)}`;
		scanning = { id, files: 0 };
		error = null;
		dismissed = true;
		try {
			await listenOnce();
			await api.scan(id);
		} catch (cause) {
			scanning = null;
			error = describe(cause);
		}
	},

	async cancel(): Promise<void> {
		if (!scanning) return;
		const id = scanning.id;
		scanning = null;
		await api.cancelScan(id).catch(() => {});
	},

	dismiss(): void {
		dismissed = true;
	},

	async importFile(path: string): Promise<boolean> {
		const name = path.split(/[\\/]/).filter(Boolean).pop() ?? path;
		return this.read(`Reading ${name}…`, () => api.importFile(path), 'Could not read that file');
	},

	/** A WSDL from a URL the person typed (FEAT-163). */
	async importWsdlUrl(url: string): Promise<boolean> {
		return this.read('Fetching the WSDL…', () => api.importWsdlUrl(url), 'Could not read that WSDL');
	},

	async importText(text: string): Promise<boolean> {
		return this.read('Reading the cURL command…', () => api.importText(text), 'Could not read that cURL command');
	},

	/**
	 * Read one source, saying so while it is read and what it found after
	 * (BUG-105). One at a time: a second click while one reads is ignored.
	 */
	async read(label: string, ask: () => Promise<Found>, failure: string): Promise<boolean> {
		if (reading) return false;
		reading = label;
		lastRead = null;
		try {
			const next = await ask();
			const added = next.sources.at(-1) ?? null;
			take(next, added?.id ?? null);
			source = added?.id ?? null;
			if (added) {
				const count = next.endpoints.filter((endpoint) => endpoint.source === added.id).length;
				lastRead =
					count === 0
						? `Nothing to bring in from ${added.label}.`
						: `${count} ${count === 1 ? 'call' : 'calls'} read from ${added.label}. Choose where they go, then bring them in.`;
			}
			return true;
		} catch (cause) {
			notice.failed(failure, cause);
			return false;
		} finally {
			reading = null;
		}
	},

	setFilter(next: Filter): void {
		filter = next;
	},

	setSource(next: string | null): void {
		source = next;
	},

	toggle(id: string): void {
		selected = selected.includes(id) ? selected.filter((other) => other !== id) : [...selected, id];
	},

	/** Tick or untick every endpoint in `ids` together. */
	toggleAll(ids: string[]): void {
		const all = ids.every((id) => selected.includes(id));
		selected = all ? selected.filter((id) => !ids.includes(id)) : [...new Set([...selected, ...ids])];
	},

	/** The agents the repository allows; the first is chosen. */
	async loadAgents(): Promise<void> {
		try {
			agents = await api.agents();
		} catch {
			agents = [];
		}
		if (!agents.some((known) => known.id === agent)) agent = agents[0]?.id ?? null;
	},

	chooseAgent(id: string | null): void {
		agent = id;
	},

	/**
	 * *Ask an agent too* (FEAT-179): it reads the chosen endpoints'
	 * controllers, and what it says of their bodies and answers is added to
	 * what the scan read.
	 */
	async askAgent(): Promise<boolean> {
		const ids = this.forAgent.map((endpoint) => endpoint.id);
		if (!ids.length || agentReading) return false;
		agentReading = true;
		agentSaid = null;
		try {
			const read = await api.agentRead(ids, agent);
			take(read.found);
			const files = `${read.files} ${read.files === 1 ? 'controller' : 'controllers'}`;
			const added = read.added === 0 ? 'found nothing more to add' : `added ${read.added} ${read.added === 1 ? 'rule or answer' : 'rules and answers'}`;
			const failed = read.failed.length ? ` ${read.failed.length} could not be read: ${read.failed.join('; ')}` : '';
			agentSaid = `${read.agent} read ${files} and ${added}.${failed}`;
			return true;
		} catch (cause) {
			notice.failed('The agent could not read them', cause);
			return false;
		} finally {
			agentReading = false;
		}
	},

	choose(change: Partial<BringChoices>): void {
		choices = { ...choices, ...change };
	},

	/** Bring the ticked endpoints in, then read the collections again. */
	async bringIn(): Promise<BroughtIn | null> {
		if (!selected.length || bringing) return null;
		bringing = true;
		try {
			const done = await api.bringIn({
				ids: selected,
				into: choices.as === 'into' ? choices.into : null,
				name: choices.as === 'new' ? choices.name : null,
				choices: { folders: choices.folders, bodies: choices.bodies, pathVars: choices.pathVars },
				serverAddress: choices.serverAddress
			});
			const parts = [
				done.brought.added ? `${done.brought.added} added` : '',
				done.brought.changed ? `${done.brought.changed} updated` : '',
				done.environments ? `${done.environments} environment${done.environments === 1 ? '' : 's'} made` : ''
			].filter(Boolean);
			notice.ok(`Brought into ${done.stem}: ${parts.join(', ') || 'nothing new'}`);
			lastRead = null;
			await calls.reload();
			selected = [];
			take(await api.found());
			choices = { ...choices, as: 'into', into: done.stem };
			return done;
		} catch (cause) {
			notice.failed('Could not bring them in', cause);
			return null;
		} finally {
			bringing = false;
		}
	},

	/** For tests. */
	reset(): void {
		for (const stop of unlisten) stop();
		unlisten = [];
		listening = null;
		repoPath = null;
		place = null;
		importScope = 'repository';
		reading = null;
		lastRead = null;
		agents = [];
		agent = null;
		agentReading = false;
		agentSaid = null;
		autoScanned = new Set();
		found = null;
		scanning = null;
		error = null;
		selected = [];
		filter = 'all';
		source = null;
		bringing = false;
		dismissed = false;
		choices = { as: 'into', into: null, name: '', folders: 'perGroup', bodies: true, pathVars: true, serverAddress: true };
	}
};
