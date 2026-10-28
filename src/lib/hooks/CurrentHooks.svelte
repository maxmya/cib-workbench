<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import Btn from '$lib/ui/Btn.svelte';
	import EmptyState from '$lib/ui/EmptyState.svelte';
	import type { StepCategory } from '$lib/types';
	import BlockEditor from './BlockEditor.svelte';
	import { builder } from './builder.svelte';
	import HookFile from './HookFile.svelte';
	import { allSteps, arrange, CATEGORIES, categoryOf, counts, HOOK_SAYS, ordinal, type Shown } from './order';
	import { MANAGER_LABEL } from './store.svelte';

	/**
	 * Current hooks (FEAT-176): what this repository's hooks run, in the
	 * order they run.
	 *
	 * Every step every hook runs, read from the hook files or the manager's
	 * configuration, laid out by the moment git runs it — commit, push,
	 * merge — and numbered straight through each moment, so the first thing
	 * a commit runs is *1*. Each step is sorted by what it does, and the kinds
	 * filter the list. A step chosen shows the file it is written in, with
	 * its line marked; one of Spagitty's own blocks shows its settings.
	 */
	const data = $derived(builder.data);
	const info = $derived(data?.info ?? null);
	const blocks = $derived(info?.manager === 'git' || builder.imported);
	const steps = $derived(allSteps(info, builder.draft, blocks));
	let filter = $state<StepCategory | 'all'>('all');
	const laid = $derived(arrange(steps, filter));
	const tally = $derived(counts(steps));
	let chosen = $state<string | null>(null);
	const flat = $derived(laid.flatMap((moment) => moment.hooks.flatMap((hook) => hook.steps)));
	const step = $derived(flat.find((each) => each.key === chosen) ?? flat[0] ?? null);

	/** The text of the file a step is written in. */
	const file = $derived.by(() => {
		if (!step || step.block || !info) return null;
		const hook = info.hooks.find((each) => each.path === step.file);
		if (hook) return { path: hook.path, text: hook.script, truncated: hook.truncated };
		if (info.config?.path === step.file) return { path: info.config.path, text: info.config.text, truncated: false };
		return null;
	});

	function pick(next: Shown) {
		chosen = next.key;
		if (next.block) builder.select(next.block);
	}

	const where = $derived(
		info
			? info.manager === 'git'
				? blocks && (builder.draft?.blocks.length ?? 0) > 0
					? `Run by git, with ${builder.draft?.shared ? 'the blocks shared in .spagitty/hooks.toml' : 'your personal blocks'}`
					: `Run by git, from ${info.dir}`
				: `Run by ${MANAGER_LABEL[info.manager]}${info.config ? `, from ${info.config.path}` : ` from ${info.dir}`}`
			: ''
	);
</script>

