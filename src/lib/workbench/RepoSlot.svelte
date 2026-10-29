<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/api';
	import { repo } from '$lib/repo.svelte';
	import { shortcut } from '$lib/platform';
	import Icon from '$lib/ui/Icon.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import { loader } from './loader.svelte';

	/**
	 * The title bar's repository slot (FEAT-184), in place of the tab strip:
	 * *Load repository* with nothing loaded; the loaded one, *Switch* and
	 * *Unload* otherwise.
	 */
	let upstream = $state<{ ahead: number; behind: number } | null>(null);

	// Ahead and behind, read again when the repository or its refs move.
	$effect(() => {
		void repo.generation;
		void repo.info?.head.id;
		if (!repo.info || !api.inTauri()) {
			upstream = null;
			return;
		}
		void api
			.homeOverview()
			.then((overview) => (upstream = overview.upstream))
			.catch(() => (upstream = null));
	});

	let keys = $state('Ctrl+O');
	onMount(() => (keys = shortcut('O')));

	const info = $derived(repo.info);
</script>

<div class="slot" data-tauri-drag-region>
	{#if info}
		<div class="loaded">
			<span class="chip" title={info.path}>
				<span class="mono repo-name">{info.name}</span>
				{#if info.head.branch}<span class="mono branch">{info.head.branch}</span>{:else if info.head.short}<span
						class="mono branch">{info.head.short}</span
					>{/if}
				{#if upstream}<span class="mono counts">↑{upstream.ahead} ↓{upstream.behind}</span>{/if}
			</span>
			<button class="switch" title="Switch repository ({keys})" onclick={() => loader.show()} ondblclick={(e) => e.stopPropagation()}>Switch</button>
			<button
				class="unload"
				aria-label="Unload {info.name}"
				title="Unload {info.name}"
				onclick={() => loader.unload()}
				ondblclick={(e) => e.stopPropagation()}><Icon name="close" size="0.85em" /></button
			>
		</div>
	{:else if repo.busy || loader.busy}
		<span class="empty"><Loader size="inline" label="Loading…" /></span>
	{:else}
		<button class="empty" onclick={() => loader.show()} ondblclick={(e) => e.stopPropagation()}>
			<span class="muted">No repository loaded</span>
			<span class="go">Load repository</span>
			<kbd class="kbd">{keys}</kbd>
		</button>
	{/if}
</div>

<style>
	.slot {
		display: flex;
		align-items: center;
		min-width: 0;
		height: 100%;
	}

	.empty {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		height: 26px;
		padding: 0 6px 0 12px;
		border-radius: var(--r-pill);
		border: 1px dashed var(--line);
		font-size: var(--fs-secondary);
		white-space: nowrap;
	}

	button.empty:hover {
		background: var(--hover);
		border-color: var(--accent);
	}

	.muted {
		color: var(--muted);
	}

	.go {
		color: var(--accent);
		font-weight: 600;
	}

	.loaded {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		height: 26px;
		padding: 0 10px;
		border-radius: var(--r-pill);
		background: var(--surface);
		border: 1px solid color-mix(in srgb, var(--line) 55%, transparent);
	}

	.repo-name {
		font-weight: 600;
		color: var(--ink);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.branch,
	.counts {
		color: var(--muted);
		white-space: nowrap;
	}

	.switch {
		height: 26px;
		padding: 0 10px;
		border-radius: var(--r-pill);
		font-size: var(--fs-secondary);
		color: var(--ink);
	}

	.switch:hover,
	.unload:hover {
		background: var(--hover);
	}

	.unload {
		width: 26px;
		height: 26px;
		display: grid;
		place-items: center;
		border-radius: var(--r-pill);
		color: var(--muted);
	}
</style>
