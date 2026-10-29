<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import Loader from '$lib/ui/Loader.svelte';
	import { changes } from '$lib/changes/store.svelte';
	import { discardHunk } from '$lib/changes/discard';
	import Chip from '$lib/ui/Chip.svelte';
	import type { DiffLine } from '$lib/types';
	import { pairWords } from '$lib/diff/words';
	import { detectLanguage, paint, tokenizeDiff, type Painted, type Token } from '$lib/diff/highlight';
	import { numbersIn, workingFileAttrs } from '$lib/files/open';
	import { repo } from '$lib/repo.svelte';
	import { gapsOf } from './whole';

	/**
	 * The hunks of the selected file, each with the actions that make sense for
	 * the side it is on: stage it or throw it away on the unstaged side, take it
	 * back out on the staged side.
	 *
	 * Discard is only offered where there is something to discard. On the staged
	 * side the change has been kept once already, and unstaging it is one click
	 * away from being discardable — offering both there would put a destructive
	 * button next to a reversible one that reads almost the same.
	 *
	 * Deliberately unified only. Split is for reading a commit someone else
	 * wrote; here the question is "does this hunk belong in the commit", and one
	 * column keeps the answer next to the button.
	 *
	 * **Whole file** (FEAT-189): the same hunks, with the unchanged lines
	 * between them filled in, so a hunk is read where it sits in the file. Each
	 * line opens in Files where it is, by the old side's number: Files reads
	 * the branch, and the old side is the branch, or the index on top of it.
	 */

	const file = $derived(changes.file);
	const head = $derived(repo.info?.head.branch ?? repo.info?.head.id ?? null);
	/** The unchanged lines between the hunks, when the file is shown whole. */
	const gaps = $derived(
		file && changes.whole && changes.wholeLines ? gapsOf(file.hunks, changes.wholeLines) : null
	);
	const gapSyntax = $derived.by(() => {
		if (!file || !gaps) return null;
		const language = detectLanguage(file.path);
		return {
			before: gaps.before.map((gap) => tokenizeDiff(gap, language)),
			after: tokenizeDiff(gaps.after, language)
		};
	});
	/** Each hunk's lines by the number Files has them at. */
	const numbers = $derived(file ? file.hunks.map((hunk) => numbersIn(hunk.lines, 'old')) : []);
	/** The changed words in each hunk's changed lines (FEAT-090). */
	const marks = $derived(file ? file.hunks.map((hunk) => pairWords(hunk.lines)) : []);
	/**
	 * Each hunk's colour (BUG-096), read the way Review reads a diff: the old
	 * and new sides apart, so a comment opened on one does not run into the
	 * other. A hunk starts fresh, and so does each run of unchanged lines
	 * shown between two.
	 */
	const syntax = $derived.by(() => {
		if (!file || file.binary) return [];
		const language = detectLanguage(file.path);
		return file.hunks.map((hunk) => tokenizeDiff(hunk.lines, language));
	});
	const side = $derived(changes.selection?.side ?? 'unstaged');

	function sign(line: DiffLine): string {
		if (line.origin === 'added') return '+';
		if (line.origin === 'removed') return '−';
		return ' ';
	}
</script>

