<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { inHomeOrder, moveCardBy, type HomeLayout } from '$lib/home/layout';
	import Icon from '$lib/ui/Icon.svelte';
	import { WIDE_HOME_CARDS, type WideCard } from './cards';

	/**
	 * Home's sections, top to bottom (FEAT-180, FEAT-183): each with up and
	 * down, and a switch. It edits `HomeLayout` (FEAT-148), so Settings › Home,
	 * onboarding and *Customise Home* change one thing.
	 */
	interface Props {
		layout: HomeLayout;
		onchange: (next: HomeLayout) => void;
	}

	let { layout, onchange }: Props = $props();

	const cards = $derived(inHomeOrder(layout.order, WIDE_HOME_CARDS) as WideCard[]);

	function move(id: string, delta: number) {
		onchange({ ...layout, order: moveCardBy(id, delta, layout.order, WIDE_HOME_CARDS) });
	}

	function flip(id: string) {
		const hidden = layout.hidden.includes(id)
			? layout.hidden.filter((other) => other !== id)
			: [...layout.hidden, id];
		onchange({ ...layout, hidden });
	}
</script>

<div class="sections">
	<div class="head">
		<h3>Home sections</h3>
		<span class="note">Top to bottom</span>
	</div>
	<ol aria-label="Home sections, in order">
		{#each cards as card, index (card.id)}
			{@const on = !layout.hidden.includes(card.id)}
			<li class:off={!on}>
				<span class="n mono">{index + 1}</span>
				<span class="what">
					<span class="name">{card.label}</span>
					<span class="note">{card.description}</span>
				</span>
				<button
					type="button"
					class="step"
					aria-label="Move {card.label} up"
					disabled={index === 0}
					onclick={() => move(card.id, -1)}><Icon name="chevron-up" size="1em" /></button
				>
				<button
					type="button"
					class="step"
					aria-label="Move {card.label} down"
					disabled={index === cards.length - 1}
					onclick={() => move(card.id, 1)}><Icon name="chevron-down" size="1em" /></button
				>
				<button
					type="button"
					class="switch"
					role="switch"
					aria-checked={on}
					aria-label="Show {card.label} on Home"
					onclick={() => flip(card.id)}><span class="knob" aria-hidden="true"></span></button
				>
			</li>
		{/each}
	</ol>
</div>

<style>
	.sections {
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

	ol {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	li {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 46px;
		padding: 4px 8px 4px 12px;
		border-radius: var(--r-row);
		background: var(--surface-2);
	}

	li.off {
		opacity: 0.6;
	}

	.n {
		width: 18px;
		color: var(--muted);
	}

	.what {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.name {
		font-weight: 600;
	}

	.step {
		width: 30px;
		height: 30px;
		display: grid;
		place-items: center;
		border-radius: var(--r-button);
		border: 1px solid var(--line);
		color: var(--ink);
	}

	.step:disabled {
		color: var(--muted);
		opacity: 0.4;
	}

	.step:hover:not(:disabled) {
		background: var(--hover);
	}

	.switch {
		position: relative;
		width: 40px;
		height: 24px;
		margin-left: 4px;
		border-radius: var(--r-pill);
		background: var(--soft);
		flex: none;
	}

	.switch[aria-checked='true'] {
		background: var(--primary);
	}

	.knob {
		position: absolute;
		top: 3px;
		left: 3px;
		width: 18px;
		height: 18px;
		border-radius: 50%;
		background: var(--on-primary);
		transition: left var(--t-fast) var(--ease);
	}

	.switch[aria-checked='true'] .knob {
		left: 19px;
	}
</style>
