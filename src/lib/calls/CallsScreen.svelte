<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import * as appApi from '$lib/api';
	import { PRODUCT_NAME } from '$lib/brand';
	import { isMac } from '$lib/platform';
	import { repo } from '$lib/repo.svelte';
	import { settings } from '$lib/settings/store.svelte';
	import { dialog } from '$lib/ui/dialog.svelte';
	import EmptyState from '$lib/ui/EmptyState.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import Splitter from '$lib/ui/Splitter.svelte';
	import { panels } from '$lib/panels.svelte';
	import type { Scope } from './api';
	import ScopeSwitch from './ScopeSwitch.svelte';
	import Menu from '$lib/ui/Menu.svelte';
	import type { MenuItem } from '$lib/ui/menu';
	import CollectionTree from './CollectionTree.svelte';
	import HistoryPanel from './HistoryPanel.svelte';
	import ImportSheet from './ImportSheet.svelte';
	import { importer } from './import.svelte';
	import { envColour, findCall, lookup, methodColour, methodOf } from './model';
	import RequestEditor from './RequestEditor.svelte';
	import CurlWindow from './CurlWindow.svelte';
	import ShareDialog from './ShareDialog.svelte';
	import SoapEditor from './SoapEditor.svelte';
	import ResponseView from './ResponseView.svelte';
	import { calls, keyOf } from './store.svelte';
	import type { HistoryEntry, Opened } from './types';
	import VariablesPanel from './VariablesPanel.svelte';

	/**
	 * Calls (2C, FEAT-160): requests saved beside the code.
	 *
	 * Collections on the left, the request above its response in the middle,
	 * the active environment's variables on the right. A chip in the header
	 * for each open call; `Ctrl+W` or a middle-click closes one. Requests are
	 * made in Rust; this screen only shows what comes back.
	 */

	const info = $derived(repo.info);
	const current = $derived(calls.current);
	const environment = $derived(calls.environment);
	const mod = isMac() ? '⌘' : 'Ctrl';

	let search = $state<HTMLInputElement | null>(null);
	let url = $state<HTMLInputElement | HTMLTextAreaElement | null>(null);
	let envMenu = $state<{ x: number; y: number; anchor: HTMLElement } | null>(null);

	/**
	 * Where collections are kept (FEAT-170): the choice in Settings, and
	 * Workbench-wide with no repository open, so Calls works without one.
	 */
	const wanted = $derived<Scope>(settings.settings.callsScope ?? 'repository');

	/** The calls of the place chosen: on arrival, when another repository is opened, and on a switch. */
	$effect(() => {
		const path = info?.path ?? null;
		const scope = wanted;
		void repo.generation;
		untrack(() => {
			if (!appApi.inTauri()) return;
			void (async () => {
				await calls.load(path, scope);
				await importer.load(path, calls.scope);
			})();
		});
	});

	/**
	 * Import, as a sheet over Calls (FEAT-178): from the Collections `+`, the
	 * card that counts endpoints in no collection, and `/calls?import`. It
	 * opens at its second step when something is found already.
	 */
	let importing = $state<1 | 2 | null>(null);
	const openImport = () => (importing = importer.found?.endpoints.length ? 2 : 1);

	$effect(() => {
		if (page.url.searchParams.has('import')) {
			untrack(openImport);
			void goto('/calls', { replaceState: true });
		}
	});

	/** The sheet closed; the collection the calls went into is shown open. */
	function closeImport(stem: string | null) {
		importing = null;
		if (!stem) return;
		if (calls.hidden.includes(stem)) calls.showCollection(stem);
		if (calls.person.collapsed.includes(`c:${stem}`)) calls.toggleCollapsed(`c:${stem}`);
	}

	async function newCollection() {
		const name = await dialog.prompt({
			title: 'New collection',
			body: calls.scope === 'global' ? `Kept in ${PRODUCT_NAME}, for every repository.` : 'Kept in this repository, as JSON.',
			label: 'Name',
			confirmLabel: 'Create',
			placeholder: 'orders-api'
		});
		if (name?.trim()) await calls.createCollection(name);
	}

	/** The switch in the head writes the setting, which the effect above reads. */
	function chooseScope(next: Scope) {
		if (next === 'repository' && !info) return;
		void settings.choose('callsScope', next);
	}

	const hiddenCollections = $derived(calls.collections.filter((stored) => calls.hidden.includes(stored.stem)));

	/** Delete a collection's file, after saying which file and that it is for good (BUG-106). */
	async function deleteCollection(stem: string) {
		const stored = calls.collections.find((candidate) => candidate.stem === stem);
		if (!stored) return;
		const name = stored.value?.name ?? stem;
		const agreed = await dialog.confirm({
			title: `Delete ${name}`,
			body:
				calls.scope === 'repository'
					? `${stored.file} is deleted from the working tree, with its calls. A version already committed stays in history. To take it off the list and keep the file, hide it instead.`
					: `${name} and its calls are deleted from ${PRODUCT_NAME}. To take it off the list and keep it, hide it instead.`,
			confirmLabel: 'Delete',
			danger: true
		});
		if (agreed) await calls.deleteCollection(stem);
	}

	/** FEAT-162: endpoints the code declares that no collection has, as a card. */
	const card = $derived.by(() => {
		if (calls.scope !== 'repository') return null;
		const missing = importer.notInACollection;
		if (!missing.length) return null;
		const files = [...new Set(missing.map((endpoint) => endpoint.file?.split('/').pop() ?? endpoint.group))];
		return { count: missing.length, files };
	});
	const askScan = $derived(
		calls.scope === 'repository' && calls.settings?.scanOnOpen === 'ask' && !importer.found && !importer.scanning && !importer.dismissed
	);

	/** Every name a field can offer: what each scope defines. */
	const names = $derived(
		[
			...(environment?.vars ?? []),
			...calls.shared.vars,
			...(current?.collection.vars ?? [])
		].map((variable) => variable.name)
	);

	/** Where a name is defined, with a kept value's value from the keychain. */
	const find = (name: string) => {
		const found = lookup(name, environment, calls.shared, current?.collection ?? null);
		if (!found?.variable?.kept) return found;
		const view = calls.private.kept.find((kept) => kept.name === name);
		return { ...found, variable: { ...found.variable, value: view?.value ?? undefined } };
	};

	let envButton = $state<HTMLButtonElement | null>(null);
	/** The open calls' strip, one row that scrolls (BUG-114). */
	let strip = $state<HTMLDivElement | null>(null);
	// The call shown is the chip in view, wherever it sits on the strip.
	$effect(() => {
		if (current === null) return;
		keyOf(current.opened);
		strip?.querySelector('.chip.on')?.scrollIntoView?.({ block: 'nearest', inline: 'nearest' });
	});
	// The strip has no bar, so the mouse wheel scrolls it sideways.
	function wheelStrip(event: WheelEvent) {
		if (strip === null || event.deltaY === 0 || Math.abs(event.deltaX) > Math.abs(event.deltaY)) return;
		if (strip.scrollWidth <= strip.clientWidth) return;
		event.preventDefault();
		strip.scrollLeft += event.deltaY;
	}
	/** FEAT-164: the Share dialog is open. */
	let sharing = $state(false);
	/** Share opened from a collection's `⋯`: that collection, and OpenAPI first (FEAT-179). */
	let shareFor = $state<{ stem: string; openapi: boolean } | null>(null);
	/** FEAT-166: the cURL window is open. */
	let curling = $state(false);

	const envItems = $derived<MenuItem[]>([
		...calls.environments.map((stored) => ({
			id: stored.stem,
			label: stored.value?.name ?? stored.stem,
			note: stored.stem === calls.person.active ? '✓' : undefined,
			disabled: !stored.value,
			reason: stored.error ? 'could not be read' : undefined,
			run: () => calls.setActive(stored.stem)
		})),
		...(calls.environments.length ? [{ separator: true } as const] : []),
		{
			id: '-',
			label: 'No environment',
			note: calls.person.active === null ? '✓' : undefined,
			run: () => calls.setActive(null)
		}
	]);

	function openEnv(event: { currentTarget: EventTarget | null }) {
		const anchor = event.currentTarget as HTMLElement;
		if (envMenu?.anchor === anchor) {
			envMenu = null;
			return;
		}
		const box = anchor.getBoundingClientRect();
		envMenu = { x: box.left, y: box.bottom + 4, anchor };
	}

	function onkey(event: KeyboardEvent) {
		if (!(event.ctrlKey || event.metaKey) || event.altKey) {
			if (event.key === 'Escape' && calls.asking) calls.dismissAsk();
			return;
		}
		if (event.key === 'Enter') {
			event.preventDefault();
			void calls.send(calls.asking !== null && current !== null && calls.asking === keyOf(current.opened));
		} else if (event.key === 'w' || event.key === 'W') {
			if (!calls.person.current) return;
			event.preventDefault();
			calls.close(calls.person.current);
		} else if (event.key === 'e' || event.key === 'E') {
			// FEAT-161: switch environment from anywhere in Calls.
			event.preventDefault();
			if (envButton) openEnv({ currentTarget: envButton });
		} else if (event.key === 'k' || event.key === 'K') {
			event.preventDefault();
			search?.focus();
			search?.select();
		}
	}

	function chipDown(event: MouseEvent, opened: Opened) {
		if (event.button === 1) {
			event.preventDefault();
			calls.close(opened);
		}
	}

	async function toggleHistory() {
		if (calls.history) calls.closeHistory();
		else await calls.loadHistory();
	}

	function openFromHistory(entry: HistoryEntry) {
		const stored = calls.collections.find((candidate) => candidate.stem === entry.collection);
		if (!stored?.value || !entry.callId || !findCall(stored.value.items, entry.callId)) return;
		calls.closeHistory();
		calls.open({ stem: stored.stem, id: entry.callId });
	}
