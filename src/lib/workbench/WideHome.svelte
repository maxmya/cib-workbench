<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import * as api from '$lib/api';
	import { shownCards } from '$lib/home/layout';
	import NeedsCard from '$lib/home/NeedsCard.svelte';
	import { age, type Need } from '$lib/home/needs';
	import { follow as openLink } from '$lib/notifications/act';
	import { repo } from '$lib/repo.svelte';
	import { settings } from '$lib/settings/store.svelte';
	import Btn from '$lib/ui/Btn.svelte';
	import Loader from '$lib/ui/Loader.svelte';
	import { WIDE_HOME_CARDS } from './cards';
	import { errorsOf, inFollowed, onlyProject, shortName, wideNeeds } from './home';
	import LoadedBand from './LoadedBand.svelte';
	import { loader } from './loader.svelte';
	import { workbench } from './state.svelte';
	import type { ProjectError } from './types';
	import { wideHome } from './wide-home.svelte';
	import '$lib/home/home.css';

	/**
	 * Home across followed repositories (2A, FEAT-183): what needs you, merge
	 * requests, pipelines, issues and releases from every project followed on
	 * CIB GitLab, and Workbench-wide Calls — with nothing loaded. While a
	 * repository is loaded, its own cards sit in a band above.
	 */

	const connected = $derived(workbench.login !== null);
	let only = $state(false);
	/** The loaded repository's path on CIB GitLab, when it is there. */
	let loadedSlug = $state<string | null>(null);

	onMount(() => {
		void workbench.load().then(() => wideHome.open(workbench.login !== null));
		return wideHome.follow(settings.settings.notifyEveryMinutes ?? 2, () => workbench.login !== null);
	});

	// Which project is loaded, to filter and to know when a row's project is here.
	$effect(() => {
		void repo.generation;
		const info = repo.info;
		untrack(() => {
			only = false;
			loadedSlug = null;
			if (!info || !api.inTauri()) return;
			void api
				.forgeRepo()
				.then((found) => (loadedSlug = found ? `${found.owner}/${found.name}`.toLowerCase() : null))
				.catch(() => (loadedSlug = null));
		});
	});

	/** The loaded project's id, from what is followed or from any row naming it. */
	const loadedId = $derived.by(() => {
		if (!loadedSlug) return null;
		const home = wideHome.home;
		const match = (path: string) => path.toLowerCase() === loadedSlug;
		return (
			workbench.followed.find((f) => match(f.path))?.id ??
			[...home.requests, ...home.pipelines, ...home.issues, ...home.releases].find((row) => match(row.project))?.projectId ??
			null
		);
	});

	const shown = $derived(onlyProject(wideHome.home, only ? loadedId : null));
	const needs = $derived(wideNeeds(shown));
	const requests = $derived(inFollowed(shown.requests, workbench.followed));
	const issues = $derived(inFollowed(shown.issues, workbench.followed));
	const empty = $derived(connected && workbench.loaded && workbench.followed.length === 0);
	/** Not connected, only Calls has anything to show; following nothing, Needs you and Calls. */
	const cards = $derived(
		shownCards(settings.home, WIDE_HOME_CARDS).filter((card) =>
			!connected ? card.id === 'calls' : !empty || card.id === 'needs' || card.id === 'calls'
		)
	);
	const now = $derived.by(() => {
		void wideHome.home;
		void wideHome.calls;
		return Date.now();
	});
	const updated = $derived(wideHome.updated);
	const repoName = $derived(repo.info?.name ?? '');

	/** A row's screen: here when its project is loaded, else load it first. */
	function openIn(projectId: number, route: string) {
		if (loadedId === projectId) void goto(route);
		else loader.show({ project: projectId, then: route });
	}

	function act(need: Need) {
		const source = need.source;
		if (source.kind === 'link') openLink({ kind: 'url', url: source.url });
		else if (source.kind === 'run' && source.run.url) openLink({ kind: 'url', url: source.run.url });
		else if (source.kind === 'issue' && source.issue.url) openLink({ kind: 'url', url: source.issue.url });
	}

	function duration(seconds: number | null): string {
		if (seconds === null) return '';
		const m = Math.floor(seconds / 60);
		return m ? `${m}m ${seconds % 60}s` : `${seconds}s`;
	}

	const GLYPH: Record<string, { glyph: string; tone: string; label: string }> = {
		success: { glyph: '✓', tone: 'ok', label: 'Passed' },
		failed: { glyph: '✕', tone: 'danger', label: 'Failed' },
		canceled: { glyph: '–', tone: 'muted', label: 'Cancelled' },
		skipped: { glyph: '–', tone: 'muted', label: 'Skipped' }
	};
	const RUNNING = { glyph: '●', tone: 'accent', label: 'Running' };
	const QUEUED = { glyph: '○', tone: 'muted', label: 'Queued' };
</script>

