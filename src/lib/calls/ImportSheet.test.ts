// SPDX-License-Identifier: MIT

/**
 * Calls › Import (FEAT-162, FEAT-178): the sheet's three steps — where
 * the calls come from, which of them, and where they are saved.
 */

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { click, fire, flushSync, render, type Mounted } from '../../testing/mount';
import { control } from '../../testing/repo-store.svelte';
import type { Call, Endpoint, Found, Loaded } from './types';

const { notice, handlers } = vi.hoisted(() => ({
	notice: { ok: vi.fn(), failed: vi.fn() },
	handlers: new Map<string, (event: { payload: unknown }) => void>()
}));

vi.mock('$app/navigation', () => ({ goto: vi.fn() }));
vi.mock('$lib/repo.svelte', async () => await import('../../testing/repo-store.svelte'));
vi.mock('$lib/api', async (importOriginal) => ({
	...(await importOriginal<typeof import('$lib/api')>()),
	inTauri: vi.fn(() => true)
}));
vi.mock('$lib/ui/notice.svelte', () => ({ notice, describe: (error: unknown) => String(error) }));
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn(async () => 'C:/Downloads/shop.postman_collection.json') }));
vi.mock('@tauri-apps/api/event', () => ({
	listen: vi.fn(async (name: string, handler: (event: { payload: unknown }) => void) => {
		handlers.set(name, handler);
		return () => handlers.delete(name);
	})
}));
vi.mock('./api', () => ({
	setScope: vi.fn(),
	currentScope: vi.fn(() => 'repository'),
	CHUNK_EVENT: 'calls-chunk',
	DONE_EVENT: 'calls-done',
	SCAN_EVENT: 'calls-scan',
	SCAN_DONE_EVENT: 'calls-scan-done',
	load: vi.fn(),
	saveState: vi.fn(async () => {}),
	privateOf: vi.fn(async () => ({ kept: [], secrets: [] })),
	foundFiles: vi.fn(),
	found: vi.fn(),
	scan: vi.fn(async () => {}),
	cancelScan: vi.fn(async () => {}),
	importFile: vi.fn(),
	importText: vi.fn(),
	bringIn: vi.fn(),
	agents: vi.fn(async () => []),
	agentRead: vi.fn()
}));

import { open as openDialog } from '@tauri-apps/plugin-dialog';
import * as api from './api';
import ImportSheet from './ImportSheet.svelte';
import { importer } from './import.svelte';
import { calls } from './store.svelte';

const call = (name: string, method: string, url: string): Call => ({ id: `c_${name}`, name, kind: 'rest', method, url });

function endpoint(id: string, method: string, path: string, status: Endpoint['status'], extra: Partial<Endpoint> = {}): Endpoint {
	return {
		id,
		key: `${method} ${path}`,
		source: 'code:src/OrderController.java',
		group: 'OrderController',
		file: 'src/OrderController.java',
		line: 3,
		method,
		path,
		hint: '',
		call: call(id, method, `{{baseUrl}}${path}`),
		status,
		...extra
	};
}

const FOUND: Found = {
	sources: [
		{ id: 'code:src/OrderController.java', kind: 'spring', label: 'OrderController.java', file: 'src/OrderController.java', count: 3, error: null, inRepository: true },
		{ id: 'file:api/orders.http', kind: 'http', label: 'orders.http', file: 'api/orders.http', count: 1, error: null, inRepository: true }
	],
	endpoints: [
		endpoint('e1', 'GET', '/orders/{id}', { kind: 'new' }, { hint: 'id' }),
		endpoint('e2', 'POST', '/orders', {
			kind: 'changed',
			collection: 'orders-api',
			callId: 'c_x',
			what: 'CreateOrderRequest gained couponCode'
		}),
		endpoint('e3', 'DELETE', '/orders/{id}', { kind: 'in', collection: 'orders-api', callId: 'c_y' }),
		endpoint('e4', 'GET', '/health', { kind: 'new' }, { source: 'file:api/orders.http', group: 'orders.http', file: 'api/orders.http' })
	],
	environments: [],
	scripted: 2,
	baseUrl: 'http://localhost:8085',
	filesRead: 40,
	capped: false
};

