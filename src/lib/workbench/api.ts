// SPDX-License-Identifier: MIT

/** The Workbench-wide commands (2.0.0). A token goes one way, to `gitlabConnect`. */

import { invoke } from '@tauri-apps/api/core';
import type { Followed, ProjectPage, TokenInfo, WideHome, WorkbenchState } from './types';

export function state(): Promise<WorkbenchState> {
	return invoke('workbench_state');
}

export function setOnboarded(done: boolean): Promise<WorkbenchState> {
	return invoke('workbench_set_onboarded', { done, at: new Date().toISOString() });
}

export function setFollowed(followed: Followed[]): Promise<WorkbenchState> {
	return invoke('workbench_set_followed', { followed });
}

export function setCloneInto(path: string): Promise<WorkbenchState> {
	return invoke('workbench_set_clone_into', { path });
}

/** Rejects with a `ConnectFailure`. */
export function gitlabConnect(token: string): Promise<TokenInfo> {
	return invoke('gitlab_connect', { token });
}

/** Null when no account is connected for CIB GitLab. Rejects with a `ConnectFailure`. */
export function gitlabTokenInfo(): Promise<TokenInfo | null> {
	return invoke('gitlab_token_info');
}

export function gitlabProjects(search: string | null, page: number): Promise<ProjectPage> {
	return invoke('gitlab_projects', { search, page });
}

export function gitlabProjectsCached(search: string | null, page: number): Promise<ProjectPage | null> {
	return invoke('gitlab_projects_cached', { search, page });
}

/** Remembered repositories that are clones of CIB GitLab projects. */
export function gitlabLocalClones(): Promise<{ path: string; project: string }[]> {
	return invoke('gitlab_local_clones');
}

export function homeWide(force: boolean): Promise<WideHome> {
	return invoke('home_workbench_wide', { force });
}

export function homeWideCached(): Promise<WideHome | null> {
	return invoke('home_workbench_wide_cached');
}
