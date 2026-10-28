<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { PRODUCT_NAME } from '$lib/brand';
	import Btn from '$lib/ui/Btn.svelte';
	import { describe, notice } from '$lib/ui/notice.svelte';
	import type { StepCategory } from '$lib/types';
	import * as api from './builder-api';
	import { builder } from './builder.svelte';
	import type { Stage } from './builder-types';
	import HookScript from './HookScript.svelte';
	import { CATEGORIES, categoryOf, HOOK_SAYS } from './order';
	import { MANAGER_LABEL } from './store.svelte';
	import { lefthookSnippet, preCommitSnippet, present, TEMPLATES, wayFor, type Template } from './templates';

	/**
	 * Create hook (FEAT-176): checks teams add to a repository, by kind, each
	 * said in a sentence with why it is worth having, what it needs and the
	 * command it runs. *Add* puts it where the repository's hooks are kept:
	 * Husky's file, Spagitty's blocks, or — for lefthook and pre-commit, whose
	 * configuration is the team's — the lines to paste, copied.
	 */
	const info = $derived(builder.data?.info ?? null);
	const manager = $derived(info?.manager ?? 'git');
	let filter = $state<StepCategory | 'all'>('all');
	let open = $state<string | null>(null);
	let adding = $state<string | null>(null);
	let added = $state<string[]>([]);

	const shown = $derived(TEMPLATES.filter((template) => filter === 'all' || template.category === filter));
	const commands = $derived([
		...(info?.steps ?? []).map((step) => step.command),
		...(builder.draft?.blocks ?? []).map((block) => ('command' in block.action ? block.action.command : ''))
	]);

	/** What a native block needs installed: nothing; Spagitty checks it itself. */
	function needs(template: Template): string | undefined {
		const native = template.block && ['message', 'secrets', 'largeFiles', 'protectBranches'].includes(template.block.kind);
		return wayFor(manager, template) === 'blocks' && native ? undefined : template.needs;
	}

	function label(template: Template): string {
		switch (wayFor(manager, template)) {
			case 'husky':
				return `Add to .husky/${template.hook}`;
			case 'blocks':
				return `Add as a block`;
			case 'lefthook':
				return 'Copy for lefthook.yml';
			case 'preCommit':
				return 'Copy for .pre-commit-config.yaml';
			case 'none':
				return 'Not with blocks';
		}
	}

	async function add(template: Template) {
		const way = wayFor(manager, template);
		if (adding) return;
		if (way === 'lefthook' || way === 'preCommit') {
			const text = way === 'lefthook' ? lefthookSnippet(template) : preCommitSnippet(template);
			try {
				await navigator.clipboard.writeText(text);
				notice.ok(
					'Copied',
					way === 'lefthook'
						? `Paste it into ${info?.config?.path ?? 'lefthook.yml'}, then run lefthook install.`
						: `Paste it under repos: in ${info?.config?.path ?? '.pre-commit-config.yaml'}, then run pre-commit install.`
				);
			} catch (cause) {
				notice.failed('Could not copy', cause);
			}
			open = template.id;
			return;
		}
		if (way === 'blocks') {
			if (builder.readonly) await builder.importManager();
			if (builder.readonly) return;
			builder.addAction(template.block ?? { kind: 'script', command: template.command }, template.hook as Stage, template.title);
			added = [...added, template.id];
			notice.ok('Added as a block', 'Save & install to have git run it.');
			void goto('/hooks');
			return;
		}
		if (way !== 'husky') return;
		adding = template.id;
		try {
			const file = await api.add(template.hook, template.command);
			added = [...added, template.id];
			await builder.reread();
			notice.ok(`Added to ${file}`, 'Commit the file to share it with the team.');
		} catch (cause) {
			notice.failed('Could not add the hook', describe(cause));
		} finally {
			adding = null;
		}
	}
</script>