function loaded(): Loaded {
	return {
		folder: '.spagitty/calls',
		collections: [{ stem: 'orders-api', file: '.spagitty/calls/orders-api.json', error: null, value: { v: 1, name: 'orders-api', items: [] } }],
		environments: [],
		shared: { v: 1, vars: [] },
		sharedError: null,
		state: { active: null, open: [], current: null, collapsed: [], filter: '' },
		settings: null as unknown as Loaded['settings'],
		overridden: [],
		today: 0
	};
}

let view: Mounted | null = null;
const onclose = vi.fn();

async function settle(): Promise<void> {
	for (let i = 0; i < 6; i++) {
		await Promise.resolve();
		await new Promise((done) => setTimeout(done, 0));
	}
	flushSync();
}

async function show(found: Found | null = FOUND, start: 1 | 2 | 3 = found ? 2 : 1): Promise<Mounted> {
	vi.mocked(api.load).mockResolvedValue(loaded());
	vi.mocked(api.found).mockResolvedValue(found);
	await calls.load('/repos/orders-service', 'repository');
	await importer.load('/repos/orders-service', 'repository');
	view = render(ImportSheet, { start, scope: 'repository' as const, onscope: vi.fn(), onclose });
	await settle();
	return view;
}

const rows = () =>
	view!.all('.endpoint').map((row) => ({
		text: row.querySelector('.path')?.textContent,
		tag: row.querySelector('.tag')?.textContent,
		ticked: (row.querySelector('input') as HTMLInputElement).checked
	}));
const button = (text: string) =>
	view!.all('button').find((found) => found.textContent?.replace(/\s+/g, ' ').trim().startsWith(text)) as HTMLButtonElement;
const step = () => view!.get('.steps [aria-current="step"]').textContent?.replace(/\d/g, '').trim();

beforeEach(() => {
	vi.clearAllMocks();
	handlers.clear();
	calls.reset();
	importer.reset();
	control.reset();
	control.setInfo({
		path: '/repos/orders-service',
		name: 'orders-service',
		bare: false,
		lastFetched: null,
		head: { branch: 'main', detached: false, id: 'a'.repeat(40), short: 'aaaaaaa' }
	});
});

afterEach(() => {
	view?.destroy();
	view = null;
});

