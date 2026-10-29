<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { FEATURES } from '$lib/brand';
	import Onboarding from '$lib/workbench/Onboarding.svelte';
	import { onboard } from '$lib/workbench/onboard.svelte';

	/** First-launch onboarding (2D, FEAT-180). Workbench only. */
	onMount(() => {
		if (!FEATURES.workbenchWide) void goto('/home');
		else if (!onboard.showing) onboard.again();
	});
</script>

{#if FEATURES.workbenchWide}<Onboarding />{/if}
