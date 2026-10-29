// SPDX-License-Identifier: MIT

/**
 * Mirrors `spagitty_core::workbench` and `src-tauri/src/workbench.rs`
 * (2.0.0, FEAT-180 to FEAT-184). Nothing here carries a token.
 */

import type { Issue } from '$lib/issues/types';
import type { Run } from '$lib/pipelines/types';
import type { Release } from '$lib/releases/types';

export interface TokenInfo {
	name: string;
	login: string;
	scopes: string[];
	/** False on a GitLab older than 15.5: the login is all there is. */
	scopesKnown: boolean;
	/** `YYYY-MM-DD`, or null for a token that does not expire. */
	expiresAt: string | null;
}

/** Why a token was not kept. */
export type ConnectFailure =
	| { kind: 'unauthorized'; detail: string }
	| { kind: 'unreachable'; detail: string }
	| { kind: 'missingScope'; scopes: string[] }
	| { kind: 'keychain'; detail: string }
	| { kind: 'other'; detail: string };

export interface Project {
	id: number;
	/** `group/sub/name`. */
	path: string;
	name: string;
	description: string;
	/** Unix seconds. */
	lastActivity: number;
	defaultBranch: string | null;
	httpUrl: string;
	webUrl: string;
}

export interface ProjectPage {
	projects: Project[];
	page: number;
	more: boolean;
}

export interface Followed {
	host: string;
	id: number;
	path: string;
	defaultBranch?: string | null;
	httpUrl?: string | null;
}

export interface Onboarded {
	version: number;
	at: string;
}

export interface WorkbenchState {
	onboarded: Onboarded | null;
	followed: Followed[];
	cloneInto: string | null;
}

export interface MergeRequest {
	id: number;
	iid: number;
	title: string;
	author: string;
	/** Unix seconds. */
	updated: number;
	sourceBranch: string;
	targetBranch: string;
	draft: boolean;
	mine: boolean;
	reviewRequested: boolean;
	conflicts: boolean;
	url: string;
}

export interface InProject<T> {
	projectId: number;
	project: string;
	item: T;
}

export interface ProjectError {
	/** 0 is a read across the host. */
	projectId: number;
	project: string;
	card: 'requests' | 'pipelines' | 'issues' | 'releases';
	detail: string;
}

export interface WideHome {
	requests: InProject<MergeRequest>[];
	pipelines: InProject<Run>[];
	issues: InProject<Issue>[];
	releases: InProject<Release>[];
	errors: ProjectError[];
	read: Record<string, number>;
	readWide: number;
}
