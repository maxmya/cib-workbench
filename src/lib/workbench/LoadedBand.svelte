<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import * as api from '$lib/api';
	import { pushCurrent, quickStash } from '$lib/graph/actions';
	import CommitsCard from '$lib/home/CommitsCard.svelte';
	import { age } from '$lib/home/needs';
	import type { Overview } from '$lib/home/types';
	import WorkingCard from '$lib/home/WorkingCard.svelte';
	import { repo } from '$lib/repo.svelte';
	import type { StatusEntry } from '$lib/types';
	import Btn from '$lib/ui/Btn.svelte';
	import { describe } from '$lib/ui/notice.svelte';
	import '$lib/home/home.css';

	/**
	 * The *Loaded* band (FEAT-183): only while a repository is loaded, its
	 * working copy, its branch against upstream, and its latest commits, from
	 * the cards Home already had. Read from git, so it needs no network.
	 */
	interface Props {
		/** `group/name` on CIB GitLab, when the loaded one is there. */
		project: string | null;
		now: number;
	}

	let { project, now }: Props = $props();

	let overview = $state<Overview | null>(null);
	let changed = $state<StatusEntry[]>([]);
	let error = $state<string | null>(null);
	let stashing = $state(false);
	let asked = 0;

	async function read() {
		if (!api.inTauri() || !repo.info) return;
		const ticket = ++asked;
		try {
			const [found, copy] = await Promise.all([api.homeOverview(), api.workingCopy().catch(() => null)]);
			if (ticket !== asked) return;
			overview = found;
			error = null;
			if (copy) {
				const seen = new Map<string, StatusEntry>();
				for (const entry of [...copy.conflicted, ...copy.staged, ...copy.unstaged]) {
					if (!seen.has(entry.path)) seen.set(entry.path, entry);
				}
				changed = [...seen.values()];
			}
		} catch (cause) {
			if (ticket === asked) error = describe(cause);
		}
	}

	// Again whenever the repository, HEAD or the counts move.
	$effect(() => {
		void repo.generation;
		void repo.info;
		void repo.counts;
		untrack(() => void read());
	});

	async function stash() {
		stashing = true;
		try {
			if (await quickStash()) await read();
		} finally {
			stashing = false;
		}
	}

	const info = $derived(repo.info);
	const upstream = $derived(overview?.upstream ?? null);
	const fetched = $derived(info?.lastFetched ? `fetched ${age(info.lastFetched * 1000, now)} ago` : 'never fetched');
</script>

{#if info}
	<section class="band card" aria-labelledby="loaded-heading">
		<div class="top">
			<h2 id="loaded-heading"><span class="tag">Loaded</span><span class="mono">{project ?? info.name}</span></h2>
			<div class="actions">
				<Btn onclick={() => goto('/')}>Open graph</Btn>
				<Btn primary onclick={() => goto('/changes')}>Commit…</Btn>
			</div>
		</div>
		{#if error}<p class="home-error">{error}</p>{/if}
		<div class="cards">
			<WorkingCard {changed} stashes={repo.counts.stashes} busy={stashing} oncommit={() => goto('/changes')} onstash={stash} />
			<div class="card home-card branch">
				<div class="home-card-head"><h2>Branch</h2></div>
				{#if info.head.branch}
					<span class="mono name">{info.head.branch}{#if upstream}<span class="muted"> → {upstream.upstream}</span>{/if}</span>
					<span class="home-quiet">
						{#if upstream}{upstream.ahead} to push, {upstream.behind} to pull · {/if}{fetched}
					</span>
					<div><Btn disabled={!upstream || upstream.ahead === 0} onclick={() => pushCurrent()}>Push</Btn></div>
				{:else}
					<span class="mono name">detached at {info.head.short}</span>
				{/if}
			</div>
			<CommitsCard {now} />
		</div>
	</section>
{/if}

<style>
	.band {
		padding: 16px 18px;
		display: flex;
		flex-direction: column;
		gap: 12px;
		border-color: color-mix(in srgb, var(--accent) 35%, transparent);
	}

	.top {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}

	h2 {
		margin: 0;
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: var(--fs-ui);
		font-weight: 600;
	}

	.tag {
		padding: 2px 8px;
		border-radius: var(--r-pill);
		background: var(--accent-soft);
		color: var(--accent);
		font-size: var(--fs-mono);
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 14px;
	}

	.branch {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.name {
		font-size: var(--fs-ui);
	}

	.muted {
		color: var(--muted);
	}
</style>
