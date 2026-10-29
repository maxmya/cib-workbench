// SPDX-License-Identifier: MIT

/**
 * Workbench-wide state (2.0.0): whether onboarding finished, the projects
 * Home follows, where *Clone and load* clones into, and who is signed in to
 * CIB GitLab.
 *
 * The token is never here. `connect` passes it straight to the backend and
 * keeps only the answer, which carries none.
 */

import * as core from '$lib/api';
import { GITLAB } from '$lib/brand';
import * as api from './api';
import { asFailure } from './gitlab';
import type { ConnectFailure, Followed, Project, TokenInfo, WorkbenchState } from './types';

const EMPTY: WorkbenchState = { onboarded: null, followed: [], cloneInto: null };

let stored = $state<WorkbenchState>(EMPTY);
let loaded = $state(false);
/** The login connected for CIB GitLab, from the accounts file: no request. */
let login = $state<string | null>(null);
/** What GitLab says about the stored token, once asked. */
let account = $state<TokenInfo | null>(null);
let accountError = $state<ConnectFailure | null>(null);
let reading = $state(false);
let error = $state<string | null>(null);

export type Connected = { ok: true; info: TokenInfo } | { ok: false; failure: ConnectFailure };

export function followedOf(project: Project): Followed {
	return {
		host: GITLAB.host,
		id: project.id,
		path: project.path,
		defaultBranch: project.defaultBranch,
		httpUrl: project.httpUrl || null
	};
}

export const workbench = {
	get loaded(): boolean {
		return loaded;
	},
	get onboarded(): boolean {
		return stored.onboarded !== null;
	},
	get followed(): Followed[] {
		return stored.followed;
	},
	get cloneInto(): string | null {
		return stored.cloneInto;
	},
	/** The login on CIB GitLab, or null when nothing is connected. */
	get login(): string | null {
		return login;
	},
	get account(): TokenInfo | null {
		return account;
	},
	get accountError(): ConnectFailure | null {
		return accountError;
	},
	get reading(): boolean {
		return reading;
	},
	get error(): string | null {
		return error;
	},

	isFollowed(id: number): boolean {
		return stored.followed.some((f) => f.id === id);
	},

	/** Read the state file and who is connected. No request leaves the machine. */
	async load(): Promise<void> {
		if (!core.inTauri()) {
			loaded = true;
			return;
		}
		const [state, accounts] = await Promise.allSettled([api.state(), core.forgeAccounts()]);
		if (state.status === 'fulfilled') stored = state.value;
		if (accounts.status === 'fulfilled') {
			login = accounts.value.find((a) => a.host === GITLAB.host)?.user ?? null;
		}
		loaded = true;
	},

	/** Ask GitLab who the stored token belongs to (Settings › GitLab). */
	async readAccount(): Promise<void> {
		if (!core.inTauri() || reading) return;
		reading = true;
		try {
			account = await api.gitlabTokenInfo();
			accountError = null;
			if (account) login = account.login;
		} catch (cause) {
			accountError = asFailure(cause);
		} finally {
			reading = false;
		}
	},

	/**
	 * Check a token and keep it. The caller clears its field either way; the
	 * token is not held here.
	 */
	async connect(token: string): Promise<Connected> {
		try {
			const info = await api.gitlabConnect(token);
			account = info;
			accountError = null;
			login = info.login;
			return { ok: true, info };
		} catch (cause) {
			return { ok: false, failure: asFailure(cause) };
		}
	},

	/** Forget the token, keychain first, then the account. */
	async disconnect(): Promise<void> {
		if (!login) return;
		try {
			await core.forgeDisconnect(GITLAB.host, login);
			login = null;
			account = null;
			accountError = null;
		} catch (cause) {
			error = String(cause);
		}
	},

	async setFollowed(next: Followed[]): Promise<void> {
		const previous = stored;
		stored = { ...stored, followed: next };
		try {
			stored = await api.setFollowed(next);
			error = null;
		} catch (cause) {
			stored = previous;
			error = String(cause);
		}
	},

	async toggleFollowed(project: Project): Promise<void> {
		const next = this.isFollowed(project.id)
			? stored.followed.filter((f) => f.id !== project.id)
			: [...stored.followed, followedOf(project)];
		await this.setFollowed(next);
	},

	async finishOnboarding(): Promise<void> {
		stored = await api.setOnboarded(true);
	},

	/** Settings › Behaviour › *Run setup again*. Accounts are kept. */
	async resetOnboarding(): Promise<void> {
		stored = await api.setOnboarded(false);
	},

	async setCloneInto(path: string): Promise<void> {
		stored = await api.setCloneInto(path);
	},

	/** For tests. */
	present(next: Partial<WorkbenchState> & { login?: string | null; account?: TokenInfo | null }): void {
		const { login: who, account: info, ...state } = next;
		stored = { ...stored, ...state };
		if (who !== undefined) login = who;
		if (info !== undefined) account = info;
		loaded = true;
	},

	/** For tests. */
	clear(): void {
		stored = EMPTY;
		loaded = false;
		login = null;
		account = null;
		accountError = null;
		reading = false;
		error = null;
	}
};
