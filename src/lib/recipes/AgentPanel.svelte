<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { PRODUCT_NAME } from '$lib/brand';
	import Btn from '$lib/ui/Btn.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import { LEVELS, levelWords, MAX_NOTE, MAX_SENTENCE, runLabel } from './model';
	import { recipes } from './store.svelte';
	import type { Level } from './types';

	/**
	 * How a recipe is run, and whether an agent helps (FEAT-133).
	 *
	 * Three parts, in the order they are wanted (TASK-076): how far the run
	 * goes on its own, and the button that starts it; *Agent help*, a switch
	 * whose fields are there only while it is on; and the commands, folded
	 * away until asked for. It said everything at once — what an agent would
	 * and would not do, two counters, the brief, the restore point — for a
	 * flow that is a choice and a button.
	 *
	 * Either way Spagitty runs every command itself, a step at a time, after a
	 * restore point is made.
	 */
	const draft = $derived(recipes.draft);
	const check = $derived(recipes.check);
	const on = $derived(draft?.agentHelp ?? false);
	const level = $derived(check?.level ?? draft?.level ?? 'stepByStep');
	const errors = $derived(check?.errors ?? 0);
	const steps = $derived(draft?.steps.length ?? 0);
	const readOnly = $derived(level === 'suggest');
	const helper = $derived(recipes.helpers.find((each) => each.id === draft?.agent) ?? null);
	/** The note, opened by hand; one with something in it is always open. */
	let noting = $state(false);
</script>

