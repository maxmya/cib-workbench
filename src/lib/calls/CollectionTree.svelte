<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import Icon from '$lib/ui/Icon.svelte';
	import Menu from '$lib/ui/Menu.svelte';
	import type { MenuItem } from '$lib/ui/menu';
	import { isMac } from '$lib/platform';
	import { callsIn, filterItems, isFolder, methodColour, methodOf } from './model';
	import type { Collection, Item, Opened, Stored } from './types';

	/**
	 * Collections, on the left of Calls (FEAT-160, proposal §5.1): a filter,
	 * each collection with its file and count, its folders, and its calls as
	 * method and name. A collapsed collection shows its kind and count. A
	 * file that could not be read is listed with why, and cannot be opened.
	 * *History* is at the foot.
	 */

	interface Props {
		collections: Stored<Collection>[];
		filter: string;
		collapsed: string[];
		current: Opened | null;
		today: number;
		historyOpen: boolean;
		onfilter: (filter: string) => void;
		ontoggle: (id: string) => void;
		onopen: (opened: Opened) => void;
		oncreate: (stem: string | null, folder: string | null) => void;
		onduplicate: (opened: Opened) => void;
		onremove: (opened: Opened) => void;
		onhistory: () => void;
		search?: HTMLInputElement | null;
		/** Collection files found in the repository, opened in place (FEAT-162). */
		found?: Stored<Collection>[];
		/** Endpoints the code declares that are in no collection, and their files. */
		card?: { count: number; files: string[] } | null;
		/** Settings says to ask before scanning, and nobody has answered. */
		askScan?: boolean;
		onscan?: () => void;
		onreview?: () => void;
		ondismiss?: () => void;
		/** Collections taken off the list, to bring back (BUG-106). */
		hidden?: Stored<Collection>[];
		/** Take a collection off the list, keeping its file. */
		onhide?: (stem: string) => void;
		onshow?: (stem: string) => void;
		/** Delete a collection's file; the caller asks first. */
		ondelete?: (stem: string) => void;
		/** Open Share on a collection; `openapi` chooses OpenAPI first (FEAT-179). */
		onshare?: (stem: string, openapi: boolean) => void;
		/** Where the collections are kept, for the empty list to say (FEAT-170). */
		where?: string;
		/** The `+` menu (FEAT-178): a new collection, and Import. */
		onnewcollection?: () => void;
		onimport?: () => void;
	}

	let {
		collections,
		filter,
		collapsed,
		current,
		today,
		historyOpen,
		onfilter,
		ontoggle,
		onopen,
		oncreate,
		onduplicate,
		onremove,
		onhistory,
		search = $bindable(null),
		found = [],
		card = null,
		askScan = false,
		onscan,
		onreview,
		ondismiss,
		hidden = [],
		onhide,
		onshow,
		ondelete,
		onshare,
		where = 'in this repository, as JSON, so they can be committed',
		onnewcollection,
		onimport
	}: Props = $props();

	let menu = $state<{ x: number; y: number; items: MenuItem[] } | null>(null);
	/** The hidden collections are listed under the rest (BUG-106). */
	let showingHidden = $state(false);

	/**
	 * A collection's own menu (BUG-106): from its `⋯`, or a right-click on
	 * its head. There was only *New call*, so nothing could take a collection
	 * away.
	 */
	function collectionItems(stem: string, readable: boolean): MenuItem[] {
		return [
			...(readable ? [{ id: 'new', label: 'New call', run: () => oncreate(stem, null) }] : []),
			...(readable && onshare
				? [
						{ id: 'share', label: 'Share…', run: () => onshare(stem, false) },
						{ id: 'openapi', label: 'Export as OpenAPI…', note: 'with field rules and answers', run: () => onshare(stem, true) }
					]
				: []),
			...(readable ? [{ separator: true } as const] : []),
			{ id: 'hide', label: 'Hide from the list', note: 'the file stays', run: () => onhide?.(stem) },
			{ id: 'delete', label: 'Delete collection…', danger: true, run: () => ondelete?.(stem) }
		];
	}

	function collectionMenu(event: MouseEvent, stem: string, readable: boolean, at: 'pointer' | 'anchor') {
		event.preventDefault();
		event.stopPropagation();
		const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
		menu = {
			x: at === 'pointer' ? event.clientX : box.left,
			y: at === 'pointer' ? event.clientY : box.bottom + 4,
			items: collectionItems(stem, readable)
		};
	}
	const searchKeys = isMac() ? '⌘ K' : 'Ctrl K';

	/** The `+` at the top (FEAT-178): where more calls come from. */
	function addMenu(event: MouseEvent) {
		const box = (event.currentTarget as HTMLElement).getBoundingClientRect();
		menu = {
			x: box.left,
			y: box.bottom + 4,
			items: [
				{ id: 'call', label: 'New call', run: () => oncreate(null, null) },
				{ id: 'collection', label: 'New collection…', run: () => onnewcollection?.() },
				{ separator: true },
				{ id: 'import', label: 'Import calls…', note: 'code, Postman, OpenAPI, cURL', run: () => onimport?.() }
			]
		};
	}

	/** What a collection is, for when it is collapsed: REST, SOAP or both. */
	function kindOf(collection: Collection): string {
		const kinds = new Set(callsIn(collection.items).map((call) => call.kind));
		if (kinds.size === 2) return 'REST · SOAP';
		return kinds.has('soap') ? 'SOAP' : 'REST';
	}

	const isOpen = (stem: string, id: string) => current?.stem === stem && current.id === id;

	function callMenu(event: MouseEvent, opened: Opened) {
		event.preventDefault();
		menu = {
			x: event.clientX,
			y: event.clientY,
			items: [
				{ id: 'open', label: 'Open', run: () => onopen(opened) },
				{ id: 'duplicate', label: 'Duplicate', run: () => onduplicate(opened) },
				{ separator: true },
				{ id: 'delete', label: 'Delete', danger: true, run: () => onremove(opened) }
			]
		};
	}

	function folderMenu(event: MouseEvent, stem: string, folder: string | null) {
		event.preventDefault();
		menu = {
			x: event.clientX,
			y: event.clientY,
			items: [{ id: 'new', label: folder ? `New call in ${folder}` : 'New call', run: () => oncreate(stem, folder) }]
		};
	}
