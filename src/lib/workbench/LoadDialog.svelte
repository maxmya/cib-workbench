<!-- SPDX-License-Identifier: MIT -->
<script lang="ts">
	import { tick } from 'svelte';
	import * as core from '$lib/api';
	import { repo } from '$lib/repo.svelte';
	import type { RepoSummary } from '$lib/types';
	import Btn from '$lib/ui/Btn.svelte';
	import Icon from '$lib/ui/Icon.svelte';
	import { nameOf } from '$lib/workspace.svelte';
	import * as api from './api';
	import { loader } from './loader.svelte';
	import { ProjectList } from './projects.svelte';
	import { workbench } from './state.svelte';
	import type { Project } from './types';

	/**
	 * Load repository (2E, FEAT-184): one search across this machine and CIB
	 * GitLab. Loading replaces what is loaded. Esc, ↑/↓ and Enter are handled
	 * on the dialog, not on the window.
	 */

	interface Row {
		key: string;
		kind: 'local' | 'gitlab';
		title: string;
		sub: string;
		followed: boolean;
		loaded: boolean;
		run: () => void;
	}

	let query = $state('');
	let recents = $state<RepoSummary[]>([]);
	let clones = $state<{ path: string; project: string }[]>([]);
	let active = $state(0);
	let dialog = $state<HTMLElement | null>(null);
	let search = $state<HTMLInputElement | null>(null);
	let list = $state<ProjectList | null>(null);
	const projects = $derived(list?.projects ?? []);
	const projectsError = $derived(list?.error ?? null);
	let returnFocus: HTMLElement | null = null;

	$effect(() => {
		if (!loader.open) return;
		returnFocus = document.activeElement as HTMLElement | null;
		query = '';
		active = 0;
		const found = new ProjectList();
		list = found;
		void (async () => {
			const [mine, cloned] = await Promise.all([
				core.recentRepos().catch(() => [] as RepoSummary[]),
				workbench.login ? api.gitlabLocalClones().catch(() => []) : Promise.resolve([])
			]);
			recents = mine;
			clones = cloned;
		})();
		if (workbench.login) void found.reload();
		void tick().then(() => search?.focus());
		return () => {
			found.dispose();
			list = null;
			returnFocus?.focus?.();
		};
	});

	const parent = $derived(workbench.cloneInto ?? (recents[0] ? parentOf(recents[0].path) : null));

	function parentOf(path: string): string {
		const trimmed = path.replace(/[/\\]+$/, '');
		const cut = Math.max(trimmed.lastIndexOf('/'), trimmed.lastIndexOf('\\'));
		return cut > 0 ? trimmed.slice(0, cut) : trimmed;
	}

	function separator(path: string): string {
		return path.includes('\\') ? '\\' : '/';
	}

	function matches(...texts: (string | null | undefined)[]): boolean {
		const q = query.trim().toLowerCase();
		return !q || texts.some((text) => text?.toLowerCase().includes(q));
	}

	async function cloneAndLoad(project: Project) {
		let into = parent;
		if (!into) {
			const { open } = await import('@tauri-apps/plugin-dialog');
			const picked = await open({ directory: true, multiple: false, title: 'Clone into' });
			if (typeof picked !== 'string') return;
			into = picked;
		}
		if (into !== workbench.cloneInto) void workbench.setCloneInto(into);
		await loader.cloneAndLoad(project, into);
	}

	async function chooseParent() {
		const { open } = await import('@tauri-apps/plugin-dialog');
		const picked = await open({ directory: true, multiple: false, title: 'Clone into' });
		if (typeof picked === 'string') await workbench.setCloneInto(picked);
	}

	const groups = $derived.by(() => {
		const clonedOf = new Map(clones.map((c) => [c.project.toLowerCase(), c.path]));
		const followedIds = new Set(workbench.followed.map((f) => f.id));
		const local: Row[] = recents
			.filter((r) => r.present && matches(r.name, r.path, clones.find((c) => c.path === r.path)?.project))
			.map((r) => {
				const project = clones.find((c) => c.path === r.path)?.project;
				return {
					key: r.path,
					kind: 'local',
					title: project ?? r.name,
					sub: [r.path, r.branch].filter(Boolean).join(' · '),
					followed: !!project && workbench.followed.some((f) => f.path.toLowerCase() === project.toLowerCase()),
					loaded: repo.info?.path === r.path,
					run: () => void loader.load(r.path)
				};
			});

		// Followed first, then what GitLab lists; one row each, none already cloned.
		const seen = new Set<number>();
		const remote: Project[] = [];
		const followedProjects: Project[] = workbench.followed.map((f) => ({
			id: f.id,
			path: f.path,
			name: nameOf(f.path),
			description: '',
			lastActivity: 0,
			defaultBranch: f.defaultBranch ?? null,
			httpUrl: f.httpUrl ?? '',
			webUrl: ''
		}));
		for (const project of [...followedProjects, ...projects]) {
			if (seen.has(project.id)) continue;
			seen.add(project.id);
			const listed = projects.find((p) => p.id === project.id);
			const best = listed ?? project;
			if (!best.httpUrl || clonedOf.has(best.path.toLowerCase())) continue;
			if (!matches(best.path, best.description)) continue;
			remote.push(best);
		}
		if (loader.selected !== null) remote.sort((a, b) => Number(b.id === loader.selected) - Number(a.id === loader.selected));
		const gitlab: Row[] = remote.map((project) => ({
			key: `gitlab:${project.id}`,
			kind: 'gitlab',
			title: project.path,
			sub: parent ? `Clones into ${parent}${separator(parent)}${nameOf(project.path)}` : 'Not on this machine',
			followed: followedIds.has(project.id),
			loaded: false,
			run: () => void cloneAndLoad(project)
		}));

		// A Home row asked for a project that is already cloned: that row first.
		if (loader.selected !== null) {
			const wanted = workbench.followed.find((f) => f.id === loader.selected)?.path.toLowerCase();
			local.sort((a, b) => Number(b.title.toLowerCase() === wanted) - Number(a.title.toLowerCase() === wanted));
		}
		return [
			{ title: 'On this machine', rows: local },
			...(workbench.login ? [{ title: 'On CIB GitLab', rows: gitlab }] : [])
		];
	});

	const rows = $derived(groups.flatMap((g) => g.rows));

	$effect(() => {
		if (active >= rows.length) active = Math.max(0, rows.length - 1);
	});

	function keydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			loader.hide();
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			active = Math.min(rows.length - 1, active + 1);
			scrollToActive();
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			active = Math.max(0, active - 1);
			scrollToActive();
		} else if (event.key === 'Enter' && !(event.target instanceof HTMLButtonElement)) {
			event.preventDefault();
			if (!loader.busy) rows[active]?.run();
		} else if (event.key === 'Tab') {
			trap(event);
		}
	}

	function scrollToActive() {
		void tick().then(() => dialog?.querySelector('[data-active="true"]')?.scrollIntoView({ block: 'nearest' }));
	}

	/** Focus stays inside the dialog while it is open. */
	function trap(event: KeyboardEvent) {
		const focusable = [
			...(dialog?.querySelectorAll<HTMLElement>('input, button:not([disabled]), [tabindex="0"]') ?? [])
		];
		if (!focusable.length) return;
		const first = focusable[0];
		const last = focusable[focusable.length - 1];
		if (event.shiftKey && document.activeElement === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && document.activeElement === last) {
			event.preventDefault();
			first.focus();
		}
	}

	function typed(event: Event) {
		query = (event.currentTarget as HTMLInputElement).value;
		active = 0;
		list?.search(query);
	}