{#if draft}
	<aside class="panel" aria-label="How it runs">
		<section class="part">
			<div class="levels" role="radiogroup" aria-label="Level">
				{#each LEVELS as each (each.id)}
					<button
						role="radio"
						class:on={draft.level === each.id}
						aria-checked={draft.level === each.id}
						title={each.label}
						onclick={() => recipes.setLevel(each.id as Level)}>{each.short}</button
					>
				{/each}
			</div>
			<p class="note small">{levelWords(level).says}</p>
			<Btn
				primary
				disabled={steps === 0 || errors > 0 || readOnly || recipes.starting || !check}
				busy={recipes.starting}
				title={readOnly
					? 'Read-only runs nothing. Copy the script, or choose another level.'
					: 'A restore point is made first'}
				onclick={() => recipes.start()}>{runLabel(steps, errors)}</Btn
			>
		</section>

		<section class="part">
			<label
				class="row switch-row"
				title="An agent proposes the blocks from a sentence, and can write a commit message. It never runs a command."
			>
				<span class="hex" aria-hidden="true"><Icon name="agent" size={15} /></span>
				<span class="label">Agent help</span>
				<input type="checkbox" role="switch" checked={on} onchange={() => recipes.setAgentHelp(!on)} />
			</label>

			{#if on}
				<select
					aria-label="Agent"
					value={draft.agent ?? ''}
					onchange={async (event) => {
						// A model the person does not agree to send anything to is not
						// chosen, and the draft is as it was. Nothing changed, so
						// nothing would put the select back: it is put back here.
						const select = event.currentTarget;
						await recipes.setAgent(select.value || null);
						select.value = recipes.draft?.agent ?? '';
					}}
				>
					<option value="">Choose an agent</option>
					{#each recipes.helpers as each (each.id)}
						<option value={each.id}>{each.name}{each.detail ? ` · ${each.detail}` : ''}</option>
					{/each}
				</select>
				{#if recipes.helpers.length === 0}
					<p class="note small">No agent is set up. Add one in Settings › Agents.</p>
				{/if}

				<div class="field">
					<textarea
						rows="3"
						aria-label="What should it do?"
						placeholder="What should it do? Stash my work, bring main in, put it back and push"
						value={recipes.sentence}
						oninput={(event) => recipes.setSentence(event.currentTarget.value)}
					></textarea>
					{#if recipes.sentence.length > MAX_SENTENCE * 0.8}
						<span class="count mono" class:over={recipes.sentence.length > MAX_SENTENCE}
							>{recipes.sentence.length}/{MAX_SENTENCE}</span
						>
					{/if}
				</div>
				<div class="row">
					<Btn
						disabled={!helper ||
							recipes.proposing ||
							recipes.sentence.trim() === '' ||
							recipes.sentence.length > MAX_SENTENCE}
						busy={recipes.proposing}
						title="The agent proposes the blocks. Nothing runs until you press Run."
						onclick={() => recipes.propose()}>Propose blocks</Btn
					>
					{#if recipes.said}
						<span class="note small said">{recipes.said}</span>
					{/if}
				</div>

				{#if noting || draft.note !== ''}
					<div class="field">
						<textarea
							rows="2"
							aria-label="A note for the agent"
							placeholder="A note the agent reads with every request"
							value={draft.note}
							oninput={(event) => recipes.setNote(event.currentTarget.value)}
						></textarea>
						{#if draft.note.length > MAX_NOTE * 0.8}
							<span class="count mono" class:over={draft.note.length > MAX_NOTE}
								>{draft.note.length}/{MAX_NOTE}</span
							>
						{/if}
					</div>
				{:else}
					<button type="button" class="link" onclick={() => (noting = true)}>Add a note for the agent</button>
				{/if}
			{/if}
		</section>

		<details class="part">
			<summary>
				{on ? 'What the agent is sent' : 'The commands'}
				{#if on}
					<span class="guide mono" title="The rules every agent is given first. They ship with {PRODUCT_NAME}.">
						recipes.md v{recipes.guideVersion}
					</span>
				{/if}
			</summary>
			<pre class="brief mono" aria-label={on ? 'The brief, as it is sent' : 'The commands, as a script'}>{(on
					? check?.brief
					: check?.script) ?? ''}</pre>
		</details>
	</aside>
{/if}

<style>
	.panel {
		flex: 0 0 300px;
		min-height: 0;
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 12px;
		background: var(--surface);
		border: 1px solid var(--pane-edge);
		border-radius: var(--r-panel);
		overflow: auto;
	}

	.part {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.part + .part {
		padding-top: 12px;
		border-top: 1px solid var(--soft);
	}

	.row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.switch-row {
		cursor: pointer;
	}

	.switch-row > .label {
		flex: 1;
		font-weight: 600;
	}

	.hex {
		display: grid;
		place-items: center;
		color: var(--agent);
	}

	.small {
		margin: 0;
		font-size: var(--fs-secondary);
	}

	.said {
		min-width: 0;
	}

	.levels {
		display: flex;
		padding: 3px;
		border-radius: var(--r-pill);
		background: var(--sunken);
	}

	.levels button {
		flex: 1;
		padding: 5px 6px;
		border-radius: var(--r-pill);
		font-size: var(--fs-secondary);
		white-space: nowrap;
	}

	.levels button.on {
		background: var(--surface-2);
		box-shadow: var(--shadow-1);
	}

	.field {
		position: relative;
		display: flex;
		flex-direction: column;
	}

	/* Only as the limit nears, in the corner of the field. */
	.count {
		position: absolute;
		right: 8px;
		bottom: 6px;
		color: var(--muted);
		font-size: var(--fs-mono);
	}

	.count.over {
		color: var(--danger);
	}

	textarea {
		resize: none;
	}

	.link {
		align-self: flex-start;
		padding: 0;
		color: var(--accent);
		font-size: var(--fs-secondary);
	}

	.link:hover {
		text-decoration: underline;
	}

	summary {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--muted);
		font-size: var(--fs-secondary);
		cursor: pointer;
	}

	summary::before {
		content: '▸';
		transition: transform var(--t-fast) var(--ease);
	}

	details[open] summary::before {
		transform: rotate(90deg);
	}

	summary::-webkit-details-marker {
		display: none;
	}

	summary:hover {
		color: var(--ink);
	}

	.guide {
		margin-left: auto;
		padding: 1px 8px;
		border: 1px solid color-mix(in srgb, var(--agent) 50%, transparent);
		border-radius: var(--r-pill);
		color: var(--agent);
		font-size: var(--fs-mono);
	}

	.brief {
		max-height: 320px;
		margin: 0;
		padding: 10px;
		border-radius: var(--r-field);
		background: var(--sunken);
		font-size: var(--fs-mono);
		white-space: pre-wrap;
		overflow: auto;
		user-select: text;
	}
</style>
