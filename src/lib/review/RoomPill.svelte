<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { flushSync } from 'svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import { centreOffset, fitLevel } from './pill';
	import { room } from './room.svelte';

	/**
	 * The review controls (FEAT-091): a second glass pill floating at the
	 * bottom of the diff column, the way the toolbar floats under the pane.
	 *
	 * The only blurred surface the room adds — WebKitGTK paints blur in
	 * software, and a diff row is never blurred.
	 */

	/** Open the Finish review card, which the room owns. */
	let { onfinish }: { onfinish: () => void } = $props();

	const file = $derived(room.selectedFile);
	const name = $derived(file ? file.path.slice(file.path.lastIndexOf('/') + 1) : '');
	const viewed = $derived(file !== null && room.isViewed(file.path));
	// Every file read: what is left is to finish (TASK-053).
	const done = $derived(room.files.length > 0 && room.viewedCount === room.files.length);

	/**
	 * How tight it is drawn (BUG-109): 0 is whole, 3 the smallest. One row
	 * always; a narrow column gets shorter labels, then a Viewed that is only
	 * its tick, then the file's place without its name.
	 */
	const TIGHTEST = 3;
	let tight = $state(0);
	const SCOPES = [
		['Changes', 'Whole file', 'Before and after'],
		['Changes', 'Whole', 'Sides'],
		['Diff', 'File', 'Sides'],
		['Diff', 'File', 'Sides']
	];
	const scopes = $derived(SCOPES[tight]);

	/** Not viewed: tick it and go on to the next. Viewed: take the tick back. */
	function mark() {
		if (file && viewed) void room.setViewed(file.path, false);
		else void room.viewedNext();
	}

	/**
	 * Onto the window's centre line, over the toolbar, as far as the column
	 * allows (BUG-104). Measured again whenever the column or the window
	 * changes size: the panes either side are dragged.
	 */
	let pill = $state<HTMLElement | null>(null);
	let shift = $state(0);

	$effect(() => {
		const element = pill;
		const column = element?.parentElement;
		if (!element || !column) return;
		const measure = () => {
			const box = column.getBoundingClientRect();
			// The room inside the pill's margins, then each level drawn until one fits.
			tight = fitLevel(
				column.clientWidth - 16,
				(level) => {
					flushSync(() => (tight = level));
					return element.scrollWidth;
				},
				TIGHTEST
			);
			flushSync();
			shift = centreOffset(box, window.innerWidth, element.offsetWidth);
		};
		// The observer answers once as soon as it is watching: the first measure.
		const observer = new ResizeObserver(measure);
		observer.observe(column);
		observer.observe(element);
		window.addEventListener('resize', measure);
		return () => {
			observer.disconnect();
			window.removeEventListener('resize', measure);
		};
	});
</script>

<div
	class="pill ornament"
	role="toolbar"
	aria-label="Review controls"
	bind:this={pill}
	style:--shift={`${shift}px`}
	data-tight={tight}
