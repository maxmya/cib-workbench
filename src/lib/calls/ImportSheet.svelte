<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { untrack } from 'svelte';
	import { open as openDialog } from '@tauri-apps/plugin-dialog';
	import { PRODUCT_NAME } from '$lib/brand';
	import { repo } from '$lib/repo.svelte';
	import Btn from '$lib/ui/Btn.svelte';
	import EmptyState from '$lib/ui/EmptyState.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import Sheet from '$lib/ui/Sheet.svelte';
	import type { Scope } from './api';
	import { importer, type Filter } from './import.svelte';
	import { methodColour, serviceOf } from './model';
	import ScopeSwitch from './ScopeSwitch.svelte';
	import { calls } from './store.svelte';
	import type { Endpoint, Service, SourceKind } from './types';

	/**
	 * Import, as a sheet over Calls (FEAT-178). Three steps, each with a way
	 * back: where the calls come from, which of them, and where they are
	 * saved. The last button says what it will do — *Import 12 calls into
	 * orders-api* — and nothing is written until it is pressed. Calls stays
	 * underneath, and shows the collection when the sheet closes.
	 *
	 * It was a screen of its own, which replaced Calls, had *Done* and no
	 * way back, and kept its save button in a side panel.
	 */
	interface Props {
		/** The step it opens at: 2 when there are endpoints waiting already. */
		start?: 1 | 2 | 3;
		/** Where it brings calls: the place Calls is showing. */
		scope: Scope;
		onscope: (next: Scope) => void;
		/** Closed: `stem` is the collection the calls went into, when they went. */
		onclose: (stem: string | null) => void;
	}

	let { start = 1, scope, onscope, onclose }: Props = $props();

	const STEPS = ['Source', 'Pick', 'Save'] as const;
	let step = $state<1 | 2 | 3>(untrack(() => start));

	const KIND_LABELS: Record<SourceKind, string> = {
		spring: 'Spring',
		jaxRs: 'JAX-RS',
		nest: 'NestJS',
		jaxWs: 'JAX-WS',
		springWs: 'Spring-WS',
		wsdl: 'WSDL',
		openApi: 'OpenAPI',
		postman: 'Postman',
		insomnia: 'Insomnia',
		bruno: 'Bruno',
		http: '.http',
		har: 'HAR',
		curl: 'cURL',
		spagitty: PRODUCT_NAME
	};
	const FILTERS: { id: Filter; label: string }[] = [
		{ id: 'all', label: 'All' },
		{ id: 'new', label: 'New' },
		{ id: 'changed', label: 'Changed' },
		{ id: 'in', label: 'Already in' }
	];

	const info = $derived(repo.info);
	const found = $derived(importer.found);
	const sources = $derived(found?.sources ?? []);
	const counts = $derived(importer.counts());
	const choices = $derived(importer.choices);
	const readable = $derived(calls.collections.filter((stored) => stored.value));
	const picked = $derived(importer.selected.length);
	const total = $derived(found?.endpoints.length ?? 0);

	let pasting = $state(false);
	let pasted = $state('');
	let wsdlUrl = $state<string | null>(null);

	/** A source read or a scan finished with endpoints: on to choosing them. */
	function onward(read: boolean) {
		if (read && step === 1 && (importer.found?.endpoints.length ?? 0) > 0) step = 2;
	}

	// A scan that ends while the first step waits for it moves on by itself.
	let scanned = false;
	$effect(() => {
		const running = importer.scanning !== null;
		untrack(() => {
			if (running) scanned = true;
			else if (scanned) {
				scanned = false;
				onward(!importer.error);
			}
		});
	});

	async function chooseFile() {
		const path = await openDialog({
			multiple: false,
			directory: false,
			filters: [
				{ name: 'Collections', extensions: ['json', 'yaml', 'yml', 'http', 'rest', 'har', 'bru', 'txt', 'sh'] },
				{ name: 'Every file', extensions: ['*'] }
			]
		}).catch(() => null);
		if (typeof path === 'string') onward(await importer.importFile(path));
	}

	/** A Bruno collection is a folder: `opencollection.yml` or `bruno.json` and the requests under it. */
	async function chooseFolder() {
		const path = await openDialog({ multiple: false, directory: true }).catch(() => null);
		if (typeof path === 'string') onward(await importer.importFile(path));
	}

	async function readPasted() {
		if (await importer.importText(pasted)) {
			pasted = '';
			pasting = false;
			onward(true);
		}
	}

	async function readWsdlUrl() {
		if (wsdlUrl && (await importer.importWsdlUrl(wsdlUrl))) {
			wsdlUrl = null;
			onward(true);
		}
	}

	function tag(endpoint: Endpoint): { text: string; tone: string; title: string } {
		const status = endpoint.status;
		if (status.kind === 'new') return { text: 'New', tone: 'ok', title: 'In no collection yet' };
		const name = calls.collections.find((stored) => stored.stem === status.collection)?.value?.name ?? status.collection;
		if (status.kind === 'changed') return { text: 'Changed', tone: 'warn', title: status.what };
		return { text: `In ${name}`, tone: 'muted', title: `Already in ${name}, the same shape` };
	}

	/** The endpoints shown: by service (FEAT-177), then by controller or folder. */
	const groups = $derived.by(() => {
		const out: { key: string; group: string; file: string | null; service: Service | null; endpoints: Endpoint[] }[] = [];
		for (const endpoint of importer.visible) {
			const key = `${endpoint.source}\u0000${endpoint.group}`;
			let group = out.find((g) => g.key === key);
			if (!group) {
				group = { key, group: endpoint.group, file: endpoint.file, service: serviceOf(found?.services, endpoint.file), endpoints: [] };
				out.push(group);
			}
			group.endpoints.push(endpoint);
		}
		const order = (group: (typeof out)[number]) =>
			group.service ? (found?.services ?? []).indexOf(group.service) : Number.MAX_SAFE_INTEGER;
		return out
			.map((group, index) => ({ group, index }))
			.sort((a, b) => order(a.group) - order(b.group) || a.index - b.index)
			.map(({ group }) => group);
	});

	const into = $derived(
		choices.as === 'new'
			? choices.name.trim() || '…'
			: (readable.find((stored) => stored.stem === choices.into)?.value?.name ?? '…')
	);
	const ready = $derived(
		picked > 0 && !importer.bringing && (choices.as === 'new' ? choices.name.trim() !== '' : choices.into !== null)
	);

	function next() {
		if (step < 3) step = (step + 1) as 2 | 3;
	}

	function back() {
		if (step > 1) step = (step - 1) as 1 | 2;
	}

	async function save() {
		const done = await importer.bringIn();
		if (done) onclose(done.stem);
	}

	const canNext = $derived(
		step === 1 ? total > 0 && !importer.scanning && !importer.reading : picked > 0 && !importer.agentReading
	);

	/** The controllers an agent would read for the chosen endpoints (FEAT-179). */
	const controllers = $derived(new Set(importer.forAgent.map((endpoint) => endpoint.file)).size);
	const agentName = $derived(importer.agents.find((known) => known.id === importer.agent)?.name ?? 'an agent');

	// The agents are asked for once the endpoints are on screen.
	let agentsLoaded = false;
	$effect(() => {
		if (step !== 2 || scope !== 'repository' || agentsLoaded) return;
		agentsLoaded = true;
		untrack(() => importer.loadAgents());
	});
