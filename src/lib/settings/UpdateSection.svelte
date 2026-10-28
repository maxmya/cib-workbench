<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { PRODUCT_NAME } from '$lib/brand';
	import Btn from '$lib/ui/Btn.svelte';
	import Chip from '$lib/ui/Chip.svelte';
	import { updates } from '$lib/updates/store.svelte';
	import { settings } from './store.svelte';

	/**
	 * Whether there is a newer Workbench, and whether to look on startup.
	 *
	 * It sits under Behaviour's heading rather than beside the licence, because
	 * the interesting half is the toggle: it is the only preference in the
	 * application that causes a network request, and somebody deciding about it
	 * should be reading the sentence that says so.
	 *
	 * The link is shown rather than opened. Spagitty does not launch browsers —
	 * there is no opener in the build and adding one to save a copy-and-paste
	 * would be a dependency and a new way for a URL from a host to be acted on.
	 *
	 * Workbench installs its own updates (FEAT-187): a newer release downloads
	 * in the background and waits for *Restart*. The link is what is left for
	 * a build that cannot install one, and the line under it says why.
	 */

	const update = $derived(settings.update);
	const stored = $derived(settings.settings);

	let copied = $state(false);

	async function copy(url: string) {
		try {
			await navigator.clipboard.writeText(url);
			copied = true;
			setTimeout(() => (copied = false), 1200);
		} catch {
			// No clipboard, which is the case in a plain browser. The URL is on
			// screen either way, which is the part that matters.
		}
	}
</script>

<section class="section" id="updates">
	<header>
		<h2 class="heading">Updates</h2>
	</header>

	<div class="row">
		<Chip
			active={stored.checkForUpdates}
			disabled={settings.busy}
			onclick={() => settings.toggle('checkForUpdates')}
			title="Ask the project for the latest release when {PRODUCT_NAME} starts"
		>
			{stored.checkForUpdates ? 'on' : 'off'}
		</Chip>
		<div class="text">
			<div title="One request to the project's releases page. Turning it off stops every request.">
				Check for updates when {PRODUCT_NAME} starts
			</div>
		</div>
	</div>

	<div class="hr"></div>

	<div class="row">
		<Btn disabled={settings.checking || updates.busy} onclick={() => updates.check()}>
			{settings.checking || updates.phase === 'checking' ? 'Checking…' : 'Check now'}
		</Btn>
		{#if updates.phase === 'ready'}
			<Btn onclick={() => updates.restart()}>Restart now</Btn>
		{/if}
		{#if update && updates.phase === 'unsupported'}
			<span class="note">
				This build:
				<span class="mono">{update.current ?? 'compiled here, with no release tag'}</span>
			</span>
		{/if}
	</div>

	{#if updates.error}
		<p class="note error">{updates.error}</p>
	{:else if updates.phase === 'downloading'}
		<p class="note">Downloading{updates.percent === null ? '…' : ` — ${updates.percent}%`}</p>
	{:else if updates.phase === 'ready'}
		<p class="note">
			<span class="mono">{updates.version}</span> is downloaded. It is installed when {PRODUCT_NAME} restarts.
		</p>
	{:else if updates.phase === 'restarting'}
		<p class="note">Restarting…</p>
	{:else if updates.phase === 'current'}
		<p class="note">Up to date — <span class="mono">{updates.version}</span>.</p>
	{:else if settings.updateError}
		<p class="note error">{settings.updateError}</p>
	{:else if updates.phase === 'unsupported' && update?.channel === 'development'}
		<!--
			A build somebody compiled has no tag to be behind, and is usually
			ahead of every release. Telling them otherwise would be wrong.
		-->
		<p class="note">Development build. Latest release: <span class="mono">{update.latest}</span>.</p>
	{:else if updates.phase === 'unsupported' && update?.newer}
		<p class="note">
			<span class="mono">{update.latest}</span> has been released. {updates.reason}
		</p>
		<div class="row">
			<span class="mono url">{update.url}</span>
			<Btn onclick={() => copy(update.url)}>{copied ? 'Copied' : 'Copy link'}</Btn>
		</div>
	{:else if updates.phase === 'unsupported' && update}
		<p class="note">
			Up to date — <span class="mono">{update.latest}</span> is the latest release.
		</p>
	{/if}
</section>

<style>
	.section {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-width: 640px;
	}

	.heading {
		margin: 0;
		font-size: var(--fs-ui);
		font-weight: inherit;
	}

	.row {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		flex-wrap: wrap;
	}

	.text {
		min-width: 0;
	}

	.url {
		font-size: var(--fs-secondary);
		color: var(--muted);
		overflow-wrap: anywhere;
		min-width: 0;
	}

	.error {
		color: var(--danger);
	}

	p {
		margin: 0;
	}
</style>
