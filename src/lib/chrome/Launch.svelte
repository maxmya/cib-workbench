<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { fade } from 'svelte/transition';
	import { BRAND_ASSETS, ORG, PRODUCT_NAME, SPLASH_LINE } from '$lib/brand';
	import { prefersReducedMotion } from '$lib/motion';
	import { theme } from '$lib/theme.svelte';
	import InternalNotice from '$lib/ui/InternalNotice.svelte';
	import Wordmark from '$lib/ui/Wordmark.svelte';
	import { version } from '$lib/version';

	/**
	 * Workbench's launch screen (`design_handoff_cib_workbench/source/Launch.dc.html`),
	 * over the window while the session's first repository opens: CIB, the
	 * wordmark, the splash line, a progress line, and the internal-use notice
	 * above a hairline. The only screen Workbench adds to Spagitty.
	 *
	 * The progress is a moving segment rather than a percentage: opening a
	 * repository is one request that says nothing until it answers.
	 */
	interface Props {
		/** The repository being opened, by name; none when there is nothing to open. */
		repository: string | null;
	}

	let { repository }: Props = $props();
</script>

<div
	class="launch"
	data-tauri-drag-region
	role="status"
	aria-label={repository ? `Opening ${repository}` : `Opening ${PRODUCT_NAME}`}
	out:fade={{ duration: prefersReducedMotion() ? 0 : 180 }}
>
	<div class="centre" data-tauri-drag-region>
		<span class="org">{ORG}</span>
		<h1 class="mark"><Wordmark size={104} glow /></h1>
		<p class="line">{SPLASH_LINE}</p>

		<div class="progress">
			<div class="track" aria-hidden="true"><div class="segment"></div></div>
			<span class="opening">
				Opening{#if repository}&nbsp;<span class="mono">{repository}</span>{/if}…
			</span>
		</div>
	</div>

	<footer class="foot">
		<img
			class="logo"
			src={theme.isDark ? BRAND_ASSETS.logoReversedTagline : BRAND_ASSETS.logoColor}
			alt={theme.isDark ? `${ORG}, The bank to trust` : ORG}
			draggable="false"
		/>
		<div class="notice"><InternalNotice version={version.number} /></div>
	</footer>
</div>

<style>
	.launch {
		position: fixed;
		inset: 0;
		z-index: 1000;
		display: flex;
		flex-direction: column;
		box-sizing: border-box;
		padding: 36px 48px 40px;
		background: var(--environment);
		color: var(--ink);
		font-size: var(--fs-ui);
	}

	.centre {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 18px;
		text-align: center;
	}

	.org {
		font-size: 0.96em;
		font-weight: 600;
		letter-spacing: 0.32em;
		color: var(--muted);
	}

	.mark {
		margin: 0;
		font-size: inherit;
		font-weight: inherit;
		line-height: 1;
	}

	.line {
		margin: 4px 0 0;
		font-size: 1.1em;
		letter-spacing: 0.02em;
		color: color-mix(in srgb, var(--ink) 70%, transparent);
	}

	.progress {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		margin-top: 34px;
	}

	.track {
		position: relative;
		width: 260px;
		height: 3px;
		border-radius: 2px;
		overflow: hidden;
		background: var(--soft);
	}

	.segment {
		position: absolute;
		inset: 0 auto 0 0;
		width: 38%;
		border-radius: 2px;
		background: var(--accent);
		animation: travel 1.4s ease-in-out infinite;
	}

	@keyframes travel {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(270%);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.segment {
			animation: none;
		}
	}

	.opening {
		font-size: var(--fs-secondary);
		color: var(--muted);
	}

	.foot {
		width: 100%;
		max-width: 1280px;
		margin: 0 auto;
		box-sizing: border-box;
		padding-top: 24px;
		border-top: 1px solid var(--soft);
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 20px 40px;
	}

	.logo {
		display: block;
		height: 60px;
		width: auto;
	}

	.notice {
		flex: 1 1 420px;
		max-width: 640px;
	}
</style>
