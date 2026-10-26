<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { PRODUCT_NAME } from '$lib/brand';
	import type { Scope } from './api';

	/**
	 * Where the collections on screen are kept (FEAT-170): beside this
	 * repository's code, or in Workbench for every repository. Drawn in
	 * Calls' head and on Import, so what is read and where an import goes is
	 * always on screen. With no repository open there is only Workbench.
	 */
	interface Props {
		scope: Scope;
		/** The open repository's name, or null with none open. */
		repository: string | null;
		onchoose: (scope: Scope) => void;
		/** What the switch is about, for a screen reader. */
		label?: string;
	}

	let { scope, repository, onchoose, label = 'Where collections are kept' }: Props = $props();

	const CHOICES = $derived<{ id: Scope; text: string; title: string; disabled: boolean }[]>([
		{
			id: 'repository',
			text: repository ?? 'Repository',
			title: repository
				? 'Kept in the repository, beside its code, to be committed'
				: 'Open a repository to keep calls beside its code',
			disabled: repository === null
		},
		{
			id: 'global',
			text: PRODUCT_NAME,
			title: `Kept in ${PRODUCT_NAME}, the same with any repository open or none, and never committed`,
			disabled: false
		}
	]);
</script>

<div class="scope" role="radiogroup" aria-label={label}>
	{#each CHOICES as choice (choice.id)}
		<button
			type="button"
			role="radio"
			class="choice"
			class:on={scope === choice.id}
			aria-checked={scope === choice.id}
			title={choice.title}
			disabled={choice.disabled}
			onclick={() => onchoose(choice.id)}>{choice.text}</button
		>
	{/each}
</div>

<style>
	.scope {
		display: flex;
		flex: none;
		padding: 2px;
		border: 1px solid var(--line);
		border-radius: var(--r-pill);
		background: var(--sunken);
	}

	.choice {
		max-width: 160px;
		padding: 3px 12px;
		border-radius: var(--r-pill);
		font-size: var(--fs-secondary);
		color: var(--muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.choice.on {
		background: var(--surface-2);
		color: var(--ink);
		font-weight: 600;
		box-shadow: var(--shadow-1);
	}

	.choice:disabled {
		opacity: 0.45;
		cursor: default;
	}
</style>
