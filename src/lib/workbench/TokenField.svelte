<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import Btn from '$lib/ui/Btn.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import { expiry, failureSentence } from './gitlab';
	import { workbench } from './state.svelte';
	import type { ConnectFailure, TokenInfo } from './types';

	/**
	 * Paste a token, check it against CIB GitLab, keep it (FEAT-181).
	 *
	 * The field is cleared once the check passes: the token went to the
	 * keychain and is held nowhere here. A refused one stays in the field to
	 * be corrected, and nothing was saved.
	 */
	interface Props {
		onconnected?: (info: TokenInfo) => void;
		/** Shown after a check passes, until the field is typed in again. */
		done?: TokenInfo | null;
	}

	let { onconnected, done = null }: Props = $props();

	let token = $state('');
	let reveal = $state(false);
	let checking = $state(false);
	let failure = $state<ConnectFailure | null>(null);
	let accepted = $state<TokenInfo | null>(null);
	const shown = $derived(accepted ?? done);

	async function check() {
		if (!token.trim() || checking) return;
		checking = true;
		failure = null;
		try {
			const result = await workbench.connect(token);
			if (result.ok) {
				token = '';
				accepted = result.info;
				onconnected?.(result.info);
			} else {
				failure = result.failure;
			}
		} finally {
			checking = false;
		}
	}

	function typed(event: Event) {
		token = (event.currentTarget as HTMLInputElement).value;
		failure = null;
	}
</script>

<div class="token-field">
	<label for="gitlab-token" class="label">Personal access token</label>
	<div class="row">
		<div class="box" class:bad={failure} class:good={shown && !failure}>
			<input
				id="gitlab-token"
				type={reveal ? 'text' : 'password'}
				value={token}
				oninput={typed}
				onkeydown={(event) => event.key === 'Enter' && check()}
				placeholder="glpat-…"
				autocomplete="off"
				spellcheck="false"
				class="mono"
			/>
			<button
				class="reveal"
				type="button"
				aria-label={reveal ? 'Hide token' : 'Show token'}
				aria-pressed={reveal}
				onclick={() => (reveal = !reveal)}
			>
				{reveal ? 'Hide' : 'Show'}
			</button>
		</div>
		<Btn primary disabled={!token.trim() || checking} busy={checking} onclick={check}>
			{checking ? 'Checking…' : 'Check'}
		</Btn>
	</div>

	<p class="status" class:error={failure} class:ok={shown && !failure} role="status">
		{#if failure}
			<Icon name="warning" size="1em" />{failureSentence(failure)}
		{:else if checking}
			Checking with CIB GitLab…
		{:else if shown}
			<Icon name="circle-check" size="1em" />Saved to your system's secure store.
		{/if}
	</p>

	{#if shown && !failure}
		{@const ends = expiry(shown.expiresAt, Date.now())}
		<div class="who">
			<span class="name">{shown.name} <span class="muted">@{shown.login}</span></span>
			<span class="note">
				{#if shown.scopesKnown}{shown.scopes.join(', ')} · {/if}<span class:warn={ends.warn}>{ends.label}</span>
			</span>
		</div>
	{/if}
</div>

<style>
	.token-field {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.label {
		font-weight: 600;
		font-size: var(--fs-secondary);
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
		align-items: center;
	}

	.box {
		flex: 1 1 320px;
		display: flex;
		align-items: center;
		gap: 6px;
		padding-right: 4px;
		border-radius: var(--r-field);
		border: 1px solid var(--line);
		background: var(--sunken);
	}

	.box.bad {
		border-color: var(--danger);
	}

	.box.good {
		border-color: var(--ok);
	}

	.box input {
		flex: 1;
		min-width: 0;
		border: 0;
		background: transparent;
		box-shadow: none;
	}

	.reveal {
		height: 30px;
		padding: 0 10px;
		border-radius: var(--r-button);
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.reveal:hover {
		background: var(--hover);
		color: var(--ink);
	}

	.reveal[aria-pressed='true'] {
		color: var(--accent);
	}

	.status {
		margin: 0;
		min-height: 1.4em;
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--fs-secondary);
		color: var(--muted);
	}

	.status.error {
		color: var(--danger);
	}

	.status.ok {
		color: var(--ok);
	}

	.who {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 12px 14px;
		border-radius: var(--r-panel);
		background: var(--ok-soft);
		border: 1px solid color-mix(in srgb, var(--ok) 40%, transparent);
	}

	.name {
		font-weight: 600;
	}

	.muted {
		font-weight: 400;
	}

	.warn {
		color: var(--warn);
	}
</style>