{#snippet lineRow(line: DiffLine, runs: Painted[], at: number)}
	<div class="line {line.origin}" {...workingFileAttrs(file?.path ?? '', file?.status ?? '', head, { line: at })}>
		<span class="num">{line.old ?? ''}</span>
		<span class="num">{line.new ?? ''}</span>
		<span class="sign">{sign(line)}</span>
		<span class="text"
			>{#each runs as run, p (p)}<span class="tok-{run.type}" class:word={run.changed}>{run.text}</span
				>{/each}</span
		>
	</div>
{/snippet}

{#snippet gapRows(gap: DiffLine[], tokens: Token[][] | undefined)}
	{#each gap as line, row (row)}
		{@render lineRow(line, paint(tokens?.[row] ?? [{ type: 'plain', text: line.text }], undefined), line.old ?? 0)}
	{/each}
{/snippet}

<div class="pane">
	{#if changes.fileError}
		<div class="pad note error">{changes.fileError}</div>
	{:else if changes.selection === null}
		<div class="pad note">Select a file to see what changed in it.</div>
	{:else if file === null}
		<div class="pad note">{#if changes.fileLoading}<Loader label="Reading…" />{:else}Could not read this diff.{/if}</div>
	{:else if file.binary}
		<div class="pad note">Binary file. There are no hunks to stage individually.</div>
	{:else if file.tooLarge}
		<div class="pad note">This file is too large to diff.</div>
	{:else if file.hunks.length === 0}
		<div class="pad note">No line changes — only the file's mode changed.</div>
	{:else}
		{#each file.hunks as hunk, index (hunk.header + index)}
			{#if gaps}{@render gapRows(gaps.before[index], gapSyntax?.before[index])}{/if}
			<section class="hunk">
				<div class="hunk-head">
					<span class="mono muted">{hunk.header}</span>
					<div class="acts">
						<Chip
							onclick={() => changes.hunk(index, hunk.header)}
							title={side === 'unstaged'
								? 'Stage this hunk and nothing else'
								: 'Take this hunk back out of the next commit'}
						>
							{side === 'unstaged' ? 'stage hunk' : 'unstage hunk'}
						</Chip>
						{#if side === 'unstaged'}
							<Chip
								danger
								onclick={() => discardHunk(index, hunk.header)}
								title="Throw these lines away — this cannot be undone"
							>
								discard hunk
							</Chip>
						{/if}
					</div>
				</div>
				{#each hunk.lines as line, row (row)}
					{@render lineRow(
						line,
						paint(syntax[index]?.[row] ?? [{ type: 'plain', text: line.text }], marks[index]?.get(row)),
						numbers[index]?.[row] ?? 0
					)}
				{/each}
			</section>
		{/each}
		{#if gaps}{@render gapRows(gaps.after, gapSyntax?.after)}{/if}
	{/if}
</div>

<style>
	.acts {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	/*
	 * Nested column flex. `flex-basis: 0` is what claims the leftover height
	 * even when the commit-message well has a large intrinsic size — WebKitGTK
	 * otherwise sizes this pane to zero at 100% zoom, and the hunks only appear
	 * after a zoom change forces a relayout. `min-height: 0` is what then lets
	 * the pane scroll inside that leftover rather than overflow it.
	 */
	.pane {
		flex: 1 1 0;
		min-width: 0;
		min-height: 0;
		overflow: auto;
		font-family: var(--code-font);
		font-size: var(--fs-code);
		line-height: var(--code-lh);
		letter-spacing: var(--code-ls);
	}

	.pad {
		padding: 10px 12px;
	}

	.error {
		color: var(--danger);
	}

	/*
	 * Same fill as the lines, not `--panel`. The file list is `--panel` with a
	 * right rule; a panel-coloured header sits on that rule and the `@@` row
	 * looks unframed while the code below it does not.
	 */
	.hunk-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
		background: var(--bg);
		border-top: 1px solid var(--soft);
		border-bottom: 1px solid var(--soft);
		padding: 2px 8px;
		position: sticky;
		top: 0;
		z-index: 1;
	}

	/* Rows are as wide as their longest line, so the tint spans the whole row
	   when the pane is scrolled sideways. */
	.line {
		display: flex;
		min-width: 100%;
		width: max-content;
	}

	.num {
		flex: none;
		width: var(--diff-gutter-w);
		padding-right: 8px;
		text-align: right;
		color: var(--muted);
		user-select: none;
	}

	.sign {
		flex: none;
		width: 14px;
		text-align: center;
		color: var(--muted);
		user-select: none;
	}

	.text {
		white-space: pre;
		tab-size: 4;
		padding-right: 12px;
	}

	/* Settings › Reading decides the strength (FEAT-090): calm is a marker
	   down the side and a faint tint, classic the full rows. */
	.line.added {
		background: var(--diff-add-bg);
		box-shadow: inset var(--diff-marker) 0 0 var(--ok);
	}

	.line.removed {
		background: var(--diff-del-bg);
		box-shadow: inset var(--diff-marker) 0 0 var(--danger);
		color: var(--diff-del-ink);
	}

	.line.added .word {
		background: var(--diff-add-hl);
		border-radius: 3px;
	}

	.line.removed .word {
		background: var(--diff-del-hl);
		border-radius: 3px;
	}
</style>
