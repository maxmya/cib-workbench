<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	/** Import is a sheet over Calls now (FEAT-178); the old address opens it. */
	onMount(() => {
		void goto('/calls?import', { replaceState: true });
	});
</script>
