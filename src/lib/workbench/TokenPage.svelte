<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { GITLAB, tokenPageUrl } from '$lib/brand';
	import Icon from '$lib/ui/Icon.svelte';
	import { safeUrl } from '$lib/ui/markdown';
	import { notice } from '$lib/ui/notice.svelte';
	import { SCOPE_USES } from './gitlab';

	/**
	 * The way to CIB GitLab's token page (FEAT-181): a link through `safeUrl`,
	 * as *Open on host ↗* is in Issues, so no new command or crate. 16.6.2
	 * keeps the form behind *Add new token*, so `guide` says what to press.
	 */
	interface Props {
		/** The numbered steps beside it, as onboarding shows them. */
		guide?: boolean;
	}

	let { guide = false }: Props = $props();

	const url = tokenPageUrl();
	const href = safeUrl(url);
	let copied = $state<string | null>(null);

	async function copy(text: string, what: string) {
		try {
			await navigator.clipboard.writeText(text);
			copied = what;
			setTimeout(() => (copied = copied === what ? null : copied), 1600);
		} catch (error) {
			notice.failed('Could not copy', error);
		}
	}
</script>

<div class="token-page">
	<div class="main">
		<div class="actions">
			{#if href}
				<a class="open" {href} target="_blank" rel="noopener noreferrer">
					Open token page in your browser <Icon name="link" size="1em" />
				</a>
			{/if}
			<button class="copy" onclick={() => copy(url, 'link')}>{copied === 'link' ? 'Copied' : 'Copy link'}</button>
		</div>
		<code class="url mono">{url}</code>
		<div class="scopes">
			{#each GITLAB.scopes as scope (scope)}
				<span class="scope" title={SCOPE_USES[scope]}><code class="mono">{scope}</code></span>
			{/each}
			<button class="copy small" onclick={() => copy(GITLAB.scopes.join(' '), 'scopes')}
				>{copied === 'scopes' ? 'Copied' : 'Copy scopes'}</button
			>
		</div>
	</div>

	{#if guide}
		<aside class="guide card" aria-label="On the token page">
			<h3>On the token page</h3>
			<ol>
				<li>Press <strong>Add new token</strong>.</li>
				<li>Name it <strong>{GITLAB.tokenName}</strong>.</li>
				<li>Tick {#each GITLAB.scopes as scope, i (scope)}<code class="mono">{scope}</code>{i < GITLAB.scopes.length - 1 ? ', ' : ''}{/each}.</li>
				<li>Set an expiry date.</li>
				<li>Press <strong>Create personal access token</strong>.</li>
				<li>Copy the token.</li>
			</ol>
			<p class="warn" role="note"><Icon name="warning" size="1.1em" />GitLab shows the token only once.</p>
		</aside>
	{/if}
</div>

<style>
	.token-page {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-start;
		gap: 20px 32px;
	}

	.main {
		flex: 1 1 420px;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 14px;
	}

	.actions,
	.scopes {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px;
	}

	.open {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		height: 40px;
		padding: 0 18px;
		border-radius: var(--r-button);
		background: var(--primary);
		color: var(--on-primary);
		font-weight: 600;
		text-decoration: none;
	}

	.open:hover {
		filter: brightness(1.08);
	}

	.copy {
		height: 40px;
		padding: 0 14px;
		border-radius: var(--r-button);
		border: 1px solid var(--line);
		background: transparent;
		color: var(--ink);
		font: inherit;
		cursor: pointer;
	}

	.copy.small {
		height: 30px;
		font-size: var(--fs-secondary);
	}

	.copy:hover {
		background: var(--hover);
	}

	.url {
		display: block;
		padding: 10px 12px;
		border-radius: var(--r-field);
		background: var(--sunken);
		color: var(--muted);
		word-break: break-all;
	}

	.scope {
		padding: 5px 10px;
		border-radius: var(--r-row);
		background: var(--soft);
	}

	.guide {
		flex: 1 1 300px;
		padding: 20px 22px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	h3 {
		margin: 0;
		font-size: var(--fs-ui);
		font-weight: 600;
	}

	ol {
		margin: 0;
		padding-left: 1.4em;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.warn {
		margin: 4px 0 0;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 12px;
		border-radius: var(--r-field);
		background: var(--warn-soft);
		color: var(--warn);
	}
</style>
