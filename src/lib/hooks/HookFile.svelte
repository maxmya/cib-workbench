<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import Btn from '$lib/ui/Btn.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import { describe } from '$lib/ui/notice.svelte';
	import * as api from './builder-api';
	import { builder } from './builder.svelte';
	import HookScript from './HookScript.svelte';

	/**
	 * One hook file, coloured, with the step's line marked (FEAT-176): what
	 * FEAT-171 showed for every file at once, for the one the chosen step is
	 * written in. *Explain* asks an agent the repository allows what the file
	 * does; *Edit* changes it in place, and committing shares the change.
	 */
	interface Props {
		path: string;
		text: string;
		truncated?: boolean;
		mark?: number | null;
	}

	let { path, text, truncated = false, mark = null }: Props = $props();

	let agents = $state<api.Askable[] | null>(null);
	let agent = $state<string | null>(null);
	let explaining = $state(false);
	let explained = $state<Record<string, api.Explanation | { error: string }>>({});
	let editing = $state<string | null>(null);
	let writing = $state(false);
	let writeError = $state<string | null>(null);

	// Another file: whatever was being edited was this one's.
	$effect(() => {
		void path;
		editing = null;
		writeError = null;
	});

	async function readAgents() {
		if (agents) return;
		try {
			agents = await api.agents();
			agent = agents[0]?.id ?? null;
		} catch {
			agents = [];
		}
	}

	async function explain() {
		if (explaining) return;
		explaining = true;
		const asked = path;
		await readAgents();
		try {
			explained[asked] = await api.explain(asked, agent);
		} catch (cause) {
			explained[asked] = { error: describe(cause) };
		} finally {
			explaining = false;
		}
	}

	async function save() {
		if (editing === null || writing) return;
		writing = true;
		writeError = null;
		try {
			await api.write(path, editing);
			editing = null;
			delete explained[path];
			await builder.reread();
		} catch (cause) {
			writeError = describe(cause);
		} finally {
			writing = false;
		}
	}

	const said = $derived(explained[path]);
</script>

<article class="file">
	<div class="file-head">
		<span class="path mono">{path}</span>
		<span class="grow"></span>
		{#if agents && agents.length > 1 && editing === null}
			<select class="agent" aria-label="The agent that explains" bind:value={agent}>
				{#each agents as one (one.id)}<option value={one.id}>{one.name}</option>{/each}
			</select>
		{/if}
		{#if editing === null}
			<Btn busy={explaining} title="An agent reads this file and says what it does. It runs nothing." onclick={explain}
				>Explain</Btn
			>
			<Btn
				disabled={truncated}
				title={truncated ? 'Too long to edit here; open it in your editor' : 'Edit the file in the repository'}
				onclick={() => {
					writeError = null;
					editing = text;
				}}>Edit</Btn
			>
		{/if}
	</div>

	{#if editing !== null}
		<textarea class="mono editor" aria-label={`Edit ${path}`} spellcheck="false" bind:value={editing}></textarea>
		{#if writeError}<p class="error" role="alert">{writeError}</p>{/if}
		<div class="actions">
			<Btn disabled={writing} onclick={() => (editing = null)}>Cancel</Btn>
			<Btn primary quiet busy={writing} disabled={editing === text} onclick={save}>Save</Btn>
		</div>
	{:else}
		<HookScript {path} {text} {truncated} {mark} />
	{/if}

	{#if explaining}
		<Loader size="inline" label="Asking the agent" />
	{:else if said}
		{#if 'error' in said}
			<p class="error" role="alert">{said.error}</p>
		{:else}
			<div class="explained" role="note">
				<span class="who">{said.agent} says</span>
				<p>{said.text}</p>
			</div>
		{/if}
	{/if}
</article>

<style>
	.file {
		display: flex;
		flex-direction: column;
		gap: 8px;
		min-height: 0;
	}

	.file-head {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.path {
		color: var(--muted);
		font-size: var(--fs-mono);
	}

	.grow {
		flex: 1;
	}

	.agent {
		max-width: 160px;
		font-size: var(--fs-secondary);
	}

	.editor {
		min-height: 240px;
		padding: 10px 14px;
		border-radius: var(--r-button);
		font-size: var(--fs-mono);
		line-height: 1.55;
		resize: vertical;
		tab-size: 4;
	}

	.actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.explained {
		padding: 10px 14px;
		border-left: 3px solid var(--accent);
		border-radius: var(--r-button);
		background: color-mix(in srgb, var(--accent) 7%, transparent);
	}

	.explained p {
		margin: 4px 0 0;
		white-space: pre-wrap;
		line-height: 1.5;
	}

	.who {
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.error {
		margin: 0;
		color: var(--danger);
		font-size: var(--fs-secondary);
	}
</style>