{#snippet failures(list: ProjectError[])}
	{#each list as failure (`${failure.projectId}:${failure.card}:${failure.detail}`)}
		<p class="home-error">{failure.project ? `${shortName(failure.project)}: ` : ''}{failure.detail}</p>
	{/each}
{/snippet}

{#snippet chip(project: string)}
	<span class="chip mono" title={project}>{shortName(project)}</span>
{/snippet}

<div class="home">
	<header class="head">
		<div class="who">
			<h1>Home</h1>
			<p class="home-quiet line">
				{#if connected}
					Following {workbench.followed.length}
					{workbench.followed.length === 1 ? 'repository' : 'repositories'} on CIB GitLab{#if updated}
						· updated {age(updated, now)} ago{/if} ·
					<button class="home-link" onclick={() => goto('/settings#home')}>Change</button>
				{:else}
					Not connected to CIB GitLab ·
					<button class="home-link" onclick={() => goto('/settings#gitlab')}>Connect</button>
				{/if}
			</p>
		</div>
		<div class="actions">
			{#if repo.info && loadedId !== null}
				<div class="segmented" role="group" aria-label="Show">
					<button aria-pressed={!only} class:on={!only} onclick={() => (only = false)}>All followed</button>
					<button aria-pressed={only} class:on={only} onclick={() => (only = true)}>{repoName} only</button>
				</div>
			{/if}
			<Btn busy={wideHome.loading} disabled={wideHome.loading} title="Read everything again" onclick={() => wideHome.refresh(connected)}
				>Refresh</Btn
			>
			<Btn onclick={() => goto('/settings#home')}>Customise Home</Btn>
		</div>
	</header>

	{#if wideHome.error}<p class="home-error">{wideHome.error}</p>{/if}

	{#if repo.info}
		<LoadedBand project={loadedSlug ? (workbench.followed.find((f) => f.id === loadedId)?.path ?? loadedSlug) : null} {now} />
	{/if}

	{#if empty}
		<section class="card empty" aria-labelledby="empty-heading">
			<div>
				<h2 id="empty-heading">You're not following any repositories</h2>
			</div>
			<div class="actions">
				<Btn primary onclick={() => goto('/settings#home')}>Choose repositories</Btn>
				<Btn onclick={() => loader.show()}>Load a repository</Btn>
			</div>
		</section>
	{/if}

	<div class="grid">
		{#each cards as card (card.id)}
			<div class="slot" class:wide={card.wide} data-card={card.id}>
				{#if card.id === 'needs'}
					<NeedsCard rows={needs} {now} onact={act} />
				{:else if card.id === 'requests'}
					<section class="card home-card" aria-labelledby="wide-requests">
						<div class="home-card-head"><h2 id="wide-requests">Merge requests</h2><span class="note">{requests.length}</span></div>
						{@render failures(errorsOf(shown, 'requests'))}
						{#each requests as { projectId, project, item } (item.id)}
							<button class="row home-row" onclick={() => openIn(projectId, '/requests')} title="!{item.iid} in {project}">
								<span class="what home-ellipsis"><span class="mono muted">!{item.iid}</span> {item.title}</span>
								{@render chip(project)}
								<span class="state">{item.draft ? 'Draft' : item.conflicts ? 'Conflicts' : item.mine ? 'Yours' : item.reviewRequested ? 'Review' : 'Open'}</span>
							</button>
						{:else}
							<p class="home-quiet clear">No open merge requests.</p>
						{/each}
					</section>
				{:else if card.id === 'pipelines'}
					<section class="card home-card" aria-labelledby="wide-pipelines">
						<div class="home-card-head"><h2 id="wide-pipelines">Pipelines</h2></div>
						{@render failures(errorsOf(shown, 'pipelines'))}
						{#each shown.pipelines as { projectId, project, item } (`${projectId}:${item.id}`)}
							{@const look = item.state === 'running' ? RUNNING : item.state === 'queued' ? QUEUED : (GLYPH[item.result] ?? QUEUED)}
							<button class="row home-row" onclick={() => openIn(projectId, '/pipelines/live')}>
								<span class="glyph {look.tone}" role="img" aria-label={look.label}>{look.glyph}</span>
								<span class="what home-ellipsis">{shortName(project)} <span class="mono muted">{item.branch}</span></span>
								<span class="home-quiet">{duration(item.duration)}</span>
							</button>
						{:else}
							{#if wideHome.loading}<Loader size="inline" label="Reading…" />{:else}<p class="home-quiet clear">No pipelines.</p>{/if}
						{/each}
					</section>
				{:else if card.id === 'issues'}
					<section class="card home-card" aria-labelledby="wide-issues">
						<div class="home-card-head"><h2 id="wide-issues">Issues</h2><span class="note">assigned to you</span></div>
						{@render failures(errorsOf(shown, 'issues'))}
						{#each issues as { projectId, project, item } (item.id)}
							<button class="row home-row" onclick={() => openIn(projectId, '/issues')}>
								<span class="what home-ellipsis"><span class="mono muted">#{item.number}</span> {item.title}</span>
								{@render chip(project)}
								{#if item.labels[0]}<span class="state">{item.labels[0].name}</span>{/if}
							</button>
						{:else}
							<p class="home-quiet clear">Nothing assigned to you.</p>
						{/each}
					</section>
				{:else if card.id === 'calls'}
					<section class="card home-card" aria-labelledby="wide-calls">
						<div class="home-card-head">
							<h2 id="wide-calls">Calls</h2>
							<button class="home-link" onclick={() => goto('/calls')}>Open Calls</button>
						</div>
						{#if wideHome.callsError}<p class="home-error">{wideHome.callsError}</p>{/if}
						{#each wideHome.calls as collection (collection.stem)}
							<button class="row home-row" onclick={() => goto('/calls')}>
								<span class="what home-ellipsis">{collection.name}</span>
								<span class="home-quiet">
									{collection.requests} {collection.requests === 1 ? 'request' : 'requests'}{#if collection.used}
										· {age(collection.used, now)}{/if}
								</span>
								{#if collection.environment}<span class="state">{collection.environment}</span>{/if}
							</button>
						{:else}
							<p class="home-quiet clear">No collections yet.</p>
						{/each}
					</section>
				{:else if card.id === 'releases'}
					<section class="card home-card" aria-labelledby="wide-releases">
						<div class="home-card-head"><h2 id="wide-releases">Releases</h2></div>
						{@render failures(errorsOf(shown, 'releases'))}
						<div class="releases">
							{#each shown.releases as { projectId, project, item } (`${projectId}:${item.tag}`)}
								<button class="release" onclick={() => (item.url ? openLink({ kind: 'url', url: item.url }) : openIn(projectId, '/releases'))}>
									<span class="mono tag">{item.tag}</span>
									{#if item.prerelease}<span class="state">pre-release</span>{/if}
									<span class="home-quiet">{shortName(project)}{#if item.created} · {age(Date.parse(item.created), now)}{/if}</span>
								</button>
							{:else}
								<p class="home-quiet clear">No releases.</p>
							{/each}
						</div>
					</section>
				{/if}
			</div>
		{/each}
	</div>
</div>

<style>
	.home {
		flex: 1;
		min-width: 0;
		overflow-y: auto;
		padding: 26px 30px 30px;
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.head {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: 16px;
	}

	.who {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	h1 {
		margin: 0;
		font-size: calc(var(--fs-title) * 1.55);
		font-weight: 600;
		letter-spacing: -0.01em;
	}

	.line {
		margin: 0;
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px;
	}

	.segmented {
		display: inline-flex;
		padding: 2px;
		border-radius: var(--r-pill);
		border: 1px solid var(--line);
	}

	.segmented button {
		padding: 4px 12px;
		border-radius: var(--r-pill);
		color: var(--muted);
	}

	.segmented button.on {
		background: var(--accent-soft);
		color: var(--ink);
	}

	.empty {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 14px;
		padding: 20px 22px;
		border-style: dashed;
	}

	.empty h2 {
		margin: 0;
		font-size: calc(var(--fs-ui) * 1.15);
	}

	.grid {
		container-type: inline-size;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
		grid-auto-flow: row dense;
		gap: 18px;
	}

	.slot {
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	.slot > :global(.home-card) {
		flex: 1;
	}

	.slot.wide {
		grid-column: 1 / -1;
	}

	.row {
		display: flex;
		align-items: center;
		gap: 10px;
		width: 100%;
		padding: 9px 0;
		background: none;
		border-left: 0;
		border-right: 0;
		border-bottom: 0;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.row:hover .what {
		color: var(--accent);
	}

	.what {
		flex: 1;
		min-width: 0;
	}

	.muted {
		color: var(--muted);
	}

	.chip {
		flex: none;
		padding: 2px 8px;
		border-radius: var(--r-row);
		background: var(--soft);
		color: var(--muted);
	}

	.state {
		flex: none;
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.glyph {
		flex: none;
		width: 1.2em;
		text-align: center;
		font-weight: 700;
	}

	.glyph.ok {
		color: var(--ok);
	}

	.glyph.danger {
		color: var(--danger);
	}

	.glyph.accent {
		color: var(--accent);
	}

	.glyph.muted {
		color: var(--muted);
	}

	.clear {
		margin: 8px 0;
	}

	.releases {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 10px;
	}

	.release {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 6px 8px;
		padding: 10px 12px;
		border-radius: var(--r-row);
		background: var(--surface-2);
		text-align: left;
		color: inherit;
		font: inherit;
		cursor: pointer;
	}

	.release:hover .tag {
		color: var(--accent);
	}

	.tag {
		font-weight: 600;
	}

	@media (max-width: 700px) {
		.home {
			padding: 18px 16px;
		}
		.grid {
			grid-template-columns: minmax(0, 1fr);
		}
	}
</style>
