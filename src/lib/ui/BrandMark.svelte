<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { BRAND_ASSETS } from '$lib/brand';

	interface Props {
		size?: number;
		class?: string;
	}

	let { size = 24, class: className = '' }: Props = $props();
</script>

<!-- Workbench's app icon: CIB's globe on CIB blue, drawn by
	 `tools/make-workbench-icon.py`, the same source the installer's icons come from. -->
<img
	class="brand-mark {className}"
	src={BRAND_ASSETS.icon}
	alt=""
	width={size}
	height={size}
	draggable="false"
/>

<style>
	.brand-mark {
		display: inline-block;
		vertical-align: middle;
		flex: none;
		object-fit: contain;
	}
</style>
