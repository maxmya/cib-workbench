<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { ruleText, statusTone } from './model';
	import type { Answer, FieldRule } from './types';

	/**
	 * A call's *Docs* (FEAT-179): what the code says the body's fields must
	 * be, and what the endpoint answers with each status. Read from the
	 * code when the call was scanned, and read again with every scan, so it
	 * is shown and not edited here.
	 */

	interface Props {
		rules: FieldRule[];
		responses: Answer[];
	}

	let { rules, responses }: Props = $props();

	/** The answer whose example is open, by status. */
	let open = $state<number | null>(null);

	const pretty = (text: string) => {
		try {
			return JSON.stringify(JSON.parse(text), null, 2);
		} catch {
			return text;
		}
	};
</script>

<div class="docs">
	{#if rules.length}
		<section aria-label="The body's fields">
			<h3>The body's fields</h3>
			<table>
				<tbody>
					{#each rules as rule (rule.path)}
						<tr>
							<td class="mono path">{rule.path}</td>
							<td class="mono type">{rule.type}</td>
							<td class="rule">{ruleText(rule) || '—'}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		</section>
	{/if}

	{#if responses.length}
		<section aria-label="What it answers">
			<h3>What it answers</h3>
			<ul>
				{#each responses as answer (answer.status)}
					<li>
						<div class="answer">
							<span class="mono status {statusTone(answer.status)}">{answer.status}</span>
							<span class="says">{answer.says}</span>
							{#if answer.from}
								<span class="from mono">{answer.from}</span>
							{/if}
							<span class="gap"></span>
							{#if answer.body}
								<button
									type="button"
									class="example"
									aria-expanded={open === answer.status}
									onclick={() => (open = open === answer.status ? null : answer.status)}
									>{open === answer.status ? 'Hide example' : 'Example'}</button
								>
							{/if}
						</div>
						{#if answer.body && open === answer.status}
							<pre class="mono">{pretty(answer.body)}</pre>
						{/if}
					</li>
				{/each}
			</ul>
		</section>
	{/if}

</div>

<style>
	.docs {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	section {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	h3 {
		margin: 0;
		color: var(--muted);
		font-size: var(--fs-secondary);
		font-weight: normal;
	}

	table {
		border-collapse: collapse;
		width: 100%;
	}

	td {
		padding: 4px 8px 4px 0;
		border-bottom: 1px solid var(--line);
		vertical-align: top;
	}

	.path {
		white-space: nowrap;
	}

	.type,
	.from {
		color: var(--muted);
	}

	.mono {
		font-family: var(--font-mono);
		font-size: var(--fs-mono);
	}

	.rule {
		width: 100%;
		overflow-wrap: anywhere;
	}

	ul {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	li {
		padding: 4px 0;
		border-bottom: 1px solid var(--line);
	}

	.answer {
		display: flex;
		align-items: baseline;
		gap: 10px;
		min-width: 0;
	}

	.status {
		flex: none;
		font-weight: 600;
	}

	.status.ok {
		color: var(--ok);
	}

	.status.warn {
		color: var(--warn);
	}

	.status.danger {
		color: var(--danger);
	}

	.from {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.gap {
		flex: 1;
	}

	.example {
		flex: none;
		padding: 2px 8px;
		border: 1px solid var(--line);
		border-radius: var(--r-pill);
		font-size: var(--fs-secondary);
	}

	.example:hover {
		background: var(--hover);
	}

	pre {
		margin: 6px 0 2px;
		padding: 8px 10px;
		overflow: auto;
		border-radius: var(--r-field);
		background: var(--surface-2);
	}

	.hint {
		margin: 0;
		color: var(--muted);
		font-size: var(--fs-secondary);
	}
</style>
