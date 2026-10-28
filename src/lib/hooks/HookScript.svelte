<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { tokenizeLines } from '$lib/diff/highlight';
	import { scriptLanguage } from './script';

	/**
	 * A hook file, coloured as its language (FEAT-171): a shell script by
	 * default, or what its first line names. Line numbers, so an agent's
	 * explanation can be followed along. Read-only; editing is a textarea
	 * the screen swaps in.
	 */
	interface Props {
		path: string;
		text: string;
		/** Cut at 64 KiB when it was read. */
		truncated?: boolean;
		/** A line to mark and bring into view, from 1: the step chosen (FEAT-176). */
		mark?: number | null;
	}

	let { path, text, truncated = false, mark = null }: Props = $props();

	let pre = $state<HTMLPreElement | null>(null);
	$effect(() => {
		const line = mark;
		if (!pre || !line) return;
		const row = pre.querySelectorAll('.line')[line - 1] as HTMLElement | undefined;
		row?.scrollIntoView?.({ block: 'nearest' });
	});

	const language = $derived(scriptLanguage(path, text));
	const lines = $derived(tokenizeLines(text.replace(/\n$/, '').split('\n'), language));
</script>

<pre class="script mono" data-language={language} bind:this={pre}><code
		>{#each lines as tokens, index (index)}<span class="line" class:marked={mark === index + 1}
				><span class="number" aria-hidden="true">{index + 1}</span><span class="text"
					>{#each tokens as token, at (at)}{#if token.type === 'plain'}{token.text}{:else}<span
								class={`tok-${token.type}`}>{token.text}</span
							>{/if}{/each}</span
				></span
			>{/each}{#if truncated}<span class="cut">… cut at 64 KiB</span>{/if}</code
	></pre>

<style>
	.script {
		margin: 0;
		padding: 10px 0;
		overflow: auto;
		max-height: 420px;
		border: 1px solid var(--line);
		border-radius: var(--r-button);
		background: var(--sunken);
		font-size: var(--fs-mono);
		line-height: 1.55;
		user-select: text;
	}

	code {
		display: block;
		min-width: fit-content;
	}

	.line {
		display: flex;
		padding-right: 14px;
	}

	.line.marked {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
		box-shadow: inset 3px 0 0 var(--accent);
	}

	.number {
		flex: none;
		width: 3.2em;
		padding-right: 12px;
		text-align: right;
		color: var(--muted);
		opacity: 0.6;
		user-select: none;
	}

	.text {
		white-space: pre;
	}

	.cut {
		display: block;
		padding: 4px 14px 0 calc(3.2em + 12px);
		color: var(--muted);
	}
</style>
