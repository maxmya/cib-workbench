// SPDX-License-Identifier: MIT

/**
 * The CIB GitLab projects a person is a member of (FEAT-182), for the follow
 * list and Load repository.
 *
 * The search runs in Rust; this only waits for the typing to stop. A search
 * that was read before paints at once from the cache, then is read again.
 */

import * as api from './api';
import type { Project, ProjectPage } from './types';

/** How long the typing has to stop for before a search is sent. */
export const SEARCH_DEBOUNCE_MS = 250;

export class ProjectList {
	query = $state('');
	projects = $state<Project[]>([]);
	page = $state(1);
	more = $state(false);
	loading = $state(false);
	error = $state<string | null>(null);

	#asked = 0;
	#timer: ReturnType<typeof setTimeout> | null = null;
	#read: typeof api.gitlabProjects;
	#cached: typeof api.gitlabProjectsCached;

	constructor(read = api.gitlabProjects, cached = api.gitlabProjectsCached) {
		this.#read = read;
		this.#cached = cached;
	}

	/** Type into the search. The read waits for a pause. */
	search(next: string): void {
		this.query = next;
		if (this.#timer) clearTimeout(this.#timer);
		this.#timer = setTimeout(() => void this.reload(), SEARCH_DEBOUNCE_MS);
	}

	/** The first page of the current search: the cache first, then GitLab. */
	async reload(): Promise<void> {
		if (this.#timer) clearTimeout(this.#timer);
		this.#timer = null;
		const ticket = ++this.#asked;
		const search = this.query.trim() || null;
		this.loading = true;
		try {
			const saved = await this.#cached(search, 1).catch(() => null);
			if (saved && ticket === this.#asked) this.#show(saved, false);
			const page = await this.#read(search, 1);
			if (ticket !== this.#asked) return;
			this.#show(page, false);
			this.error = null;
		} catch (cause) {
			if (ticket === this.#asked) this.error = String(cause);
		} finally {
			if (ticket === this.#asked) this.loading = false;
		}
	}

	/** The next page, after what is shown. */
	async next(): Promise<void> {
		if (!this.more || this.loading) return;
		const ticket = ++this.#asked;
		this.loading = true;
		try {
			const page = await this.#read(this.query.trim() || null, this.page + 1);
			if (ticket === this.#asked) this.#show(page, true);
		} catch (cause) {
			if (ticket === this.#asked) this.error = String(cause);
		} finally {
			if (ticket === this.#asked) this.loading = false;
		}
	}

	dispose(): void {
		if (this.#timer) clearTimeout(this.#timer);
		this.#asked += 1;
	}

	#show(page: ProjectPage, append: boolean): void {
		const known = new Set(append ? this.projects.map((p) => p.id) : []);
		const fresh = page.projects.filter((p) => !known.has(p.id));
		this.projects = append ? [...this.projects, ...fresh] : page.projects;
		this.page = page.page;
		this.more = page.more;
	}
}
