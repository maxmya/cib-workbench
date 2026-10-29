<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { goto } from '$app/navigation';
	import { BRAND_ASSETS, GITLAB, NOTICE, ORG, PRODUCT_NAME, SPLASH_LINE } from '$lib/brand';
	import { HOME_DEFAULT, type HomeLayout } from '$lib/home/layout';
	import { settings } from '$lib/settings/store.svelte';
	import { theme } from '$lib/theme.svelte';
	import Btn from '$lib/ui/Btn.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import Wordmark from '$lib/ui/Wordmark.svelte';
	import FollowList from './FollowList.svelte';
	import { keychainName, platformOf } from './gitlab';
	import { onboard } from './onboard.svelte';
	import { back, canGoOn, FIRST, marks, next, STEPS, WELCOME_PAGES, type Stage } from './onboarding';
	import SectionsOrder from './SectionsOrder.svelte';
	import { workbench } from './state.svelte';
	import TokenField from './TokenField.svelte';
	import TokenPage from './TokenPage.svelte';
	import type { Followed, TokenInfo } from './types';

	/**
	 * First launch (2D, FEAT-180): what Workbench is, CIB GitLab, the token,
	 * and Home. Full window, no rail.
	 *
	 * What step 4 chooses is held here and written only when Workbench is
	 * opened, not on every toggle. *Set up later* writes nothing and leaves
	 * `onboarded` unset, so the next launch offers setup again.
	 */

	let stage = $state<Stage>(FIRST);
	/** Connected before this run began: GitLab and Token are passed over. */
	const connected = workbench.login !== null;
	const connectedAs = workbench.login;
	let checked = $state<TokenInfo | null>(null);
	let followed = $state<Followed[]>([...workbench.followed]);
	let layout = $state<HomeLayout>({ ...(settings.home ?? HOME_DEFAULT) });
	let finishing = $state(false);

	const rules = $derived({ connected, checked: checked !== null });
	const steps = $derived(marks(stage, rules));
	const last = $derived(stage.step === 4);
	const store = keychainName(platformOf(typeof navigator === 'undefined' ? undefined : navigator));

	function forward() {
		const after = next(stage, rules);
		if (after) stage = after;
	}

	async function finish() {
		if (finishing) return;
		finishing = true;
		try {
			await workbench.setFollowed(followed);
			// Written whole, over what is stored: `choose` would skip it while
			// the settings are busy, and the order would be lost.
			if (!settings.loaded) await settings.load().catch(() => {});
			if (settings.loaded) await settings.write({ ...settings.settings, home: layout });
			await workbench.finishOnboarding();
			onboard.done();
			await goto('/home');
		} finally {
			finishing = false;
		}
	}

	async function later() {
		onboard.done();
		await goto('/home');
	}

	function keydown(event: KeyboardEvent) {
		if (event.target instanceof HTMLInputElement) return;
		if (event.key === 'ArrowRight' && stage.step === 1) forward();
		if (event.key === 'ArrowLeft' && stage.step === 1) stage = back(stage, rules);
	}
</script>

<svelte:window onkeydown={keydown} />