</script>

<aside class="tree" aria-label="Collections">
	<div class="top">
		<label class="filter">
			<Icon name="search" size={14} />
			<input
				bind:this={search}
				type="text"
				placeholder="Filter calls"
				aria-label="Filter calls"
				spellcheck="false"
				value={filter}
				oninput={(event) => onfilter(event.currentTarget.value)}
				onkeydown={(event) => {
					if (event.key === 'Escape' && filter) {
						event.stopPropagation();
						onfilter('');
					}
				}}
			/>
			<kbd class="mono">{searchKeys}</kbd>
		</label>
		<button
			type="button"
			class="add"
			aria-label="New call, new collection or import"
			title="New call, new collection, or import calls"
			onclick={addMenu}><Icon name="plus" size={15} /></button
		>
	</div>

	<div class="list">
		{#each collections as stored (stored.stem)}
			{@const id = `c:${stored.stem}`}
			{@const shut = collapsed.includes(id) && !filter.trim()}
			{#if stored.value}
				{@const items = filterItems(stored.value.items, filter)}
				{@const count = callsIn(stored.value.items).length}
				{#if !filter.trim() || items.length}
					<section class="collection">
						<div class="head-row">
							<button
								type="button"
								class="head"
								aria-expanded={!shut}
								title={stored.file}
								onclick={() => ontoggle(id)}
								oncontextmenu={(event) => collectionMenu(event, stored.stem, true, 'pointer')}
							>
								{#if shut}<span class="arrow" aria-hidden="true">▸</span>{/if}
								<span class="name">{stored.value.name}</span>
								<span class="gap"></span>
								{#if shut}
									<span class="kind mono">{kindOf(stored.value)}</span>
								{/if}
								<span class="count">{count} {shut ? '' : count === 1 ? 'call' : 'calls'}</span>
							</button>
							<button
								type="button"
								class="more"
								aria-label={`${stored.value.name}: more`}
								title="New call, hide or delete"
								onclick={(event) => collectionMenu(event, stored.stem, true, 'anchor')}
								><Icon name="more" size={14} /></button
							>
						</div>
						{#if !shut}
							<p class="file mono">{stored.file}</p>
							{@render tree(stored.stem, items, 0)}
							{#if count === 0}
								<button type="button" class="empty" onclick={() => oncreate(stored.stem, null)}>+ New call</button>
							{/if}
						{/if}
					</section>
				{/if}
			{:else}
				<section class="collection broken" title={stored.error ?? ''}>
					<div class="head-row">
						<div class="head"><span class="name">{stored.stem}</span></div>
						<!-- A file that cannot be read can still be taken away (BUG-106). -->
						<button
							type="button"
							class="more"
							aria-label={`${stored.stem}: more`}
							title="Hide or delete"
							onclick={(event) => collectionMenu(event, stored.stem, false, 'anchor')}
							><Icon name="more" size={14} /></button
						>
					</div>
					<p class="file mono">{stored.file}</p>
					<p class="why">Not read: {stored.error}</p>
				</section>
			{/if}
		{:else}
			<div class="none">
				<p>No collections {hidden.length ? 'on the list' : 'yet'}. They are kept {where}.</p>
				<button type="button" class="empty" onclick={() => oncreate(null, null)}>+ New call</button>
			</div>
		{/each}

		{#if found.length || card || askScan}
			<h3 class="found-head">Found in this repository</h3>
		{/if}
		{#each found as stored (stored.stem)}
			{@const shut = !collapsed.includes(`open:${stored.stem}`) && !filter.trim()}
			{#if stored.value}
				{@const items = filterItems(stored.value.items, filter)}
				{#if !filter.trim() || items.length}
					<section class="collection found">
						<button
							type="button"
							class="head"
							aria-expanded={!shut}
							title={`${stored.file}: opened in place, never rewritten. Import it to keep changes.`}
							onclick={() => ontoggle(`open:${stored.stem}`)}
						>
							<span class="arrow" aria-hidden="true">{shut ? '▸' : '▾'}</span>
							<span class="name">{stored.value.name}</span>
							<span class="gap"></span>
							<span class="count">{callsIn(stored.value.items).length}</span>
						</button>
						{#if !shut}
							<p class="file mono">{stored.file}</p>
							{@render tree(stored.stem, items, 0)}
						{/if}
					</section>
				{/if}
			{/if}
		{/each}
		{#if card}
			<button type="button" class="card" onclick={onreview}>
				<span class="card-line"
					><span class="dot" aria-hidden="true"></span>{card.count}
					{card.count === 1 ? 'endpoint' : 'endpoints'} not in a collection</span
				>
				<span class="card-note">{card.files.slice(0, 3).join(', ')}{card.files.length > 3 ? ', …' : ''} · Review</span>
			</button>
		{:else if askScan}
			<div class="card ask">
				<span class="card-line">Look for endpoints in this repository's code?</span>
				<span class="card-note">Reads the source files only. Nothing is built or run.</span>
				<span class="card-actions">
					<button type="button" class="empty" onclick={ondismiss}>Not now</button>
					<button type="button" class="empty" onclick={onscan}>Scan</button>
				</span>
			</div>
		{/if}
	</div>

	<!-- BUG-106: what was hidden can be brought back, or deleted for good. -->
	{#if hidden.length}
		<div class="hidden">
			<button
				type="button"
				class="hidden-head"
				aria-expanded={showingHidden}
				onclick={() => (showingHidden = !showingHidden)}
				><span class="arrow" aria-hidden="true">{showingHidden ? '▾' : '▸'}</span>{hidden.length}
				{hidden.length === 1 ? 'collection' : 'collections'} hidden</button
			>
			{#if showingHidden}
				{#each hidden as stored (stored.stem)}
					<div class="hidden-row">
						<span class="label" title={stored.file}>{stored.value?.name ?? stored.stem}</span>
						<button type="button" class="empty" onclick={() => onshow?.(stored.stem)}>Show</button>
						<button type="button" class="empty danger" onclick={() => ondelete?.(stored.stem)}>Delete…</button>
					</div>
				{/each}
			{/if}
		</div>
	{/if}

	<button type="button" class="history" class:on={historyOpen} aria-pressed={historyOpen} onclick={onhistory}>
		<Icon name="history" size={15} />
		<span>History</span>
		<span class="gap"></span>
		<span class="count">{today} today</span>
	</button>
</aside>

{#snippet tree(stem: string, items: Item[], depth: number, trail = '')}
	{#each items as item, index (isFolder(item) ? `f:${item.folder}:${index}` : item.id)}
		{#if isFolder(item)}
			<!-- The folders above it are in its id: two services can each have a HealthController. -->
			{@const id = `f:${stem}/${trail}${item.folder}`}
			{@const shut = collapsed.includes(id) && !filter.trim()}
			<!-- A folder of folders is a service's (FEAT-177): Import nests controllers in it. -->
			{@const service = item.items.length > 0 && item.items.every(isFolder)}
			<button
				type="button"
				class="folder"
				class:service
				style:padding-left={`${8 + depth * 14}px`}
				aria-expanded={!shut}
				title={service ? `${item.folder}: its controllers are inside` : undefined}
				onclick={() => ontoggle(id)}
				oncontextmenu={(event) => folderMenu(event, stem, item.folder)}
				><span class="arrow" aria-hidden="true">{shut ? '▸' : '▾'}</span>{#if service}<Icon
						name="machine"
						size={13}
					/>{/if}{item.folder}</button
			>
			{#if !shut}
				{@render tree(stem, item.items, depth + 1, `${trail}${item.folder}/`)}
			{/if}
		{:else}
			{@const method = methodOf(item)}
			<button
				type="button"
				class="call"
				class:on={isOpen(stem, item.id)}
				aria-current={isOpen(stem, item.id) ? 'true' : undefined}
				style:padding-left={`${8 + depth * 14}px`}
				title={item.url}
				onclick={() => onopen({ stem, id: item.id })}
				oncontextmenu={(event) => callMenu(event, { stem, id: item.id })}
				><span class="method mono" style:color={methodColour(method)}>{method}</span><span class="label"
					>{item.name}</span
				></button
			>
		{/if}
	{/each}
{/snippet}

{#if menu}
	<Menu x={menu.x} y={menu.y} label="Call" items={menu.items} onclose={() => (menu = null)} />
{/if}

<style>
	.tree {
		flex: none;
		width: var(--calls-tree-w, 260px);
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 10px;
		background: var(--surface);
		border: 1px solid var(--pane-edge);
		border-radius: var(--r-panel);
	}

	/* The filter and the `+` share the top row (FEAT-178). */
	.top {
		display: flex;
		gap: 6px;
	}

	.top .filter {
		flex: 1;
		min-width: 0;
	}

	.add {
		flex: none;
		display: grid;
		place-items: center;
		width: 30px;
		border: 1px solid var(--line);
		border-radius: var(--r-field);
		color: var(--muted);
	}

	.add:hover {
		background: var(--hover);
		color: var(--ink);
	}

	.filter {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 4px 8px;
		border: 1px solid var(--line);
		border-radius: var(--r-field);
		color: var(--muted);
	}

	.filter input {
		flex: 1;
		min-width: 0;
		padding: 2px 0;
		border: 0;
		background: transparent;
		color: var(--ink);
	}

	.filter input:focus {
		outline: none;
	}

	.filter:focus-within {
		border-color: var(--ring);
	}

	kbd {
		padding: 0 5px;
		border: 1px solid var(--soft);
		border-radius: 4px;
		font-size: var(--fs-secondary);
	}

	.list {
		flex: 1;
		min-height: 0;
		overflow: auto;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.collection {
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.head {
		display: flex;
		align-items: baseline;
		gap: 6px;
		padding: 4px 8px;
		border-radius: var(--r-row);
		text-align: left;
	}

	/* The head and its `⋯`, side by side; the `⋯` shows on hover or focus. */
	.head-row {
		display: flex;
		align-items: center;
		gap: 2px;
	}

	.head-row .head {
		flex: 1;
		min-width: 0;
	}

	.more {
		flex: none;
		width: 24px;
		height: 24px;
		display: grid;
		place-items: center;
		border-radius: var(--r-row);
		color: var(--muted);
		opacity: 0;
		transition: opacity var(--t-fast) var(--ease);
	}

	.head-row:hover .more,
	.more:focus-visible,
	.broken .more {
		opacity: 1;
	}

	.more:hover {
		background: var(--hover);
		color: var(--ink);
	}

	.hidden {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding-top: 6px;
		border-top: 1px solid var(--soft);
	}

	.hidden-head {
		display: flex;
		gap: 6px;
		padding: 2px 8px;
		color: var(--muted);
		font-size: var(--fs-secondary);
		text-align: left;
	}

	.hidden-row {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 0 4px 0 22px;
		font-size: var(--fs-secondary);
	}

	.hidden-row .label {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.danger {
		color: var(--danger);
	}

	.head:hover,
	.folder:hover,
	.call:hover {
		background: var(--hover);
	}

	.name {
		font-weight: 600;
	}

	.gap {
		flex: 1;
	}

	.count,
	.why {
		color: var(--muted);
		font-size: var(--fs-secondary);
		white-space: nowrap;
	}

	.kind {
		color: var(--agent);
		font-size: var(--fs-secondary);
	}

	.file {
		margin: 0 0 4px;
		padding: 0 8px;
		overflow: hidden;
		color: var(--muted);
		font-size: var(--fs-secondary);
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.broken .name {
		color: var(--danger);
	}

	.why {
		margin: 0;
		padding: 0 8px;
		white-space: normal;
	}

	.folder,
	.call {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		border-radius: var(--r-row);
		text-align: left;
	}

	.folder {
		color: var(--muted);
	}

	.folder.service {
		color: var(--ink);
		font-weight: 600;
	}

	.arrow {
		width: 10px;
		color: var(--muted);
		font-size: 0.8em;
	}

	.call.on {
		background: var(--surface-2);
	}

	.method {
		flex: none;
		width: 52px;
		font-size: var(--fs-secondary);
		font-weight: 600;
	}

	.label {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.none p {
		margin: 4px 8px 8px;
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.empty {
		align-self: flex-start;
		margin: 4px 8px;
		padding: 3px 10px;
		border: 1px dashed var(--soft);
		border-radius: var(--r-button);
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.empty:hover {
		color: var(--ink);
	}

	.found-head {
		margin: 8px 8px 0;
		padding-top: 10px;
		border-top: 1px solid var(--line);
		color: var(--muted);
		font-size: var(--fs-secondary);
		font-weight: normal;
	}

	.collection.found .name {
		font-weight: normal;
	}

	.card {
		display: flex;
		flex-direction: column;
		gap: 4px;
		margin: 0 4px;
		padding: 10px 12px;
		border: 1px dashed var(--soft);
		border-radius: var(--r-field);
		text-align: left;
	}

	button.card:hover {
		background: var(--hover);
	}

	.card-line {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.card .dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--lane-2);
	}

	.card-note {
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.card-actions {
		display: flex;
		justify-content: flex-end;
	}

	.card-actions .empty {
		margin: 0 0 0 6px;
	}

	.history {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 8px;
		border-top: 1px solid var(--line);
		border-radius: var(--r-row);
		color: var(--muted);
	}

	.history:hover,
	.history.on {
		background: var(--hover);
		color: var(--ink);
	}
</style>