</script>

<svelte:window onkeydown={onkey} />

<!-- With no repository open, Calls is Workbench-wide (FEAT-170): it no longer waits for one. -->
	<div class="screen">
		<header class="head">
			<span class="title">Calls</span>
			<ScopeSwitch scope={calls.scope} repository={info?.name ?? null} onchoose={chooseScope} />
			<div class="chips" role="tablist" aria-label="Open calls" bind:this={strip} onwheel={wheelStrip}>
				{#each calls.tabs as { opened, call } (keyOf(opened))}
					{@const on = current !== null && keyOf(current.opened) === keyOf(opened)}
					{@const method = methodOf(call)}
					<span class="chip" class:on>
						<button
							type="button"
							role="tab"
							class="open"
							aria-selected={on}
							title={call.url}
							onclick={() => calls.open(opened)}
							onauxclick={(event) => chipDown(event, opened)}
							onmousedown={(event) => event.button === 1 && event.preventDefault()}
							><span class="method mono" style:color={methodColour(method)}>{method}</span>
							{call.name}</button
						><button
							type="button"
							class="close"
							aria-label={`Close ${call.name}`}
							title={`Close (${mod}+W)`}
							onclick={() => calls.close(opened)}>✕</button
						>
					</span>
				{/each}
				<button type="button" class="chip add" aria-label="New call" title="New call" onclick={() => calls.create()}
					>+</button
				>
			</div>
			<span class="gap"></span>
			<button
				type="button"
				class="env"
				title={`The environment calls are sent in (${mod}+E)`}
				bind:this={envButton}
				onclick={openEnv}
			>
				<span class="muted">Env</span>
				{#if environment}
					<span class="dot" style:background={envColour(environment.color)} aria-hidden="true"></span>
					{environment.name}
				{:else}
					none
				{/if}
				<span class="muted">▾</span>
			</button>
			<button
				type="button"
				class="env"
				title="Export a collection; secrets stay here"
				disabled={!calls.collections.some((stored) => stored.value)}
				onclick={() => (sharing = true)}>Share</button
			>
		</header>

		{#if calls.error}
			<p class="error" role="alert">{calls.error}</p>
		{/if}

		<div class="body">
			<CollectionTree
				collections={calls.shown}
				filter={calls.person.filter}
				collapsed={calls.person.collapsed}
				current={calls.person.current}
				today={calls.today}
				historyOpen={calls.history !== null}
				onfilter={(filter) => calls.setFilter(filter)}
				ontoggle={(id) => calls.toggleCollapsed(id)}
				onopen={(opened) => {
					calls.closeHistory();
					calls.open(opened);
				}}
				oncreate={(stem, folder) => calls.create(stem, folder)}
				onduplicate={(opened) => calls.duplicate(opened)}
				onremove={(opened) => calls.remove(opened)}
				onhistory={toggleHistory}
				bind:search
				found={calls.found}
				{card}
				{askScan}
				onscan={() => importer.scan()}
				onreview={() => (importing = 2)}
				onimport={openImport}
				onnewcollection={newCollection}
				ondismiss={() => importer.dismiss()}
				hidden={hiddenCollections}
				onhide={(stem) => calls.hideCollection(stem)}
				onshow={(stem) => calls.showCollection(stem)}
				ondelete={deleteCollection}
				onshare={(stem, openapi) => {
					shareFor = { stem, openapi };
					sharing = true;
				}}
			/>
			<Splitter panel="callsTree" label="Resize the collections" />

			<main class="middle">
				{#if calls.history}
					<div class="card grow">
						<HistoryPanel
							entries={calls.history}
							onclose={() => calls.closeHistory()}
							onclear={() => calls.clearHistory()}
							onopen={openFromHistory}
						/>
					</div>
				{:else if current}
					{@const opened = current.opened}
					<div class="card">
						<input
							class="name"
							type="text"
							aria-label="The call's name"
							spellcheck="false"
							value={current.call.name}
							oninput={(event) => {
								const name = event.currentTarget.value;
								calls.edit(opened, (call) => ({ ...call, name }));
							}}
						/>
						{#if current.call.kind === 'soap'}
							<SoapEditor
								call={current.call}
								wsdl={current.collection.wsdl ?? null}
								onchange={(change) => calls.edit(opened, change)}
								{find}
								{names}
								sending={calls.sendingFor(opened)}
								refusal={calls.refusalFor(opened)}
								onsend={() => calls.send()}
								oncancel={() => calls.cancel(opened)}
								oncurl={() => (curling = true)}
								bind:url
							/>
						{:else}
							<RequestEditor
								call={current.call}
								onchange={(change) => calls.edit(opened, change)}
								{find}
								{names}
								sending={calls.sendingFor(opened)}
								refusal={calls.refusalFor(opened)}
								onsend={() => calls.send()}
								oncancel={() => calls.cancel(opened)}
								oncurl={() => (curling = true)}
								bind:url
							/>
						{/if}
						{#if environment && calls.asking === keyOf(opened)}
							<div class="ask" role="alertdialog" aria-label="Ask before sending" style:--env={envColour(environment.color)}>
								<span class="dot" aria-hidden="true"></span>
								<span>Send <strong>{current.call.name}</strong> to <strong>{environment.name}</strong>?</span>
								<span class="gap"></span>
								<button type="button" class="yes" onclick={() => calls.send(true)}>Send</button>
								<button type="button" class="no" onclick={() => calls.dismissAsk()}>Not now</button>
							</div>
						{/if}
					</div>
					<div class="card grow">
						<ResponseView
							outcome={calls.outcomeFor(opened)}
							sending={calls.sendingFor(opened)}
							beautify={calls.settings?.beautify ?? true}
							name={current.call.name}
							ontoggleraw={() => calls.toggleRaw(opened)}
							onsave={(path) => calls.saveBody(opened, path)}
							rules={current.call.keep ?? []}
							{environment}
							keepEvery={calls.settings?.keepEvery ?? true}
							suggest={calls.settings?.suggestNames ?? true}
							onkeep={calls.settings?.clickToKeep === false ? undefined : (ask) => calls.keep(opened, ask)}
							onforget={(name) => calls.forget(opened, name)}
							soap={current.call.kind === 'soap'}
						/>
					</div>
				{:else if calls.loaded}
					<div class="card grow">
						<EmptyState message="No call open" />
					</div>
				{/if}
			</main>

			{#if panels.isHidden('callsVars')}
				<button
					type="button"
					class="vars-tab"
					title="Show the variables"
					onclick={() => panels.toggleHidden('callsVars')}
				>
					<Icon name="chevron-left" size="0.95em" weight={2.2} />
					<span class="tab-label">Variables</span>
				</button>
			{:else}
				<Splitter panel="callsVars" label="Resize the variables" />
				<VariablesPanel
					{environment}
					shared={calls.shared}
					collection={current?.collection ?? null}
					privateValues={calls.private}
					privateError={calls.privateError}
					onmanage={() => goto('/calls/environments')}
					onhide={() => panels.toggleHidden('callsVars')}
				/>
			{/if}
		</div>
	</div>

{#if importing}
	<ImportSheet start={importing} scope={calls.scope} onscope={chooseScope} onclose={closeImport} />
{/if}

{#if curling && current}
	<CurlWindow
		opened={current.opened}
		call={current.call}
		environment={calls.person.active}
		collections={calls.collections}
		onreplace={(call) => current && calls.replaceRequest(current.opened, call)}
		oncreate={(call, stem, folder) => calls.create(stem, folder, call)}
		onclose={() => (curling = false)}
	/>
{/if}

{#if sharing}
	<ShareDialog
		collections={calls.collections}
		environments={calls.environments}
		stem={shareFor?.stem ?? (current && !current.opened.stem.startsWith('found:') ? current.opened.stem : null)}
		shareAs={shareFor?.openapi ? 'file' : (calls.settings?.shareAs ?? 'zip')}
		flavour={shareFor?.openapi ? 'openapi' : undefined}
		onclose={() => {
			sharing = false;
			shareFor = null;
		}}
	/>
{/if}

{#if envMenu}
	<Menu
		x={envMenu.x}
		y={envMenu.y}
		anchor={envMenu.anchor}
		label="Environment"
		items={envItems}
		onclose={() => (envMenu = null)}
	/>
{/if}

<style>
	.screen {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	/*
	 * Flat fields (BUG-113): the card's colour with a hairline, and the
	 * accent ring on focus, rather than a well a shade under the card. The
	 * request and the response were dark slabs inside lighter cards.
	 */
	.screen :global(:is(input:not([type='checkbox']):not([type='radio']), textarea, select)),
	.screen :global(:is(input:not([type='checkbox']):not([type='radio']), textarea, select):is(:hover, :focus)) {
		background-color: transparent;
	}

	/* One row: the open calls scroll rather than wrap onto a second (BUG-114). */
	.head {
		flex: none;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px 10px;
	}

	.head > :not(.chips, .gap) {
		flex: none;
	}

	.title {
		margin-right: 6px;
		font-size: var(--fs-title);
		white-space: nowrap;
	}

	.chips {
		flex: 0 1 auto;
		display: flex;
		gap: 6px;
		min-width: 0;
		overflow-x: auto;
		scrollbar-width: none;
	}

	.chip {
		flex: none;
		display: inline-flex;
		align-items: center;
		border: 1px solid var(--soft);
		border-radius: var(--r-pill);
		background-color: var(--surface-veil);
		white-space: nowrap;
	}

	.chip:hover {
		background-color: var(--hover);
	}

	.chip.on {
		border-color: color-mix(in srgb, var(--accent) 62%, transparent);
		background-color: color-mix(in srgb, var(--accent) 14%, var(--surface));
	}

	.chip .open {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 4px 3px 10px;
	}

	.chip.on .open {
		color: var(--accent);
	}

	.chip .method {
		font-size: var(--fs-secondary);
		font-weight: 600;
	}

	.chip .close {
		padding: 3px 9px 3px 5px;
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.chip .close:hover {
		color: var(--ink);
	}

	.chip.add {
		padding: 3px 10px;
		color: var(--muted);
	}

	.gap {
		flex: 1;
	}

	.env {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 12px;
		border: 1px solid var(--soft);
		border-radius: var(--r-button);
	}

	.env:hover {
		background: var(--hover);
	}

	.muted {
		color: var(--muted);
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}

	.error {
		margin: 0;
		padding: 0 16px 8px;
		color: var(--danger);
	}

	.body {
		flex: 1;
		min-height: 0;
		display: flex;
		gap: 10px;
		padding: 0 12px 12px;
	}

	/* The dividers sit in the gap between the panels, not beside it. */
	.body > :global(.splitter) {
		margin-inline: -8px;
	}

	/* The variables put away (FEAT-175): a tab down the right edge. */
	.vars-tab {
		flex: none;
		width: 30px;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 10px 0;
		border-radius: var(--r-panel);
		background: var(--surface);
		border: 1px solid var(--pane-edge);
		color: var(--muted);
	}

	.vars-tab:hover {
		color: var(--ink);
		background: var(--hover);
	}

	.tab-label {
		writing-mode: vertical-rl;
		font-size: var(--fs-secondary);
	}

	.middle {
		flex: 1;
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.card {
		flex: none;
		max-height: 55%;
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px;
		background: var(--surface);
		border: 1px solid var(--pane-edge);
		border-radius: var(--r-panel);
		overflow: hidden;
	}

	.card.grow {
		flex: 1;
		max-height: none;
		min-height: 0;
	}

	.name {
		align-self: flex-start;
		min-width: 200px;
		padding: 2px 6px;
		border: 1px solid transparent;
		background: transparent;
		font-weight: 600;
	}

	.name:hover,
	.name:focus {
		border-color: var(--line);
	}

	.ask {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 12px;
		border: 1px solid color-mix(in srgb, var(--env) 55%, transparent);
		border-radius: var(--r-field);
		background: color-mix(in srgb, var(--env) 12%, var(--surface));
	}

	.ask .dot {
		background: var(--env);
	}

	.ask strong {
		color: var(--env);
	}

	.ask .yes,
	.ask .no {
		padding: 3px 12px;
		border-radius: var(--r-button);
	}

	.ask .yes {
		background: var(--env);
		color: var(--on-accent);
		font-weight: 600;
	}

	.ask .no {
		border: 1px solid var(--soft);
	}
</style>
