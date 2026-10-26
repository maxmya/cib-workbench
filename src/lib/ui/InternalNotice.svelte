<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { NOTICE, PRODUCT_NAME } from '$lib/brand';

	/**
	 * Workbench's internal-use notice, in the only approved wording
	 * (`docs/branding.md`): on the launch screen and in Settings › About.
	 */
	interface Props {
		/** The version to name in the licence line. */
		version: string;
	}

	let { version }: Props = $props();
</script>

<div class="notice">
	<p class="lead"><span class="badge">{NOTICE.badge}</span>{NOTICE.lead}</p>
	<p class="small">
		{NOTICE.trademark}
		<span class="mono">{PRODUCT_NAME} {version}</span>
		{NOTICE.licence}
	</p>
</div>

<style>
	.notice {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	p {
		margin: 0;
	}

	.lead {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		font-size: var(--fs-secondary);
		color: color-mix(in srgb, var(--ink) 80%, transparent);
	}

	.badge {
		padding: 1px 8px;
		border-radius: 999px;
		border: 1px solid color-mix(in srgb, var(--warn) 60%, transparent);
		color: var(--warn);
		font-size: 0.85em;
		font-weight: 600;
		letter-spacing: 0.04em;
	}

	.small {
		font-size: var(--fs-secondary);
		line-height: 1.5;
		color: var(--muted);
	}
</style>