>
	<span class="group">
		<button class="tool icon" aria-label="Previous file" title={tight === TIGHTEST ? file?.path : undefined} onclick={() => room.step(-1)}>
			<Icon name="chevron-left" size="1.1em" weight={1.9} />
		</button>
		<span class="where">
			{#if room.layout === 'one'}
				{#if tight < TIGHTEST}<span class="name mono" title={file?.path}>{name}</span>{/if}
				<span class="note" title={tight === TIGHTEST ? file?.path : undefined}
					>{tight === TIGHTEST ? `${room.position + 1}/${room.files.length}` : `${room.position + 1} of ${room.files.length}`}</span
				>
			{:else}
				{#if tight < TIGHTEST}<span class="name mono">All files</span>{/if}
				<span class="note">{room.files.length} {room.files.length === 1 ? 'file' : 'files'}</span>
			{/if}
		</span>
		<button class="tool icon" aria-label="Next file" onclick={() => room.step(1)}>
			<Icon name="chevron-right" size="1.1em" weight={1.9} />
		</button>
	</span>
	<span class="vr"></span>
	<span class="segments">
		<button class="tool" aria-pressed={room.scope === 'changes'} title="Only what changed" onclick={() => room.setScope('changes')}>
			{scopes[0]}
		</button>
		<button
			class="tool"
			aria-pressed={room.scope === 'whole'}
			disabled={!room.canShowWhole}
			title={room.canShowWhole ? 'The whole file, its changes marked' : 'Needs the pull request fetched'}
			onclick={() => room.setScope('whole')}
		>
			{scopes[1]}
		</button>
		<button
			class="tool"
			aria-pressed={room.scope === 'sides'}
			disabled={!room.canShowWhole}
			title={room.canShowWhole
				? 'The whole file as it was, beside the whole file as it is'
				: 'Needs the pull request fetched'}
			onclick={() => room.setScope('sides')}
		>
			{scopes[2]}
		</button>
	</span>
	<span class="segments">
		<button class="tool" aria-pressed={room.layout === 'one'} title="One file at a time" onclick={() => room.setLayout('one')}>One</button>
		<button class="tool" aria-pressed={room.layout === 'all'} title="Every file, one after another" onclick={() => room.setLayout('all')}>All</button>
	</span>
	<span class="vr"></span>
	{#if done}
		<button class="tool finish" disabled={room.phase !== 'ready'} onclick={onfinish}>
			{tight >= 2 ? 'Finish' : 'Finish review'}
		</button>
	{:else}
		<button
			class="tool viewed"
			aria-pressed={viewed}
			aria-label="Viewed"
			disabled={!file}
			title={viewed ? 'Viewed' : 'Tick it and go to the next file'}
			onclick={mark}
		>
			<Icon name={viewed ? 'circle-check' : 'circle'} size="1.1em" weight={2} />{#if tight < 2}Viewed{/if}
		</button>
	{/if}
</div>
<style>
	/*
	 * Centred by its margins, between the column's two edges (BUG-080). With
	 * left at 50% and a transform it had half the column to be laid out in:
	 * in a narrow column the file's name was squeezed to nothing and the rest
	 * ran out over the conversation. It keeps to one row (BUG-109): a column
	 * too narrow for it gets it drawn tighter, `data-tight` 1 to 3.
	 */
	.pill {
		position: absolute;
		left: 8px;
		right: 8px;
		bottom: 14px;
		width: fit-content;
		max-width: calc(100% - 16px);
		margin-inline: auto;
		box-sizing: border-box;
		display: flex;
		flex-wrap: nowrap;
		align-items: center;
		justify-content: center;
		gap: 6px;
		height: 52px;
		overflow: hidden;
		padding: 9px;
		border-radius: 26px;
		white-space: nowrap;
		z-index: 2;
		transform: translateX(var(--shift, 0px));
	}

	.group {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.tool {
		display: flex;
		align-items: center;
		gap: 7px;
		height: 34px;
		padding: 0 12px;
		border-radius: var(--r-pill);
		font-size: var(--fs-secondary);
		color: var(--ink);
		white-space: nowrap;
		transition:
			background var(--t-fast) var(--ease),
			color var(--t-fast) var(--ease);
	}

	.tool:hover:not(:disabled) {
		background: var(--hover);
	}

	.tool:disabled {
		opacity: 0.4;
	}

	.tool.icon {
		width: 32px;
		padding: 0;
		justify-content: center;
	}

	/* One width whatever the file is called (BUG-104): a long name is cut, a
	   short one leaves room, and the buttons either side never move. */
	.where {
		display: flex;
		flex-direction: column;
		line-height: 1.15;
		flex: none;
		width: 150px;
		min-width: 0;
	}

	.name {
		font-size: var(--fs-mono);
		font-weight: 600;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.where .note {
		font-size: var(--fs-mono);
	}

	/* Two or three choices in a sunken track; the chosen one raised out of it. */
	.segments {
		display: flex;
		padding: 2px;
		border-radius: var(--r-pill);
		background: color-mix(in srgb, var(--sunken) 70%, transparent);
		border: 1px solid var(--pane-edge);
	}

	.segments .tool {
		height: 28px;
		color: var(--muted);
	}

	.segments .tool[aria-pressed='true'] {
		background: var(--surface-2);
		color: var(--ink);
		font-weight: 600;
		box-shadow: var(--shadow-1);
	}

	/* A ring while the file is still to read; the ring ticked, in green, once
	   it is read (TASK-053). */
	.viewed {
		font-weight: 600;
	}

	/* Viewed and Finish review take the same place, so the swap moves nothing. */
	.viewed,
	.finish {
		min-width: 128px;
		justify-content: center;
	}

	/* Tighter: a narrower name, then shorter labels and a Viewed that is its
	   tick, then the file's place alone. */
	.pill[data-tight='1'] .where {
		width: 110px;
	}

	.pill[data-tight='2'] .where {
		width: 84px;
	}

	.pill[data-tight='2'] .tool:not(.icon),
	.pill[data-tight='3'] .tool:not(.icon) {
		padding: 0 9px;
	}

	.pill[data-tight='2'] .viewed,
	.pill[data-tight='3'] .viewed {
		min-width: 0;
		width: 38px;
		padding: 0;
	}

	.pill[data-tight='2'] .finish,
	.pill[data-tight='3'] .finish {
		min-width: 0;
	}

	.pill[data-tight='3'] {
		gap: 4px;
		padding: 9px 7px;
	}

	.pill[data-tight='3'] .where {
		width: auto;
		min-width: 40px;
		align-items: center;
	}

	.pill[data-tight='3'] .vr {
		display: none;
	}

	.viewed[aria-pressed='true'] {
		background: color-mix(in srgb, var(--ok) 18%, transparent);
		color: var(--ok);
	}

	.finish {
		background: var(--accent);
		color: var(--on-accent);
		font-weight: 600;
	}

	.finish:hover:not(:disabled) {
		background: color-mix(in srgb, var(--accent) 86%, var(--ink));
	}
</style>
