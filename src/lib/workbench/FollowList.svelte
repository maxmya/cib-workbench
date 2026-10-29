<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { age } from '$lib/home/needs';
	import Btn from '$lib/ui/Btn.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import { ProjectList } from './projects.svelte';
	import { followedOf } from './state.svelte';
	import type { Followed, Project } from './types';

	/**
	 * Which CIB GitLab projects Home follows (FEAT-182): onboarding step 4,
	 * and Settings › Home. Each row is a toggle; *Start empty* clears them.
	 */
	interface Props {
		selected: Followed[];
		onchange: (next: Followed[]) => void;
		/** At most this many. */
		cap?: number;
	}

	let { selected, onchange, cap = 50 }: Props = $props();

	const list = new ProjectList();
	onMount(() => {
		void list.reload();
		return () => list.dispose();
	});

	const ids = $derived(new Set(selected.map((f) => f.id)));
	const full = $derived(selected.length >= cap);
	const now = $derived.by(() => {
		void list.projects;
		return Date.now();
	});

	function toggle(project: Project) {
		onchange(
			ids.has(project.id) ? selected.filter((f) => f.id !== project.id) : [...selected, followedOf(project)]
		);
	}

	const count = $derived(
		selected.length === 0 ? 'None' : selected.length === 1 ? '1 selected' : `${selected.length} selected`
	);
</script>

<div class="follow">
	<div class="head">
		<h3>Repositories to follow</h3>
		<span class="note">{count}{full ? ` · at most ${cap}` : ''}</span>
	</div>
	<div class="tools">
		<label class="visually-hidden" for="follow-search">Find a project</label>
		<input
			id="follow-search"
			type="search"
			placeholder="Find a project on CIB GitLab"
			value={list.query}
			oninput={(event) => list.search((event.currentTarget as HTMLInputElement).value)}
		/>
		<Btn disabled={selected.length === 0} onclick={() => onchange([])}>Start empty</Btn>
	</div>

	{#if list.error}<p class="error" role="alert">{list.error}</p>{/if}

	<div class="rows" aria-busy={list.loading}>
		{#each list.projects as project (project.id)}
			{@const on = ids.has(project.id)}
			<button
				type="button"
				class="row"
				class:on
				aria-pressed={on}
				disabled={!on && full}
				onclick={() => toggle(project)}
			>
				<span class="box" aria-hidden="true">{#if on}✓{/if}</span>
				<span class="what">
					<span class="path mono">{project.path}</span>
					{#if project.description}<span class="note line">{project.description}</span>{/if}
				</span>
				{#if project.lastActivity}
					<span class="note when">{age(project.lastActivity * 1000, now)}</span>
				{/if}
			</button>
		{:else}
			{#if list.loading}
				<Loader label="Reading CIB GitLab…" />
			{:else if !list.error}
				<p class="note">{list.query ? `No project matches “${list.query}”.` : 'No projects.'}</p>
			{/if}
		{/each}
		{#if list.more}
			<Btn quiet busy={list.loading} onclick={() => list.next()}>More</Btn>
		{/if}
	</div>
</div>

<style>
	.follow {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
	}

	.head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}

	h3 {
		margin: 0;
		font-size: var(--fs-ui);
		font-weight: 600;
	}

	.tools {
		display: flex;
		gap: 10px;
	}

	.tools input {
		flex: 1;
		min-width: 0;
	}

	.rows {
		display: flex;
		flex-direction: column;
		gap: 4px;
		max-height: 420px;
		overflow-y: auto;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		min-height: 44px;
		padding: 6px 12px;
		border-radius: var(--r-row);
		border: 1px solid transparent;
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.row:hover:not(:disabled) {
		background: var(--hover);
	}

	.row.on {
		background: var(--accent-soft);
		border-color: color-mix(in srgb, var(--accent) 45%, transparent);
	}

	.row:disabled {
		opacity: 0.5;
		cursor: default;
	}

	.box {
		flex: none;
		width: 18px;
		height: 18px;
		display: grid;
		place-items: center;
		border-radius: var(--r-row);
		border: 1.5px solid var(--line);
		font-size: var(--fs-mono);
		color: var(--on-primary);
	}

	.row.on .box {
		background: var(--primary);
		border-color: var(--accent);
	}

	.what {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.line,
	.path {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.when {
		white-space: nowrap;
	}

	.error {
		margin: 0;
		color: var(--danger);
		font-size: var(--fs-secondary);
	}

	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
	}
</style>
