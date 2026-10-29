<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import Btn from '$lib/ui/Btn.svelte';
	import { notice } from '$lib/ui/notice.svelte';
	import { onboard } from './onboard.svelte';
	import { workbench } from './state.svelte';

	/**
	 * Settings › Behaviour › *Run setup again* (FEAT-180): clears `onboarded`
	 * and opens `/welcome`. The connected account is kept, so GitLab and Token
	 * are passed over.
	 */
	let busy = $state(false);

	async function again() {
		busy = true;
		try {
			await workbench.load();
			await workbench.resetOnboarding();
			onboard.again();
			await goto('/welcome');
		} catch (error) {
			notice.failed('Could not start setup', error);
		} finally {
			busy = false;
		}
	}
</script>

<div class="again">
	<Btn {busy} disabled={busy} onclick={again}>Run setup again</Btn>
</div>

<style>
	.again {
		margin-top: 6px;
	}
</style>
