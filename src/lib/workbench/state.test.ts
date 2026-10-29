// SPDX-License-Identifier: MIT

import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	inTauri: vi.fn(() => true),
	forgeAccounts: vi.fn(async () => [
		{ kind: 'gitHub', host: 'github.com', user: 'someone' },
		{ kind: 'gitLab', host: 'gitlab.apps.ocp-nonprod-01.hodomain.local', user: 'n.hassan' }
	]),
	forgeDisconnect: vi.fn(async () => [])
}));

vi.mock('./api', () => ({
	state: vi.fn(async () => ({ onboarded: { version: 1, at: 'x' }, followed: [], cloneInto: 'C:\\Projects' })),
	setOnboarded: vi.fn(async (done: boolean) => ({ onboarded: done ? { version: 1, at: 'y' } : null, followed: [], cloneInto: null })),
	setFollowed: vi.fn(async (followed: unknown[]) => ({ onboarded: null, followed, cloneInto: null })),
	setCloneInto: vi.fn(async (path: string) => ({ onboarded: null, followed: [], cloneInto: path })),
	gitlabConnect: vi.fn(),
	gitlabTokenInfo: vi.fn()
}));

import * as core from '$lib/api';
import * as api from './api';
import { workbench } from './state.svelte';
import type { Project } from './types';

const info = { name: 'Nour Hassan', login: 'n.hassan', scopes: ['api'], scopesKnown: true, expiresAt: null };
const project: Project = { id: 7, path: 'a/b', name: 'b', description: '', lastActivity: 0, defaultBranch: 'main', httpUrl: 'https://h/a/b.git', webUrl: '' };

beforeEach(() => {
	vi.clearAllMocks();
	workbench.clear();
});

describe('Workbench-wide state', () => {
	it('reads the state file and the login on CIB GitLab, with no request to GitLab', async () => {
		await workbench.load();
		expect(workbench.loaded).toBe(true);
		expect(workbench.onboarded).toBe(true);
		expect(workbench.cloneInto).toBe('C:\\Projects');
		expect(workbench.login).toBe('n.hassan');
		expect(api.gitlabTokenInfo).not.toHaveBeenCalled();
	});

	it('keeps the answer of a connect, never the token, and says why one failed', async () => {
		vi.mocked(api.gitlabConnect).mockResolvedValueOnce(info);
		expect(await workbench.connect('glpat-x')).toEqual({ ok: true, info });
		expect(workbench.login).toBe('n.hassan');
		expect(JSON.stringify(workbench.account)).not.toContain('glpat');

		vi.mocked(api.gitlabConnect).mockRejectedValueOnce({ kind: 'keychain', detail: 'locked' });
		expect(await workbench.connect('glpat-y')).toEqual({ ok: false, failure: { kind: 'keychain', detail: 'locked' } });
	});

	it('reads who the stored token belongs to, or why it could not', async () => {
		vi.mocked(api.gitlabTokenInfo).mockResolvedValueOnce(info);
		await workbench.readAccount();
		expect(workbench.account).toEqual(info);

		vi.mocked(api.gitlabTokenInfo).mockRejectedValueOnce({ kind: 'unreachable', detail: 'vpn' });
		await workbench.readAccount();
		expect(workbench.accountError).toEqual({ kind: 'unreachable', detail: 'vpn' });
	});

	it('disconnects the CIB GitLab account', async () => {
		workbench.present({ login: 'n.hassan', account: info });
		await workbench.disconnect();
		expect(core.forgeDisconnect).toHaveBeenCalledWith('gitlab.apps.ocp-nonprod-01.hodomain.local', 'n.hassan');
		expect(workbench.login).toBeNull();
		expect(workbench.account).toBeNull();

		await workbench.disconnect();
		expect(core.forgeDisconnect).toHaveBeenCalledTimes(1);
	});

	it('says when a disconnect failed', async () => {
		workbench.present({ login: 'n.hassan' });
		vi.mocked(core.forgeDisconnect).mockRejectedValueOnce('keychain locked');
		await workbench.disconnect();
		expect(workbench.error).toBe('keychain locked');
		expect(workbench.login).toBe('n.hassan');
	});

	it('follows and unfollows, and puts the list back when the write fails', async () => {
		await workbench.toggleFollowed(project);
		expect(workbench.isFollowed(7)).toBe(true);
		await workbench.toggleFollowed(project);
		expect(workbench.isFollowed(7)).toBe(false);

		vi.mocked(api.setFollowed).mockRejectedValueOnce('disk full');
		await workbench.toggleFollowed(project);
		expect(workbench.isFollowed(7)).toBe(false);
		expect(workbench.error).toBe('disk full');
	});

	it('finishes, resets onboarding and remembers the clone folder', async () => {
		await workbench.finishOnboarding();
		expect(workbench.onboarded).toBe(true);
		await workbench.resetOnboarding();
		expect(workbench.onboarded).toBe(false);
		await workbench.setCloneInto('D:\\src');
		expect(workbench.cloneInto).toBe('D:\\src');
	});

	it('reads nothing outside the app', async () => {
		vi.mocked(core.inTauri).mockReturnValueOnce(false);
		await workbench.load();
		expect(api.state).not.toHaveBeenCalled();
		expect(workbench.loaded).toBe(true);
	});
});