</script>

{#if loader.open}
	<div class="backdrop" role="presentation" onclick={() => loader.hide()}></div>
	<div
		class="dialog floating"
		role="dialog"
		aria-modal="true"
		aria-labelledby="load-heading"
		tabindex="-1"
		bind:this={dialog}
		onkeydown={keydown}
	>
		<h2 id="load-heading">Load a repository</h2>
		<div class="search">
			<Icon name="search" size="1.1em" />
			<label class="visually-hidden" for="load-search">Find a repository</label>
			<input
				id="load-search"
				type="search"
				placeholder={workbench.login ? 'Find a repository on this machine or on CIB GitLab' : 'Find a repository on this machine'}
				autocomplete="off"
				value={query}
				oninput={typed}
				bind:this={search}
				aria-controls="load-rows"
			/>
			<kbd class="kbd">Esc</kbd>
		</div>

		<div class="rows" id="load-rows">
			{#each groups as group (group.title)}
				<div class="group">
					<span class="group-title">{group.title}</span>
					{#each group.rows as row (row.key)}
						{@const index = rows.indexOf(row)}
						{@const busy = loader.busy?.key === row.key ? loader.busy : null}
						<button
							type="button"
							class="row"
							data-active={index === active}
							disabled={loader.busy !== null}
							onmouseenter={() => (active = index)}
							onclick={row.run}
						>
							<span class="icon" class:remote={row.kind === 'gitlab'}>
								<Icon name={row.kind === 'local' ? 'folder' : 'clone'} size="1.05em" />
							</span>
							<span class="text">
								<span class="title">
									<span class="mono">{row.title}</span>
									{#if row.loaded}<span class="badge">Loaded</span>{/if}
									{#if row.followed}<span class="badge quiet">Followed</span>{/if}
								</span>
								<span class="sub mono">{row.sub}</span>
								{#if busy}
									<span class="progress" role="progressbar" aria-label={busy.text} aria-valuenow={busy.percent ?? undefined}>
										<span class="bar" style:width={busy.percent === null ? '35%' : `${busy.percent}%`} class:indeterminate={busy.percent === null}></span>
									</span>
									<span class="sub">{busy.text}</span>
								{/if}
							</span>
							<span class="action">{row.kind === 'local' ? (row.loaded ? 'Loaded' : 'Load') : 'Clone and load'}</span>
						</button>
					{:else}
						<p class="none">
							{group.title === 'On CIB GitLab' && projectsError ? projectsError : query ? `Nothing matches “${query}”.` : 'Nothing here yet.'}
						</p>
					{/each}
				</div>
			{/each}
		</div>

		{#if loader.error}<p class="error" role="alert">{loader.error}</p>{/if}

		<footer class="foot">
			<Btn onclick={() => loader.loadFolder()} disabled={loader.busy !== null}>Open a folder…</Btn>
			{#if workbench.login}
				<span class="note into">
					Clone into <span class="mono">{parent ?? 'a folder you choose'}</span>
					<button class="link" onclick={chooseParent}>Change</button>
				</span>
			{/if}
		</footer>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 60;
		/* The scrim: dimmed and blurred behind a modal (FEAT-188). */
		background: var(--scrim);
		-webkit-backdrop-filter: var(--scrim-blur);
		backdrop-filter: var(--scrim-blur);
	}

	.dialog {
		position: fixed;
		z-index: 61;
		top: 12vh;
		left: 50%;
		transform: translateX(-50%);
		width: min(720px, calc(100vw - 32px));
		max-height: 72vh;
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 18px;
	}

	h2 {
		margin: 0;
		font-size: var(--fs-ui);
		font-weight: 600;
	}

	.search {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 0 12px;
		border-radius: var(--r-field);
		border: 1px solid var(--line);
		background: var(--sunken);
		color: var(--muted);
	}

	.search input {
		flex: 1;
		min-width: 0;
		border: 0;
		background: transparent;
		box-shadow: none;
		color: var(--ink);
	}

	.rows {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.group {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.group-title {
		padding: 6px 6px 4px;
		font-size: var(--fs-mono);
		font-weight: 600;
		letter-spacing: 0.06em;
		text-transform: uppercase;
		color: var(--muted);
	}

	.row {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		padding: 9px 12px;
		border-radius: var(--r-row);
		text-align: left;
		color: inherit;
		font: inherit;
	}

	.row[data-active='true'] {
		background: var(--accent-soft);
	}

	.row:disabled {
		cursor: default;
	}

	.icon {
		flex: none;
		color: var(--accent);
	}

	.icon.remote {
		color: var(--muted);
	}

	.text {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.title {
		display: flex;
		align-items: center;
		gap: 8px;
		font-weight: 600;
	}

	.sub {
		color: var(--muted);
		font-size: var(--fs-secondary);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.badge {
		padding: 1px 7px;
		border-radius: var(--r-pill);
		background: var(--accent-soft);
		color: var(--accent);
		font-size: var(--fs-mono);
		font-weight: 600;
	}

	.badge.quiet {
		background: var(--soft);
		color: var(--muted);
	}

	.action {
		flex: none;
		font-size: var(--fs-secondary);
		color: var(--accent);
	}

	.progress {
		display: block;
		height: 4px;
		border-radius: var(--r-pill);
		background: var(--soft);
		overflow: hidden;
	}

	.bar {
		display: block;
		height: 100%;
		background: var(--accent);
		transition: width var(--t-fast) var(--ease);
	}

	.bar.indeterminate {
		animation: slide 1.2s var(--ease) infinite;
	}

	@keyframes slide {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(300%);
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.bar.indeterminate {
			animation: none;
		}
	}

	.none {
		margin: 4px 6px;
		color: var(--muted);
		font-size: var(--fs-secondary);
	}

	.error {
		margin: 0;
		color: var(--danger);
		font-size: var(--fs-secondary);
	}

	.foot {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 10px;
	}

	.into {
		display: flex;
		align-items: baseline;
		gap: 6px;
		min-width: 0;
	}

	.link {
		color: var(--accent);
		font: inherit;
	}

	.link:hover {
		text-decoration: underline;
	}

	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		overflow: hidden;
		clip: rect(0 0 0 0);
	}
</style>