/** FEAT-178: Import is three steps in a sheet over Calls, each with a way back. */
describe('the Import sheet', () => {
	it('picks from the endpoints found, ticking New and Changed, and says what Import will do', async () => {
		await show();
		expect(step()).toBe('Pick');
		expect(rows()).toEqual([
			{ text: '/orders/{id}', tag: 'New', ticked: true },
			{ text: '/orders', tag: 'Changed', ticked: true },
			{ text: '/orders/{id}', tag: 'In orders-api', ticked: false },
			{ text: '/health', tag: 'New', ticked: true }
		]);
		// A changed one says what changed.
		expect(view!.all('.endpoint')[1].textContent).toContain('CreateOrderRequest gained couponCode');
		click(button('Next'));
		expect(step()).toBe('Save');
		expect(button('Import 3 calls into orders-api')).toBeTruthy();
		click(button('Back'));
		expect(step()).toBe('Pick');
	});

	/** FEAT-177: a controller says which service it is in. */
	it('heads each service’s controllers with its name and what it is built as', async () => {
		await show({
			...FOUND,
			endpoints: [
				endpoint('e1', 'GET', '/invoices', { kind: 'new' }, { group: 'InvoiceController', file: 'billing/src/invoice.controller.ts', source: 'code:billing' }),
				endpoint('e2', 'GET', '/orders', { kind: 'new' }, { file: 'services/orders/src/OrderController.java' })
			],
			services: [
				{ name: 'orders-service', kind: 'Spring Boot', root: 'services/orders' },
				{ name: 'billing-api', kind: 'NestJS', root: 'billing' }
			]
		});
		const heads = view!.all('.list .service').map((head) => head.textContent?.replace(/\s+/g, ' ').trim());
		// In the order the services were found, each above its own controllers.
		expect(heads).toEqual(['orders-service Spring Boot services/orders', 'billing-api NestJS billing']);
		expect(view!.all('.group-head .name').map((name) => name.textContent)).toEqual(['OrderController', 'InvoiceController']);
	});

	it('filters by status and by source', async () => {
		await show();
		click(button('Changed'));
		expect(rows().map((row) => row.text)).toEqual(['/orders']);
		click(button('All'));
		const source = view!.get('select[aria-label="Source"]') as HTMLSelectElement;
		source.value = 'file:api/orders.http';
		fire(source, 'change');
		expect(rows().map((row) => row.text)).toEqual(['/health']);
	});

	it('imports the ticked ones with the choices into a new collection, and closes on the collection', async () => {
		await show();
		vi.mocked(api.bringIn).mockResolvedValue({ stem: 'shop', brought: { added: 2, changed: 1, unchanged: 0 }, environments: 1 });
		click(view!.get('input[aria-label="GET /health"]'));
		click(button('Next'));
		const fresh = view!.all('input[type="radio"][name="as"]')[1] as HTMLInputElement;
		fresh.checked = true;
		fire(fresh, 'change');
		const name = view!.get('input[aria-label="The new collection\'s name"]') as HTMLInputElement;
		name.value = 'shop';
		fire(name, 'input');
		const byPath = view!.all('input[type="radio"][name="folders"]')[1] as HTMLInputElement;
		byPath.checked = true;
		fire(byPath, 'change');
		click(button('Import 2 calls into shop'));
		await settle();
		expect(api.bringIn).toHaveBeenCalledWith({
			ids: ['e1', 'e2'],
			into: null,
			name: 'shop',
			choices: { folders: 'byPath', bodies: true, pathVars: true },
			serverAddress: true
		});
		expect(notice.ok).toHaveBeenCalledWith('Brought into shop: 2 added, 1 updated, 1 environment made');
		expect(onclose).toHaveBeenCalledWith('shop');
	});

	it('cancels without writing anything', async () => {
		await show();
		click(button('Cancel'));
		expect(onclose).toHaveBeenCalledWith(null);
		expect(api.bringIn).not.toHaveBeenCalled();
	});

	it('scans, says how far it has got, and moves on to what it found', async () => {
		await show(null);
		expect(step()).toBe('Source');
		expect(button('Next').disabled).toBe(true);
		click(button('Scan this repository’s code'));
		await settle();
		const id = vi.mocked(api.scan).mock.calls[0][0];
		handlers.get('calls-scan')!({ payload: { scanId: id, files: 1200 } });
		flushSync();
		expect(view!.get('.progress').textContent).toBe('Reading files… 1,200');
		handlers.get('calls-scan-done')!({ payload: { scanId: id, found: FOUND, error: null, cancelled: false } });
		await settle();
		expect(step()).toBe('Pick');
		expect(rows()).toHaveLength(4);
	});

	it('reads a file chosen from outside, a Bruno folder and a pasted cURL', async () => {
		await show(FOUND, 1);
		const outside: Found = {
			...FOUND,
			sources: [...FOUND.sources, { id: 'file:C:/Downloads/shop.postman_collection.json', kind: 'postman', label: 'shop.postman_collection.json', file: 'C:/Downloads/shop.postman_collection.json', count: 1, error: null, inRepository: false }],
			endpoints: [
				...FOUND.endpoints,
				endpoint('e5', 'GET', '/ping', { kind: 'new' }, { source: 'file:C:/Downloads/shop.postman_collection.json', group: 'Requests' })
			]
		};
		vi.mocked(api.importFile).mockResolvedValue(outside);
		click(button('A file'));
		await settle();
		expect(api.importFile).toHaveBeenCalledWith('C:/Downloads/shop.postman_collection.json');
		// On to choosing, the file's calls on their own, ticked.
		expect(step()).toBe('Pick');
		expect(rows()).toEqual([{ text: '/ping', tag: 'New', ticked: true }]);

		click(button('Back'));
		// A Bruno collection is chosen as its folder.
		vi.mocked(openDialog).mockResolvedValueOnce('C:/Documents/collections/DXP');
		click(button('A Bruno collection folder'));
		await settle();
		expect(openDialog).toHaveBeenLastCalledWith({ multiple: false, directory: true });
		expect(api.importFile).toHaveBeenLastCalledWith('C:/Documents/collections/DXP');

		click(button('Back'));
		vi.mocked(api.importText).mockRejectedValue('this is not a cURL command');
		click(button('Paste a cURL command'));
		const area = view!.get('textarea[aria-label="A cURL command"]') as HTMLTextAreaElement;
		area.value = 'wget x';
		fire(area, 'input');
		click(button('Read it'));
		await settle();
		expect(notice.failed).toHaveBeenCalledWith('Could not read that cURL command', 'this is not a cURL command');
		expect(step()).toBe('Source');
	});

	/** BUG-105: reading a collection says so while it reads. */
	it('says what it is reading, and keeps the sources still until it is done', async () => {
		await show(FOUND, 1);
		const source = { id: 'file:C:/Downloads/big.json', kind: 'postman' as const, label: 'big.json', file: 'C:/Downloads/big.json', count: 2, error: null, inRepository: false };
		let answer: (found: Found) => void = () => {};
		vi.mocked(api.importFile).mockReturnValue(new Promise<Found>((resolve) => (answer = resolve)));
		vi.mocked(openDialog).mockResolvedValueOnce('C:/Downloads/big.json');

		click(button('A file'));
		await settle();
		expect(view!.text()).toContain('Reading big.json…');
		expect(button('A file').disabled).toBe(true);

		answer({
			...FOUND,
			sources: [...FOUND.sources, source],
			endpoints: [
				...FOUND.endpoints,
				endpoint('b1', 'GET', '/a', { kind: 'new' }, { source: source.id, group: 'Requests' }),
				endpoint('b2', 'GET', '/b', { kind: 'new' }, { source: source.id, group: 'Requests' })
			]
		});
		await settle();
		expect(view!.text()).not.toContain('Reading big.json…');
		expect(step()).toBe('Pick');
		expect(rows().map((row) => row.text)).toEqual(['/a', '/b']);
	});
});