</script>

<Sheet labelledby="import-title" width="980px" ondismiss={() => onclose(null)}>
	<div class="sheet">
		<header class="top">
			<h2 id="import-title">Import calls</h2>
			<ol class="steps" aria-label="Steps">
				{#each STEPS as name, index (name)}
					<li class:on={step === index + 1} class:done={step > index + 1} aria-current={step === index + 1 ? 'step' : undefined}>
						<span class="number">{index + 1}</span>{name}
					</li>
				{/each}
			</ol>
			<span class="grow"></span>
			<button type="button" class="close" aria-label="Close" title="Close (Esc)" onclick={() => onclose(null)}>✕</button>
		</header>

		{#if importer.error}<p class="error" role="alert">{importer.error}</p>{/if}

		<div class="body">
			{#if step === 1}
				<div class="where">
					<span class="note">Into</span>
					<ScopeSwitch {scope} repository={info?.name ?? null} onchoose={onscope} label="Where calls are imported" />
				</div>
				<div class="ways">
					{#if scope === 'repository'}
						<button type="button" class="way" disabled={importer.scanning !== null} onclick={() => importer.scan()}>
							<strong>Scan this repository’s code</strong>
							<span class="note"
								>Spring, JAX-RS and NestJS controllers, SOAP services, WSDL and OpenAPI files, and the Postman, Insomnia,
								Bruno, .http and HAR files kept here. Nothing is built or run.</span
							>
							{#if importer.scanning}
								<span class="progress" role="status"
									><Loader size="inline" label="Reading files" />Reading files… {importer.scanning.files.toLocaleString()}</span
								>
							{/if}
						</button>
					{/if}
					<button type="button" class="way" disabled={importer.reading !== null} onclick={chooseFile}>
						<strong>A file</strong>
						<span class="note">Postman, Insomnia, OpenAPI, .http, HAR, a Bruno .bru, or a script of cURL commands.</span>
					</button>
					<button type="button" class="way" disabled={importer.reading !== null} onclick={chooseFolder}>
						<strong>A Bruno collection folder</strong>
						<span class="note">The folder with bruno.json or opencollection.yml in it.</span>
					</button>
					<div class="way" class:open={pasting}>
						<button type="button" class="way-head" onclick={() => (pasting = !pasting)}>
							<strong>Paste a cURL command</strong>
							<span class="note">One request, as copied from a browser’s network panel or a terminal.</span>
						</button>
						{#if pasting}
							<textarea class="mono" rows="4" aria-label="A cURL command" placeholder="curl https://…" spellcheck="false" bind:value={pasted}
							></textarea>
							<span class="row"><Btn primary disabled={!pasted.trim() || importer.reading !== null} onclick={readPasted}>Read it</Btn></span>
						{/if}
					</div>
					<div class="way" class:open={wsdlUrl !== null}>
						<button type="button" class="way-head" onclick={() => (wsdlUrl = wsdlUrl === null ? '' : null)}>
							<strong>A WSDL from a URL</strong>
							<span class="note">Fetched once, as a call is.</span>
						</button>
						{#if wsdlUrl !== null}
							<input
								class="mono"
								type="url"
								aria-label="A WSDL's address"
								placeholder="https://…/service?wsdl"
								spellcheck="false"
								bind:value={wsdlUrl}
								onkeydown={(event) => event.key === 'Enter' && readWsdlUrl()}
							/>
							<span class="row"><Btn primary disabled={!wsdlUrl?.trim() || importer.reading !== null} onclick={readWsdlUrl}>Read it</Btn></span>
						{/if}
					</div>
				</div>
				{#if importer.reading}
					<p class="status" role="status"><Loader size="inline" label={importer.reading} />{importer.reading}</p>
				{:else if total > 0}
					<p class="status">
						{total} {total === 1 ? 'endpoint' : 'endpoints'} found, from {sources.length}
						{sources.length === 1 ? 'source' : 'sources'}. Next to choose which.
					</p>
				{/if}
			{:else if step === 2}
				<div class="filters">
					<select aria-label="Source" value={importer.source ?? ''} onchange={(event) => importer.setSource(event.currentTarget.value || null)}>
						<option value="">Every source · {total}</option>
						{#each sources as source (source.id)}
							<option value={source.id}>{KIND_LABELS[source.kind]} · {source.label} · {source.count}</option>
						{/each}
					</select>
					<div class="chips" role="radiogroup" aria-label="Show">
						{#each FILTERS as item (item.id)}
							<button
								type="button"
								role="radio"
								class="chip"
								class:on={importer.filter === item.id}
								aria-checked={importer.filter === item.id}
								onclick={() => importer.setFilter(item.id)}>{item.label} <span class="count">{counts[item.id]}</span></button
							>
						{/each}
					</div>
					<span class="grow"></span>
					<span class="note">{picked} chosen</span>
				</div>
				<div class="list">
					{#each groups as group, index (group.key)}
						{@const ids = group.endpoints.map((endpoint) => endpoint.id)}
						{#if group.service && group.service !== groups[index - 1]?.service}
							<h3 class="service" title={group.service.root || 'The repository’s root'}>
								{group.service.name}
								<span class="service-kind">{group.service.kind}</span>
								{#if group.service.root}<span class="file mono">{group.service.root}</span>{/if}
							</h3>
						{/if}
						<section class="group">
							<label class="group-head">
								<input
									type="checkbox"
									aria-label={`Every endpoint in ${group.group}`}
									checked={ids.every((id) => importer.selected.includes(id))}
									onchange={() => importer.toggleAll(ids)}
								/>
								<span class="name">{group.group}</span>
								{#if group.file}<span class="file mono">{group.file}</span>{/if}
							</label>
							{#each group.endpoints as endpoint (endpoint.id)}
								{@const shown = tag(endpoint)}
								<label class="endpoint">
									<input
										type="checkbox"
										aria-label={`${endpoint.method} ${endpoint.path}`}
										checked={importer.selected.includes(endpoint.id)}
										onchange={() => importer.toggle(endpoint.id)}
									/>
									<span class="method mono" style:color={methodColour(endpoint.method)}>{endpoint.method}</span>
									<span class="path mono" title={endpoint.call.name}>{endpoint.path}</span>
									<span class="hint">{endpoint.status.kind === 'changed' ? endpoint.status.what : endpoint.hint}</span>
									<span class="tag {shown.tone}" title={shown.title}>{shown.text}</span>
								</label>
							{/each}
						</section>
					{:else}
						<EmptyState message="Nothing here with these filters." />
					{/each}
				</div>
				{#if scope === 'repository' && controllers > 0}
					<div class="agent">
						{#if importer.agents.length}
							<span class="note"
								>An agent can read the {controllers === 1 ? 'chosen controller' : `${controllers} chosen controllers`} for rules
								and answers the scan missed. It runs nothing.</span
							>
							<span class="grow"></span>
							{#if importer.agents.length > 1}
								<select
									aria-label="The agent that reads"
									value={importer.agent ?? ''}
									onchange={(event) => importer.chooseAgent(event.currentTarget.value || null)}
								>
									{#each importer.agents as known (known.id)}
										<option value={known.id}>{known.name}</option>
									{/each}
								</select>
							{/if}
							<Btn busy={importer.agentReading} onclick={() => importer.askAgent()}>Ask {agentName}</Btn>
						{:else}
							<span class="note">To have an agent read the controllers too, set one up in Settings › Agents and allow it here.</span>
						{/if}
					</div>
					{#if importer.agentReading}
						<p class="status" role="status"><Loader size="inline" label="An agent is reading" />{agentName} is reading the code…</p>
					{:else if importer.agentSaid}
						<p class="status" role="status">{importer.agentSaid}</p>
					{/if}
				{/if}
			{:else}
				<div class="save">
					<fieldset class="part">
						<legend>Save to</legend>
						<label class="radio">
							<input
								type="radio"
								name="as"
								checked={choices.as === 'into'}
								disabled={readable.length === 0}
								onchange={() => importer.choose({ as: 'into' })}
							/>
							A collection
							<select
								aria-label="Into"
								value={choices.into ?? ''}
								disabled={choices.as !== 'into' || readable.length === 0}
								onchange={(event) => importer.choose({ into: event.currentTarget.value || null })}
							>
								{#each readable as stored (stored.stem)}
									<option value={stored.stem}>{stored.value?.name}</option>
								{/each}
							</select>
						</label>
						<label class="radio">
							<input type="radio" name="as" checked={choices.as === 'new'} onchange={() => importer.choose({ as: 'new' })} />
							A new collection
							<input
								type="text"
								aria-label="The new collection's name"
								spellcheck="false"
								value={choices.name}
								disabled={choices.as !== 'new'}
								oninput={(event) => importer.choose({ name: event.currentTarget.value })}
							/>
						</label>
					</fieldset>
					<fieldset class="part">
						<legend>Folders</legend>
						<label class="radio"
							><input
								type="radio"
								name="folders"
								checked={choices.folders === 'perGroup'}
								onchange={() => importer.choose({ folders: 'perGroup' })}
							/>One per controller{(found?.services?.length ?? 0) > 1 ? ', inside its service' : ''}</label
						>
						<label class="radio"
							><input
								type="radio"
								name="folders"
								checked={choices.folders === 'byPath'}
								onchange={() => importer.choose({ folders: 'byPath' })}
							/>By the path’s first part: /orders/… in orders</label
						>
					</fieldset>
					<fieldset class="part">
						<legend>Calls</legend>
						<label class="switch" title="A sample of the request class's fields, to a depth of four">
							<input
								type="checkbox"
								role="switch"
								checked={choices.bodies}
								onchange={(event) => importer.choose({ bodies: event.currentTarget.checked })}
							/>
							Example bodies from request classes
						</label>
						<label class="switch" title="/orders/{'{'}id{'}'} is sent as /orders/{'{{'}id{'}}'}">
							<input
								type="checkbox"
								role="switch"
								checked={choices.pathVars}
								onchange={(event) => importer.choose({ pathVars: event.currentTarget.checked })}
							/>
							Path variables become variables
						</label>
						{#if found?.baseUrl}
							<label class="switch" title={`A local environment with baseUrl ${found.baseUrl}, when there is none`}>
								<input
									type="checkbox"
									role="switch"
									checked={choices.serverAddress}
									onchange={(event) => importer.choose({ serverAddress: event.currentTarget.checked })}
								/>
								A local environment at <span class="mono">{found.baseUrl}</span>
							</label>
						{/if}
					</fieldset>
					<p class="note">A changed call gains its new fields and loses the old ones; what you typed stays.</p>
				</div>
			{/if}
		</div>

		<footer class="bottom">
			<Btn onclick={() => onclose(null)}>Cancel</Btn>
			<span class="grow"></span>
			{#if step > 1}<Btn onclick={back}>Back</Btn>{/if}
			{#if step < 3}
				<Btn primary disabled={!canNext} onclick={next}>Next</Btn>
			{:else}
				<Btn primary disabled={!ready} busy={importer.bringing} onclick={save}
					>Import {picked} {picked === 1 ? 'call' : 'calls'} into {into}</Btn
				>
			{/if}
		</footer>
	</div>
</Sheet>

<style>
	.sheet {
		height: min(720px, 80vh);
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.top {
		display: flex;
		align-items: center;
		gap: 16px;
	}

	h2 {
		margin: 0;
		font-size: var(--fs-title);
		font-weight: 600;
	}

	.steps {
		display: flex;
		gap: 14px;
		margin: 0;
		padding: 0;
		list-style: none;
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.steps li {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.steps li.on {
		color: var(--ink);
		font-weight: 600;
	}

	.number {
		display: grid;
		place-items: center;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		border: 1px solid var(--line);
		font-size: var(--fs-mono);
	}

	.steps li.on .number {
		border-color: var(--accent);
		background: var(--accent);
		color: var(--on-accent);
	}

	.steps li.done .number {
		border-color: var(--ok);
		color: var(--ok);
	}

	.close {
		padding: 2px 8px;
		border-radius: var(--r-button);
		color: var(--muted);
	}

	.close:hover {
		background: var(--hover);
		color: var(--ink);
	}

	.grow {
		flex: 1;
	}

	.body {
		flex: 1;
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
		overflow: auto;
	}

	.note,
	.count,
	.file,
	.hint {
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.error {
		margin: 0;
		color: var(--danger);
	}

	.where {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.ways {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
		gap: 10px;
	}

	.way {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		padding: 14px;
		border: 1px solid var(--line);
		border-radius: var(--r-panel);
		text-align: left;
	}

	button.way:hover:not(:disabled),
	.way-head:hover {
		border-color: color-mix(in srgb, var(--accent) 60%, var(--line));
	}

	.way.open {
		grid-column: 1 / -1;
	}

	.way-head {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		padding: 0;
		text-align: left;
		width: 100%;
		border: 0;
	}

	.way textarea,
	.way input {
		width: 100%;
		box-sizing: border-box;
	}

	.row {
		display: flex;
		justify-content: flex-end;
		width: 100%;
	}

	.progress,
	.status {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		margin: 0;
		font-size: var(--fs-secondary);
	}

	.status {
		padding: 8px 12px;
		border-radius: var(--r-button);
		background: color-mix(in srgb, var(--ok) 12%, transparent);
	}

	.filters {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	/* *Ask an agent too* (FEAT-179), under the list. */
	.agent {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px 12px;
		border: 1px solid var(--line);
		border-radius: var(--r-panel);
	}

	.agent select {
		max-width: 220px;
	}

	.filters select {
		max-width: 340px;
	}

	.chips {
		display: flex;
		gap: 4px;
	}

	.chip {
		padding: 3px 10px;
		border-radius: var(--r-pill);
		color: var(--muted);
	}

	.chip.on {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		color: var(--ink);
	}

	.list {
		flex: 1;
		min-height: 0;
		overflow: auto;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.service {
		display: flex;
		align-items: baseline;
		gap: 8px;
		margin: 10px 6px 2px;
		font-size: var(--fs-ui);
		font-weight: 600;
	}

	.service-kind {
		padding: 0 8px;
		border-radius: var(--r-pill);
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		color: var(--accent);
		font-size: var(--fs-mono);
	}

	.group {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.group-head,
	.endpoint {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 4px 6px;
		border-radius: var(--r-row);
	}

	.group-head .name {
		font-weight: 600;
	}

	.endpoint:hover {
		background: var(--hover);
	}

	.method {
		flex: none;
		width: 56px;
		font-size: var(--fs-secondary);
		font-weight: 600;
	}

	.path {
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.hint {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.tag {
		flex: none;
		padding: 1px 8px;
		border-radius: var(--r-pill);
		font-size: var(--fs-mono);
	}

	.tag.ok {
		background: color-mix(in srgb, var(--ok) 16%, transparent);
		color: var(--ok);
	}

	.tag.warn {
		background: color-mix(in srgb, var(--warn) 16%, transparent);
		color: var(--warn);
	}

	.tag.muted {
		background: var(--soft);
		color: var(--muted);
	}

	.save {
		display: flex;
		flex-direction: column;
		gap: 14px;
		max-width: 640px;
	}

	.part {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
		padding: 12px 14px;
		border: 1px solid var(--line);
		border-radius: var(--r-panel);
	}

	legend {
		padding: 0 4px;
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.radio,
	.switch {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.radio select,
	.radio input[type='text'] {
		margin-left: auto;
		width: 280px;
	}

	.bottom {
		display: flex;
		align-items: center;
		gap: 8px;
		padding-top: 10px;
		border-top: 1px solid var(--soft);
	}
</style>