<div class="create">
	<nav class="kinds pane" aria-label="Kinds">
		<button type="button" class="kind" class:on={filter === 'all'} aria-pressed={filter === 'all'} onclick={() => (filter = 'all')}>
			<span class="dot" aria-hidden="true" style:--tone="var(--ink)"></span>Every kind
			<span class="count">{TEMPLATES.length}</span>
		</button>
		{#each CATEGORIES.filter((category) => TEMPLATES.some((template) => template.category === category.id)) as category (category.id)}
			<button
				type="button"
				class="kind"
				class:on={filter === category.id}
				aria-pressed={filter === category.id}
				style:--tone={category.tone}
				onclick={() => (filter = category.id)}
			>
				<span class="dot" aria-hidden="true"></span>{category.label}
				<span class="count">{TEMPLATES.filter((template) => template.category === category.id).length}</span>
			</button>
		{/each}
		<span class="grow"></span>
		<p class="note">
			{#if manager === 'husky'}
				Husky runs this repository’s hooks: a hook is added to its file in <span class="mono">.husky/</span>.
			{:else if manager === 'git'}
				{PRODUCT_NAME} writes this repository’s hooks: a hook is added as a block, and Save &amp; install puts it in place.
			{:else}
				{MANAGER_LABEL[manager]} runs this repository’s hooks from its configuration, which stays yours: Add copies the
				lines to paste in.
			{/if}
		</p>
	</nav>

	<div class="cards">
		{#each shown as template (template.id)}
			{@const kind = categoryOf(template.category)}
			{@const way = wayFor(manager, template)}
			{@const has = added.includes(template.id) || present(template, commands)}
			<article class="card-t" style:--tone={kind.tone} aria-label={template.title}>
				<header>
					<span class="tag">{kind.label}</span>
					<span class="hook mono" title={HOOK_SAYS[template.hook]}>{template.hook}</span>
				</header>
				<h3>{template.title}</h3>
				<p class="says">{template.says}</p>
				<p class="why note">{template.why}</p>
				{#if needs(template)}<p class="needs note">Needs {needs(template)}.</p>{/if}
				{#if open === template.id}
					{#if way === 'lefthook' || way === 'preCommit'}
						<HookScript
							path={way === 'lefthook' ? 'lefthook.yml' : '.pre-commit-config.yaml'}
							text={way === 'lefthook' ? lefthookSnippet(template) : preCommitSnippet(template)}
						/>
					{:else}
						<HookScript path={`.husky/${template.hook}`} text={template.command} />
					{/if}
				{/if}
				<footer>
					<button type="button" class="link" onclick={() => (open = open === template.id ? null : template.id)}
						>{open === template.id ? 'Hide the code' : 'Show the code'}</button
					>
					<span class="grow"></span>
					{#if has}
						<span class="done note">Runs here</span>
					{:else}
						<Btn
							primary={way === 'husky' || way === 'blocks'}
							disabled={way === 'none' || adding !== null}
							busy={adding === template.id}
							title={way === 'none'
								? `${PRODUCT_NAME}’s blocks run in pre-commit, commit-msg and pre-push only`
								: HOOK_SAYS[template.hook]}
							onclick={() => add(template)}>{label(template)}</Btn
						>
					{/if}
				</footer>
			</article>
		{/each}
	</div>
</div>

<style>
	.create {
		flex: 1;
		min-height: 0;
		display: flex;
		gap: 10px;
	}

	.pane {
		min-height: 0;
		overflow: auto;
		padding: 14px;
		border: 1px solid var(--pane-edge);
		border-radius: var(--r-panel);
		background: var(--surface);
	}

	.kinds {
		flex: 0 0 230px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.kind {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 7px 9px;
		border-radius: var(--r-button);
		text-align: left;
		font-size: var(--fs-secondary);
	}

	.kind:hover {
		background: var(--hover);
	}

	.kind.on {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		font-weight: 600;
	}

	.dot {
		width: 9px;
		height: 9px;
		border-radius: 3px;
		background: var(--tone);
	}

	.count {
		margin-left: auto;
		color: var(--muted);
		font-size: var(--fs-mono);
		font-weight: normal;
	}

	.grow {
		flex: 1;
	}

	.note {
		margin: 0;
		color: var(--muted);
		font-size: var(--fs-secondary);
		line-height: 1.45;
	}

	.cards {
		flex: 1;
		min-width: 0;
		min-height: 0;
		overflow: auto;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		align-content: start;
		gap: 10px;
	}

	.card-t {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px;
		border: 1px solid var(--pane-edge);
		border-top: 3px solid var(--tone);
		border-radius: var(--r-panel);
		background: var(--surface);
		min-width: 0;
	}

	.card-t header {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.tag {
		padding: 1px 8px;
		border-radius: var(--r-pill);
		background: color-mix(in srgb, var(--tone) 16%, transparent);
		font-size: var(--fs-mono);
	}

	.hook {
		color: var(--muted);
		font-size: var(--fs-mono);
	}

	h3 {
		margin: 0;
		font-size: var(--fs-ui);
	}

	.says {
		margin: 0;
		font-size: var(--fs-secondary);
		line-height: 1.45;
	}

	footer {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: auto;
		padding-top: 4px;
	}

	.link {
		padding: 0;
		color: var(--accent);
		font-size: var(--fs-secondary);
	}

	.link:hover {
		text-decoration: underline;
	}

	.done {
		color: var(--ok);
	}

	@media (max-width: 1000px) {
		.kinds {
			flex-basis: 190px;
		}
	}
</style>