<div class="onboarding">
	<header class="top">
		<div class="brand">
			<span class="org">{ORG}</span>
			<Wordmark size={22} />
		</div>
		<ol class="steps" aria-label="Setting up {PRODUCT_NAME}">
			{#each STEPS as label, index (label)}
				{@const mark = steps[index]}
				<li aria-current={mark === 'current' ? 'step' : undefined} class={mark}>
					{#if index > 0}<span class="bar" aria-hidden="true"></span>{/if}
					<span class="dot">
						{#if mark === 'done'}<Icon name="check" size="0.9em" />{:else}{index + 1}{/if}
					</span>
					<span class="label">{label}</span>
				</li>
			{/each}
		</ol>
		{#if connected}<span class="note">Connected as @{connectedAs}</span>{/if}
	</header>

	<main class="body">
		<div class="frame">
			{#if stage.step === 1 && stage.page === 0}
				<section class="welcome" aria-labelledby="w1">
					<span class="eyebrow">Welcome</span>
					<h1 id="w1" class="hero"><Wordmark size={96} /></h1>
					<p class="lead">{SPLASH_LINE}</p>
				</section>
			{:else if stage.step === 1 && stage.page === 1}
				<section class="page" aria-labelledby="w2">
					<span class="eyebrow">Welcome · 2 of 3</span>
					<h2 id="w2">Home follows your repositories</h2>
					<ul class="points">
						<li><Icon name="bell" size="1.2em" /><span><strong>Needs you</strong><br />Reviews, failed pipelines and issues assigned to you.</span></li>
						<li><Icon name="pipelines" size="1.2em" /><span><strong>Pipelines and merge requests</strong><br />For every repository you follow, without cloning.</span></li>
						<li><Icon name="calls" size="1.2em" /><span><strong>Calls</strong><br />No repository needed.</span></li>
					</ul>
				</section>
			{:else if stage.step === 1}
				<section class="page" aria-labelledby="w3">
					<span class="eyebrow">Welcome · 3 of 3</span>
					<h2 id="w3">Load a repository when you need one</h2>
					<ul class="points">
						<li><Icon name="folder" size="1.2em" /><span><strong>Load with Ctrl+O</strong><br />From this machine, or cloned from CIB GitLab.</span></li>
						<li><Icon name="graph" size="1.2em" /><span><strong>Repository tools act on it</strong><br />Graph, working copy, branches, rebase, conflicts.</span></li>
						<li><Icon name="swap" size="1.2em" /><span><strong>Switch any time</strong><br />Loading another replaces it.</span></li>
					</ul>
				</section>
			{:else if stage.step === 2}
				<section class="page" aria-labelledby="s2">
					<span class="eyebrow">Step 2 of 4 · GitLab</span>
					<h2 id="s2">Connect to CIB GitLab</h2>
					<label class="field">
						<span class="label">GitLab address</span>
						<input type="text" class="mono" readonly value={GITLAB.host} title="Set in this build" />
					</label>
					<TokenPage guide />
				</section>
			{:else if stage.step === 3}
				<section class="page split" aria-labelledby="s3">
					<div class="grow">
						<span class="eyebrow">Step 3 of 4 · Token</span>
						<h2 id="s3">Paste your token</h2>
						<TokenField done={checked} onconnected={(info) => (checked = info)} />
					</div>
					<aside class="card store" aria-label="Where the token is kept">
						<span class="shield cc" class:saved={checked}><Icon name="lock" size="1.4em" /></span>
						<h3>{checked ? "Saved to your system's secure store" : "It goes into your system's secure store"}</h3>
						<p class="note">{store}</p>
						<ul>
							<li><Icon name="check" size="1em" />Never in {PRODUCT_NAME}'s settings or logs</li>
							<li><Icon name="check" size="1em" />Sent only to <span class="mono">{GITLAB.host}</span></li>
							<li><Icon name="check" size="1em" />Replace or remove it in Settings › GitLab</li>
						</ul>
					</aside>
				</section>
			{:else}
				<section class="page" aria-labelledby="s4">
					<span class="eyebrow">Step 4 of 4 · Home</span>
					<h2 id="s4">Set up your Home</h2>
					<div class="cards">
						<div class="card pane wide"><FollowList selected={followed} onchange={(next) => (followed = next)} /></div>
						<div class="card pane"><SectionsOrder {layout} onchange={(next) => (layout = next)} /></div>
					</div>
				</section>
			{/if}
		</div>
	</main>

	<footer class="foot">
		<div class="nav">
			<span class="side">
				{#if !(stage.step === 1 && stage.page === 0)}
					<Btn onclick={() => (stage = back(stage, rules))}>Back</Btn>
				{/if}
				{#if stage.step === 2 || stage.step === 3}
					<Btn quiet onclick={later}>Set up later</Btn>
				{/if}
			</span>
			{#if stage.step === 1}
				<div class="dots" role="group" aria-label="Welcome pages">
					{#each Array.from({ length: WELCOME_PAGES }, (_, i) => i) as index (index)}
						<button
							type="button"
							aria-label="Welcome page {index + 1} of {WELCOME_PAGES}"
							aria-current={index === stage.page ? 'page' : undefined}
							onclick={() => (stage = { step: 1, page: index })}><span class:on={index === stage.page}></span></button
						>
					{/each}
				</div>
			{/if}
			<span class="side end">
				{#if last}
					<Btn primary busy={finishing} disabled={finishing} onclick={finish}>Open {PRODUCT_NAME}</Btn>
				{:else}
					<Btn primary disabled={!canGoOn(stage, rules)} onclick={forward}>
						{stage.step === 2 ? 'I have my token' : 'Next'}
					</Btn>
				{/if}
			</span>
		</div>
		<div class="legal">
			<img class="logo" src={theme.isDark ? BRAND_ASSETS.logoReversed : BRAND_ASSETS.logoColor} alt={ORG} draggable="false" />
			<p class="notice"><span class="badge">{NOTICE.badge}</span>{NOTICE.lead}</p>
		</div>
	</footer>
</div>

<style>
	.onboarding {
		flex: 1;
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		overflow-y: auto;
		background: radial-gradient(1100px 640px at 50% 42%, color-mix(in srgb, var(--primary) 26%, transparent), transparent 70%);
		border-radius: var(--r-pane);
	}

	.top {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 16px 32px;
		padding: 20px 40px;
	}

	.brand {
		display: flex;
		align-items: baseline;
		gap: 12px;
	}

	.org {
		font-size: var(--fs-mono);
		font-weight: 600;
		letter-spacing: 0.3em;
		color: var(--muted);
	}

	.steps {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px;
	}

	.steps li {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--muted);
	}

	.bar {
		width: 28px;
		height: 2px;
		border-radius: var(--r-pill);
		background: var(--soft);
	}

	.steps li.done .bar,
	.steps li.current .bar {
		background: var(--ok);
	}

	.dot {
		width: 28px;
		height: 28px;
		box-sizing: border-box;
		border-radius: 50%;
		display: grid;
		place-items: center;
		font-size: var(--fs-mono);
		font-weight: 600;
		border: 1.5px solid var(--line);
	}

	.steps li.done .dot {
		color: var(--ok);
		border: 2px solid var(--ok);
	}

	.steps li.done {
		color: var(--ink);
	}

	.steps li.current .dot {
		color: var(--accent);
		border: 2px solid var(--accent);
		box-shadow: 0 0 0 5px var(--accent-soft);
	}

	.steps li.current .label {
		color: var(--ink);
		font-weight: 600;
	}

	.body {
		flex: 1 0 auto;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 12px 40px 20px;
	}

	.frame {
		width: 100%;
		max-width: 1120px;
	}

	.welcome {
		display: flex;
		flex-direction: column;
		align-items: center;
		text-align: center;
		gap: 16px;
	}

	.hero {
		margin: 0;
		line-height: 1;
	}

	.lead {
		margin: 0;
		font-size: calc(var(--fs-ui) * 1.3);
		color: var(--muted);
	}


	.page {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.page.split {
		flex-direction: row;
		flex-wrap: wrap;
		align-items: flex-start;
		gap: 32px 56px;
	}

	.grow {
		flex: 1 1 460px;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.eyebrow {
		font-size: var(--fs-secondary);
		font-weight: 600;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--accent);
	}

	h2 {
		margin: 0;
		font-size: calc(var(--fs-title) * 2.2);
		line-height: 1.15;
		font-weight: 600;
		letter-spacing: -0.02em;
	}

	.points {
		list-style: none;
		margin: 6px 0 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 14px;
		max-width: 620px;
	}

	.points li {
		display: flex;
		gap: 14px;
		align-items: flex-start;
		color: var(--muted);
	}

	.points li :global(svg) {
		flex: none;
		color: var(--accent);
		margin-top: 2px;
	}

	.points strong {
		color: var(--ink);
		font-weight: 600;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-width: 560px;
	}

	.label {
		font-weight: 600;
		font-size: var(--fs-secondary);
	}

	.store {
		flex: 1 1 300px;
		padding: 22px;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.store h3 {
		margin: 0;
		font-size: calc(var(--fs-ui) * 1.15);
		font-weight: 600;
	}

	.store p {
		margin: 0;
	}

	.store ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 8px;
		font-size: var(--fs-secondary);
	}

	.store li {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.store li :global(svg) {
		color: var(--ok);
		flex: none;
	}

	.shield {
		width: 44px;
		height: 44px;
		border-radius: var(--r-panel);
		display: grid;
		place-items: center;
		background: var(--accent-soft);
		color: var(--accent);
	}

	.shield.saved {
		background: var(--ok-soft);
		color: var(--ok);
	}

	.cards {
		display: flex;
		flex-wrap: wrap;
		gap: 20px;
	}

	.pane {
		flex: 1 1 360px;
		min-width: 0;
		padding: 18px;
	}

	.pane.wide {
		flex-basis: 500px;
	}

	.foot {
		padding: 0 40px 20px;
	}

	.nav,
	.legal {
		max-width: 1120px;
		margin: 0 auto;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
	}

	.legal {
		flex-wrap: wrap;
		margin-top: 16px;
		padding-top: 14px;
		border-top: 1px solid var(--soft);
	}

	.side {
		flex: 1;
		display: flex;
		gap: 8px;
	}

	.side.end {
		justify-content: flex-end;
	}

	.dots {
		display: flex;
		gap: 4px;
	}

	.dots button {
		width: 28px;
		height: 28px;
		display: grid;
		place-items: center;
	}

	.dots span {
		display: block;
		width: 8px;
		height: 8px;
		border-radius: var(--r-pill);
		background: var(--line);
		transition: width var(--t-fast) var(--ease);
	}

	.dots span.on {
		width: 22px;
		background: var(--accent);
	}

	.logo {
		display: block;
		height: 26px;
		width: auto;
	}

	.notice {
		margin: 0;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
		font-size: var(--fs-secondary);
		color: var(--muted);
	}

	.badge {
		padding: 2px 8px;
		border-radius: var(--r-pill);
		border: 1px solid var(--line);
		color: var(--ink);
		font-size: calc(var(--fs-mono) * 0.9);
		font-weight: 600;
		letter-spacing: 0.04em;
	}

	@media (max-width: 700px) {
		.top,
		.body,
		.foot {
			padding-left: 16px;
			padding-right: 16px;
		}
	}
</style>
