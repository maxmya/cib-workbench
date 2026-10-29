<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { GITLAB } from '$lib/brand';
	import Btn from '$lib/ui/Btn.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import { expiry, failureSentence } from './gitlab';
	import { workbench } from './state.svelte';
	import TokenField from './TokenField.svelte';
	import TokenPage from './TokenPage.svelte';

	/**
	 * Settings › GitLab (FEAT-181). In Workbench it stands in for the generic
	 * accounts section, which stays in the code for Spagitty's path.
	 */
	let replacing = $state(false);

	onMount(() => {
		void workbench.load().then(() => {
			if (workbench.login) void workbench.readAccount();
		});
	});

	const account = $derived(workbench.account);
	const ends = $derived(account ? expiry(account.expiresAt, Date.now()) : null);
</script>

<section class="section" aria-labelledby="gitlab-heading">
	<header>
		<h2 class="heading" id="gitlab-heading">GitLab</h2>
	</header>

	<label class="field">
		<span class="label">Address</span>
		<input type="text" class="mono" readonly value={GITLAB.host} title="Set in this build" />
	</label>

	{#if !workbench.login}
		<p class="state">Not connected</p>
		{#if !replacing}
			<div class="row"><Btn primary onclick={() => (replacing = true)}>Connect</Btn></div>
		{/if}
	{:else if workbench.reading && !account}
		<Loader label="Reading CIB GitLab…" />
	{:else if account}
		<dl class="facts">
			<dt>Signed in as</dt>
			<dd>{account.name} <span class="muted">@{account.login}</span></dd>
			{#if account.scopesKnown}
				<dt>Scopes</dt>
				<dd class="mono">{account.scopes.join(', ')}</dd>
			{/if}
			{#if ends}
				<dt>Token</dt>
				<dd class:warn={ends.warn}>{ends.label}</dd>
			{/if}
		</dl>
	{:else}
		<p class="state">Signed in as <span class="mono">@{workbench.login}</span></p>
		{#if workbench.accountError}<p class="error" role="alert">{failureSentence(workbench.accountError)}</p>{/if}
	{/if}

	{#if workbench.login && !replacing}
		<div class="row">
			<Btn onclick={() => (replacing = true)}>Replace token</Btn>
			<Btn danger onclick={() => workbench.disconnect()}>Disconnect</Btn>
		</div>
	{/if}

	{#if replacing}
		<div class="card replace">
			<TokenPage />
			<TokenField onconnected={() => (replacing = false)} />
			<div class="row"><Btn quiet onclick={() => (replacing = false)}>Cancel</Btn></div>
		</div>
	{/if}

	{#if workbench.error}<p class="error" role="alert">{workbench.error}</p>{/if}
</section>

<style>
	.section {
		display: flex;
		flex-direction: column;
		gap: 14px;
		max-width: 760px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-width: 520px;
	}

	.label {
		font-weight: 600;
		font-size: var(--fs-secondary);
	}

	.state {
		margin: 0;
	}

	.facts {
		display: grid;
		grid-template-columns: max-content 1fr;
		gap: 6px 18px;
		margin: 0;
	}

	dt {
		color: var(--muted);
	}

	dd {
		margin: 0;
	}

	.muted {
		color: var(--muted);
	}

	.warn {
		color: var(--warn);
		font-weight: 600;
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		gap: 10px;
	}

	.replace {
		padding: 18px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.error {
		margin: 0;
		color: var(--danger);
	}
</style>
