<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { FEATURES } from '$lib/brand';
	import HomeScreen from '$lib/home/HomeScreen.svelte';
	import WideHome from '$lib/workbench/WideHome.svelte';
</script>

<!-- Workbench: Home across followed repositories (FEAT-183). -->
{#if FEATURES.workbenchWide}<WideHome />{:else}<HomeScreen />{/if}