/** FEAT-179: an agent can be asked to read the chosen controllers too. */
describe('asking an agent too', () => {
	it('reads the chosen controllers, and says what it added', async () => {
		vi.mocked(api.agents).mockResolvedValue([{ id: 'claude', name: 'Claude Code' }]);
		const read = {
			...FOUND,
			endpoints: FOUND.endpoints.map((found) =>
				found.id === 'e2' ? { ...found, call: { ...found.call, responses: [{ status: 409, says: 'Already paid', from: 'Claude Code' }] } } : found
			)
		};
		vi.mocked(api.agentRead).mockResolvedValue({ agent: 'Claude Code', files: 1, added: 3, failed: [], found: read });
		await show();
		// e1 and e2 are from the controller; e4, from an .http file, is not for an agent.
		expect(view!.text()).toContain('An agent can read the chosen controller');
		click(button('Ask Claude Code'));
		await settle();
		expect(api.agentRead).toHaveBeenCalledWith(['e1', 'e2'], 'claude');
		expect(view!.text()).toContain('Claude Code read 1 controller and added 3 rules and answers.');
		expect(importer.found?.endpoints.find((found) => found.id === 'e2')?.call.responses?.[0].status).toBe(409);
		expect(rows().filter((row) => row.ticked)).toHaveLength(3);
	});

	it('asks the agent chosen, and says when it added nothing or could not read a file', async () => {
		vi.mocked(api.agents).mockResolvedValue([
			{ id: 'claude', name: 'Claude Code' },
			{ id: 'codex', name: 'Codex' }
		]);
		vi.mocked(api.agentRead).mockResolvedValue({
			agent: 'Codex',
			files: 2,
			added: 0,
			failed: ['src/Other.java: could not be read'],
			found: FOUND
		});
		await show();
		const select = view!.get('select[aria-label="The agent that reads"]') as HTMLSelectElement;
		select.value = 'codex';
		fire(select, 'change');
		flushSync();
		click(button('Ask Codex'));
		await settle();
		expect(api.agentRead).toHaveBeenCalledWith(['e1', 'e2'], 'codex');
		expect(view!.text()).toContain(
			'Codex read 2 controllers and found nothing more to add. 1 could not be read: src/Other.java: could not be read'
		);
	});

	it('says plainly when the agent could not read them', async () => {
		vi.mocked(api.agents).mockResolvedValue([{ id: 'claude', name: 'Claude Code' }]);
		vi.mocked(api.agentRead).mockRejectedValue('This repository does not allow that agent.');
		await show();
		click(button('Ask Claude Code'));
		await settle();
		expect(notice.failed).toHaveBeenCalledWith('The agent could not read them', 'This repository does not allow that agent.');
		expect(button('Next').disabled).toBe(false);
	});

	it('reads no agents when they cannot be listed', async () => {
		vi.mocked(api.agents).mockRejectedValue('no repository');
		await show();
		expect(importer.agents).toEqual([]);
		expect(importer.agent).toBeNull();
	});

	it('says where to set one up when none may be asked', async () => {
		vi.mocked(api.agents).mockResolvedValue([]);
		await show();
		expect(view!.text()).toContain('set one up in Settings › Agents');
		expect(button('Ask')).toBeUndefined();
	});
});