<div class="current">
	<div class="bar">
		<div class="kinds" role="radiogroup" aria-label="Show">
			<button
				type="button"
				role="radio"
				class="kind"
				class:on={filter === 'all'}
				aria-checked={filter === 'all'}
				onclick={() => (filter = 'all')}>All <span class="count">{tally.all}</span></button
			>
			{#each CATEGORIES.filter((category) => tally[category.id] > 0) as category (category.id)}
				<button
					type="button"
					role="radio"
					class="kind"
					class:on={filter === category.id}
					aria-checked={filter === category.id}
					style:--tone={category.tone}
					onclick={() => (filter = category.id)}
					><span class="dot" aria-hidden="true"></span>{category.label} <span class="count">{tally[category.id]}</span></button
				>
			{/each}
		</div>
		<span class="grow"></span>
		{#if where}<span class="where note" title={info?.dir}>{where}</span>{/if}
	</div>

	{#if steps.length === 0}
		<div class="pane empty">
			<EmptyState message="No hooks yet" />
			<Btn primary onclick={() => goto('/hooks/create')}>Create a hook</Btn>
		</div>
	{:else}
		<div class="split">
			<nav class="pane order" aria-label="What runs, in order">
				{#each laid as moment (moment.moment)}
					<section class="moment">
						<h3>{moment.label}</h3>
						{#each moment.hooks as hook (hook.hook)}
							<div class="hook">
								<div class="hook-head" title={hook.says}>
									<span class="mono">{hook.hook}</span>
									<span class="note says">{hook.says}</span>
								</div>
								<ol>
									{#each hook.steps as each (each.key)}
										{@const kind = categoryOf(each.category)}
										<li>
											<button
												type="button"
												class="step"
												class:on={step?.key === each.key}
												class:off={each.off}
												style:--tone={kind.tone}
												aria-current={step?.key === each.key ? 'true' : undefined}
												onclick={() => pick(each)}
											>
												<span class="place mono" title={each.off ? 'Switched off' : `Runs ${ordinal(each.place)}`}
													>{each.off ? '–' : each.place}</span
												>
												<span class="text">
													<span class="name">{each.name}</span>
													<span class="command mono">{each.command}</span>
												</span>
												<span class="tag" title={kind.label}>{kind.label}</span>
											</button>
										</li>
									{/each}
								</ol>
							</div>
						{/each}
					</section>
				{:else}
					<p class="note pad">No step of this kind runs here.</p>
				{/each}
			</nav>

			<section class="pane detail" aria-label="The step chosen">
				{#if step}
					{@const kind = categoryOf(step.category)}
					<header class="detail-head">
						<span class="big-place mono" style:--tone={kind.tone}>{step.off ? '–' : step.place}</span>
						<div class="titles">
							<h2>{step.name}</h2>
							<p class="note">
								{step.off ? 'Switched off' : `Runs ${ordinal(step.place)}`} · in <span class="mono">{step.hook}</span>{step.together
									? ', at the same time as the hook’s other steps'
									: ''}. {HOOK_SAYS[step.hook] ?? ''}
							</p>
						</div>
						<span class="tag" style:--tone={kind.tone}>{kind.label}</span>
					</header>
					<pre class="run mono" aria-label="What it runs">{step.command}</pre>
					{#if step.block}
						<BlockEditor />
					{:else if file}
						<HookFile path={file.path} text={file.text} truncated={file.truncated} mark={step.line} />
					{:else}
						<p class="note">{step.file}</p>
					{/if}
				{/if}
			</section>
		</div>
	{/if}
</div>

<style>
	.current {
		flex: 1;
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.bar {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
	}

	.kinds {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.kind {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 3px 11px;
		border: 1px solid var(--soft);
		border-radius: var(--r-pill);
		background: var(--surface-veil);
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.kind:hover {
		color: var(--ink);
	}

	.kind.on {
		border-color: color-mix(in srgb, var(--accent) 60%, transparent);
		background: color-mix(in srgb, var(--accent) 14%, var(--surface));
		color: var(--ink);
	}

	.dot {
		width: 8px;
		height: 8px;
		border-radius: 3px;
		background: var(--tone);
	}

	.count {
		color: var(--muted);
		font-size: var(--fs-mono);
	}

	.grow {
		flex: 1;
	}

	.where {
		max-width: 46ch;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.note {
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.pad {
		padding: 6px;
	}

	.pane {
		min-height: 0;
		overflow: auto;
		padding: 14px;
		border: 1px solid var(--pane-edge);
		border-radius: var(--r-panel);
		background: var(--surface);
	}

	.empty {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
	}

	.split {
		flex: 1;
		min-height: 0;
		display: flex;
		gap: 10px;
	}

	.order {
		flex: 0 0 380px;
		display: flex;
		flex-direction: column;
		gap: 18px;
	}

	.moment h3 {
		margin: 0 0 8px;
		color: var(--muted);
		font-size: var(--fs-secondary);
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}

	.hook + .hook {
		margin-top: 10px;
	}

	.hook-head {
		display: flex;
		align-items: baseline;
		gap: 8px;
		margin-bottom: 4px;
		min-width: 0;
	}

	.hook-head .mono {
		font-weight: 600;
		font-size: var(--fs-mono);
	}

	.says {
		min-width: 0;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	ol {
		margin: 0;
		padding: 0 0 0 10px;
		list-style: none;
		border-left: 2px solid var(--soft);
	}

	.step {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 8px;
		border-radius: var(--r-button);
		text-align: left;
	}

	.step:hover {
		background: var(--hover);
	}

	.step.on {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
	}

	.step.off {
		opacity: 0.5;
	}

	.place {
		flex: none;
		display: grid;
		place-items: center;
		width: 22px;
		height: 22px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--tone) 22%, transparent);
		color: var(--ink);
		font-size: var(--fs-mono);
		font-weight: 600;
	}

	.text {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 1px;
	}

	.name {
		font-size: var(--fs-secondary);
		font-weight: 600;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.command {
		color: var(--muted);
		font-size: var(--fs-mono);
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}

	.tag {
		flex: none;
		padding: 1px 8px;
		border-radius: var(--r-pill);
		background: color-mix(in srgb, var(--tone) 16%, transparent);
		color: var(--ink);
		font-size: var(--fs-mono);
		white-space: nowrap;
	}

	.detail {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.detail-head {
		display: flex;
		align-items: flex-start;
		gap: 12px;
	}

	.big-place {
		flex: none;
		display: grid;
		place-items: center;
		width: 34px;
		height: 34px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--tone) 24%, transparent);
		font-weight: 600;
	}

	.titles {
		flex: 1;
		min-width: 0;
	}

	h2 {
		margin: 0;
		font-size: var(--fs-ui);
	}

	.titles p {
		margin: 2px 0 0;
		line-height: 1.45;
	}

	.run {
		margin: 0;
		padding: 10px 12px;
		border-radius: var(--r-field);
		background: var(--sunken);
		font-size: var(--fs-mono);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		user-select: text;
	}

	@media (max-width: 1000px) {
		.order {
			flex-basis: 300px;
		}
	}
</style>
