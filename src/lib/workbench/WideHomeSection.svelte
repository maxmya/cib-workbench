<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { HOME_DEFAULT } from '$lib/home/layout';
	import { settings } from '$lib/settings/store.svelte';
	import Btn from '$lib/ui/Btn.svelte';
	import FollowList from './FollowList.svelte';
	import SectionsOrder from './SectionsOrder.svelte';
	import { workbench } from './state.svelte';

	/**
	 * Settings › Home in Workbench (FEAT-182, FEAT-183): what Home follows, and
	 * the order of its sections. Changes are kept as they are made.
	 */
	onMount(() => void workbench.load());

	const layout = $derived(settings.home);
	const isDefault = $derived(layout.hidden.length === 0 && layout.order.length === 0);
</script>

<section class="section" aria-labelledby="wide-home-heading">
	<header class="top">
		<h2 class="heading" id="wide-home-heading">Home</h2>
		<Btn disabled={isDefault} onclick={() => settings.choose('home', HOME_DEFAULT)}>Reset order</Btn>
	</header>
	{#if workbench.login}
		<div class="card pane"><FollowList selected={workbench.followed} onchange={(next) => workbench.setFollowed(next)} /></div>
	{:else}
		<p class="note">Connect CIB GitLab in Settings › GitLab to choose repositories.</p>
	{/if}
	<div class="card pane"><SectionsOrder {layout} onchange={(next) => settings.choose('home', next)} /></div>
	{#if workbench.error}<p class="error" role="alert">{workbench.error}</p>{/if}
</section>

<style>
	.section {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-start;
		gap: 16px;
	}

	.top {
		flex-basis: 100%;
		display: flex;
		align-items: center;
		gap: 14px;
	}

	.pane {
		flex: 1 1 380px;
		min-width: 0;
		padding: 18px;
	}

	.note {
		flex-basis: 100%;
		margin: 0;
	}

	.error {
		margin: 0;
		color: var(--danger);
	}
</style>
