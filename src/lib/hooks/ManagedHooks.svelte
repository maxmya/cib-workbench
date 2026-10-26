<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { PRODUCT_NAME } from '$lib/brand';
	import Btn from '$lib/ui/Btn.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import { describe } from '$lib/ui/notice.svelte';
	import * as api from './builder-api';
	import { builder } from './builder.svelte';
	import HookScript from './HookScript.svelte';
	import { MANAGER_LABEL } from './store.svelte';

	/**
	 * What a hook manager runs, read here (FEAT-171).
	 *
	 * With Husky, lefthook or pre-commit in charge, the builder could only say
	 * it was read-only, and what the hooks actually do was nowhere on the
	 * screen. Now every file the manager runs is shown, coloured, with an
	 * agent's explanation a click away, and can be edited in place: the file
	 * is the manager's, in the repository, and is committed like any other.
	 * *Import* stays the way to rebuild them as blocks.
	 */
	const info = $derived(builder.data?.info ?? null);
	const manager = $derived(info ? MANAGER_LABEL[info.manager] : '');

	/** Every file that says what runs: the hooks' own scripts, then the manager's configuration. */
	const files = $derived([
		...(info?.config ? [{ key: info.config.path, title: 'Configuration', path: info.config.path, text: info.config.text, truncated: false }] : []),
		...(info?.hooks ?? []).map((hook) => ({
			key: hook.path,
			title: hook.name,
			path: hook.path,
			text: hook.script,
			truncated: hook.truncated
		}))
	]);

	let agents = $state<api.Askable[] | null>(null);
	let agent = $state<string | null>(null);
	let explaining = $state<string | null>(null);
	let explained = $state<Record<string, api.Explanation | { error: string }>>({});
	let editing = $state<{ path: string; text: string } | null>(null);
	let writing = $state(false);
	let writeError = $state<string | null>(null);

	async function readAgents() {
		if (agents) return;
		try {
			agents = await api.agents();
			agent = agents[0]?.id ?? null;
		} catch {
			agents = [];
		}
	}

	async function explain(path: string) {
		if (explaining) return;
		explaining = path;
		await readAgents();
		try {
			explained[path] = await api.explain(path, agent);
		} catch (cause) {
			explained[path] = { error: describe(cause) };
		} finally {
			explaining = null;
		}
	}

	async function save() {
		if (!editing || writing) return;
		writing = true;
		writeError = null;
		try {
			await api.write(editing.path, editing.text);
			const path = editing.path;
			editing = null;
			delete explained[path];
			await builder.reread();
		} catch (cause) {
			writeError = describe(cause);
		} finally {
			writing = false;
		}
	}
</script>

<section class="managed" aria-label="What runs">
	<header class="intro">
		<h3>What {manager} runs</h3>
		<p class="note">
			{#if info?.manager === 'husky'}
				Husky keeps its hooks in <span class="mono">.husky/</span>, committed with the code, so everyone who
				clones the repository runs them.
			{:else}
				{manager} reads its steps from its configuration, committed with the code, so everyone who clones the repository
				runs them.
			{/if}
			They are shown here as they are. Edit one and the file in the repository changes; commit it to share the change.
			To build hooks from blocks in {PRODUCT_NAME} instead, use Import.
		</p>
		{#if agents && agents.length > 1}
			<label class="agent note"
				>Explain with
				<select aria-label="The agent that explains" bind:value={agent}>
					{#each agents as one (one.id)}<option value={one.id}>{one.name}</option>{/each}
				</select></label
			>
		{/if}
	</header>

	{#each files as file (file.key)}
		<article class="file">
			<div class="file-head">
				<strong class="mono">{file.title}</strong>
				<span class="path mono">{file.path}</span>
				<span class="grow"></span>
				{#if editing?.path !== file.path}
					<Btn
						busy={explaining === file.path}
						disabled={explaining !== null && explaining !== file.path}
						title="An agent reads this file and says what it does. It runs nothing."
						onclick={() => explain(file.path)}>Explain</Btn
					>
					<Btn
						disabled={file.truncated || editing !== null}
						title={file.truncated ? 'Too long to edit here; open it in your editor' : 'Edit the file in the repository'}
						onclick={() => {
							writeError = null;
							editing = { path: file.path, text: file.text };
						}}>Edit</Btn
					>
				{/if}
			</div>

			{#if editing?.path === file.path}
				<textarea class="mono editor" aria-label={`Edit ${file.path}`} spellcheck="false" bind:value={editing.text}
				></textarea>
				{#if writeError}<p class="error" role="alert">{writeError}</p>{/if}
				<div class="actions">
					<Btn disabled={writing} onclick={() => (editing = null)}>Cancel</Btn>
					<Btn primary quiet busy={writing} disabled={editing.text === file.text} onclick={save}>Save</Btn>
				</div>
			{:else}
				<HookScript path={file.path} text={file.text} truncated={file.truncated} />
			{/if}

			{#if explaining === file.path}
				<Loader size="inline" label="Asking the agent" />
			{:else if explained[file.path]}
				{@const said = explained[file.path]}
				{#if 'error' in said}
					<p class="error" role="alert">{said.error}</p>
				{:else}
					<div class="explained" role="note">
						<span class="who note">{said.agent} says</span>
						<p>{said.text}</p>
					</div>
				{/if}
			{/if}
		</article>
	{:else}
		<p class="note">{manager} is set up, but no hook it runs was found.</p>
	{/each}
</section>

<style>
	.managed {
		flex: 1;
		min-width: 0;
		min-height: 0;
		overflow: auto;
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 16px;
		border: 1px solid var(--pane-edge);
		border-radius: var(--r-panel);
		background: var(--surface);
	}

	.intro h3 {
		margin: 0 0 6px;
		font-size: var(--fs-ui);
	}

	.intro .note {
		margin: 0;
		max-width: 80ch;
		line-height: 1.5;
	}

	.agent {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		margin-top: 8px;
	}

	.file {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.file-head {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.path {
		color: var(--muted);
		font-size: var(--fs-mono);
	}

	.grow {
		flex: 1;
	}

	.editor {
		min-height: 220px;
		padding: 10px 14px;
		border: 1px solid var(--ring);
		border-radius: var(--r-button);
		background: var(--sunken);
		color: var(--ink);
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

	.note {
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.error {
		margin: 0;
		color: var(--danger);
		font-size: var(--fs-secondary);
	}
</style>
