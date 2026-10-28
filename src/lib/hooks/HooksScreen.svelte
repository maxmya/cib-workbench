<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import Btn from '$lib/ui/Btn.svelte';
	import EmptyState from '$lib/ui/EmptyState.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import CreateHook from './CreateHook.svelte';
	import CurrentHooks from './CurrentHooks.svelte';
	import HookRuns from './HookRuns.svelte';
	import { builder } from './builder.svelte';
	import type { Stage } from './builder-types';
	import { MANAGER_LABEL } from './store.svelte';

	/**
	 * Hooks (1X, FEAT-176): two tabs. *Current hooks* is what this
	 * repository's hooks run, in the order they run, sorted by kind; *Create
	 * hook* is checks to add, explained. Runs — what they printed each time —
	 * is a button away, not a third tab.
	 */
	let { mode }: { mode: 'current' | 'create' | 'runs' } = $props();
	const draft = $derived(builder.draft);
	const data = $derived(builder.data);
	/** Spagitty writes the hooks from blocks: its controls are wanted. */
	const blocks = $derived(data !== null && !builder.readonly && (draft?.blocks.length ?? 0) > 0);
	let dryStage = $state<Stage>('pre-commit');

	async function dryRun() {
		await builder.dryRun(dryStage, () => {
			void goto('/hooks/runs');
		});
	}
</script>

<div class="screen">
	<header class="head">
		<span class="title">Hooks</span>
		{#if mode === 'runs'}
			<nav class="crumbs" aria-label="Where you are">
				<a href="/hooks">Hooks</a><span class="sep" aria-hidden="true">›</span><span>Runs</span>
			</nav>
		{:else}
			<nav class="tabs" aria-label="Hooks views">
				<a href="/hooks" class:on={mode === 'current'} aria-current={mode === 'current' ? 'page' : undefined}>Current hooks</a>
				<a href="/hooks/create" class:on={mode === 'create'} aria-current={mode === 'create' ? 'page' : undefined}
					>Create hook</a
				>
			</nav>
		{/if}
		{#if builder.dirty}<span class="unsaved">Unsaved changes</span>{/if}
		<span class="grow"></span>
		{#if mode === 'current' && draft && blocks}
			<label class="share" title="Shared: kept in .spagitty/hooks.toml and committed. Personal: kept for you alone."
				><input
					type="checkbox"
					role="switch"
					checked={draft.shared}
					disabled={builder.saving}
					onchange={(event) => builder.share(event.currentTarget.checked)}
				/> Share with the repository</label
			>
			<select
				aria-label="Dry run stage"
				value={dryStage}
				onchange={(event) => {
					dryStage = event.currentTarget.value as Stage;
				}}><option value="pre-commit">pre-commit</option><option value="pre-push">pre-push</option></select
			>
			<Btn disabled={builder.saving} busy={builder.running} onclick={dryRun}>Dry run on staged</Btn>
			<Btn primary quiet busy={builder.saving} disabled={builder.running} onclick={() => builder.save()}>Save &amp; install</Btn>
		{/if}
		{#if data && data.info.manager !== 'git' && !builder.imported && mode === 'current'}
			<Btn
				title={`Have ${MANAGER_LABEL.git} run these steps from blocks you edit here, in place of ${MANAGER_LABEL[data.info.manager]}`}
				onclick={() => builder.importManager()}>Import as blocks</Btn
			>
		{/if}
		{#if mode === 'runs'}
			<Btn onclick={() => builder.refreshRuns(true)}>Refresh</Btn>
		{:else if data}
			<Btn title="Every time the hooks ran, and what each step printed" onclick={() => goto('/hooks/runs')}
				>Runs{data.runs.length ? ` · ${data.runs.length}` : ''}</Btn
			>
		{/if}
	</header>
	{#each Object.entries(builder.errors).filter(([name, value]) => name !== 'matches' && value) as [name, error] (name)}<p
			class="error"
			role="alert"
		>
			{error}
		</p>{/each}
	{#if builder.loading}<Loader label="Reading hooks…" />
	{:else if !data || !draft}<EmptyState message={builder.errors.load || 'Open a repository to see and create its hooks.'} />
	{:else}
		{#if blocks && !data.installed}
			<div class="banner">
				<span>These blocks are saved but not installed for this clone, so git does not run them yet.</span><Btn
					disabled={builder.saving}
					onclick={() => builder.save()}>Install</Btn
				>
			</div>
		{/if}
		<div class="body">
			{#if mode === 'runs'}<HookRuns />
			{:else if mode === 'create'}<CreateHook />
			{:else}<CurrentHooks />{/if}
		</div>
	{/if}
</div>

<style>
	.screen {
		flex: 1;
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px;
		padding: 12px 16px;
		flex: none;
	}

	.title {
		font-size: var(--fs-title);
		margin-right: 5px;
	}

	.tabs {
		display: flex;
		padding: 3px;
		border: 1px solid var(--line);
		border-radius: var(--r-pill);
		background: var(--sunken);
	}

	.tabs a {
		padding: 4px 14px;
		color: var(--muted);
		border-radius: var(--r-pill);
		text-decoration: none;
	}

	.tabs a.on {
		background: var(--surface-2);
		color: var(--ink);
		box-shadow: var(--shadow-1);
	}

	.crumbs {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--muted);
	}

	.crumbs a {
		color: var(--accent);
		text-decoration: none;
	}

	.grow {
		flex: 1;
	}

	.unsaved {
		color: var(--warn);
		font-size: var(--fs-secondary);
	}

	.share {
		display: flex;
		align-items: center;
		gap: 7px;
		white-space: nowrap;
		font-size: var(--fs-secondary);
	}

	.head select {
		max-width: 120px;
		font-size: var(--fs-secondary);
	}

	.banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin: 0 12px 10px;
		padding: 10px 14px;
		border: 1px solid var(--line);
		border-radius: var(--r-button);
		font-size: var(--fs-secondary);
		color: var(--muted);
	}

	.body {
		display: flex;
		flex: 1;
		min-height: 0;
		gap: 10px;
		padding: 0 12px 12px;
	}

	.error {
		color: var(--danger);
		margin: 0;
		padding: 0 16px 8px;
	}
</style>
