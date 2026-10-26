<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { BRAND_ASSETS, PRODUCT_NAME } from '$lib/brand';

	/**
	 * The name as Workbench sets it: "W", CIB's globe as the "o", "rkbench", in
	 * Sora SemiBold.
	 *
	 * The globe is a lowercase letter, so it is measured from Sora 600's own
	 * "o": 0.58em tall, from 0.02em below the baseline to the top of the
	 * x-height overshoot, with the o's side bearings (0.043em) as its margins.
	 * The handoff drew it at cap height on the baseline, where it read as a
	 * second capital (corrected by the author, 2026-10-26).
	 */
	interface Props {
		/** Font size in px. */
		size?: number;
		/** The launch screen's soft orange glow round the globe. */
		glow?: boolean;
	}

	let { size = 15, glow = false }: Props = $props();

	const at = PRODUCT_NAME.indexOf('o');
	const head = PRODUCT_NAME.slice(0, at);
	const tail = PRODUCT_NAME.slice(at + 1);
</script>

<span class="wordmark" style="font-size: {size}px" role="img" aria-label={PRODUCT_NAME}
	><span aria-hidden="true">{head}</span><img
		class="globe"
		class:glow
		src={BRAND_ASSETS.globe}
		alt=""
		aria-hidden="true"
		draggable="false"
	/><span aria-hidden="true">{tail}</span></span
>

<style>
	.wordmark {
		display: inline-block;
		font-family: 'Sora', system-ui, sans-serif;
		font-weight: 600;
		letter-spacing: -0.02em;
		line-height: 1;
		white-space: nowrap;
		color: var(--ink);
	}

	.globe {
		display: inline-block;
		width: 0.58em;
		height: 0.58em;
		vertical-align: -0.02em;
		/* The o's bearings; the right one less the letter-spacing text would add. */
		margin: 0 0.023em 0 0.043em;
	}

	.glow {
		filter: drop-shadow(0 0 0.23em var(--globe-glow));
	}
</style>
