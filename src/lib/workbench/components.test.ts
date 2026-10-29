// SPDX-License-Identifier: MIT

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { click, flushSync, render } from '../../testing/mount';

const goto = vi.fn(async (_path: string) => {});
vi.mock('$app/navigation', () => ({ goto: (path: string) => goto(path) }));
vi.mock('$app/state', () => ({ page: { url: new URL('http://localhost/home') } }));
vi.mock('$lib/graph/store.svelte', async () => await import('../../testing/graph-store.svelte'));
vi.mock('$lib/repo.svelte', async () => await import('../../testing/repo-store.svelte'));

vi.mock('$lib/api', () => ({
	inTauri: vi.fn(() => true),
	forgeAccounts: vi.fn(async () => [{ kind: 'gitLab', host: 'gitlab.apps.ocp-nonprod-01.hodomain.local', user: 'n.hassan' }]),
	forgeDisconnect: vi.fn(async () => []),
	homeOverview: vi.fn(async () => ({
		remote: null, licence: null, version: null, versionTime: null, upstream: { upstream: 'origin/main', ahead: 2, behind: 1 },
		defaultBranch: 'main', branches: [], branchCount: 1, commitTimes: [], unreleased: null
	})),
	workingCopy: vi.fn(async () => ({ conflicted: [], staged: [{ path: 'a.txt', status: 'added' }], unstaged: [{ path: 'a.txt', status: 'modified' }, { path: 'b.txt', status: 'modified' }] })),
	setSettings: vi.fn(async () => {})
}));

vi.mock('./api', () => ({
	state: vi.fn(async () => ({ onboarded: { version: 1, at: 'x' }, followed: [], cloneInto: null })),
	setOnboarded: vi.fn(async (done: boolean) => ({ onboarded: done ? { version: 1, at: 'y' } : null, followed: [], cloneInto: null })),
	gitlabTokenInfo: vi.fn(async () => ({ name: 'Nour Hassan', login: 'n.hassan', scopes: ['api', 'read_repository', 'write_repository'], scopesKnown: true, expiresAt: '2000-01-01' })),
	gitlabConnect: vi.fn(),
	gitlabProjects: vi.fn(async () => ({ projects: [], page: 1, more: false })),
	gitlabProjectsCached: vi.fn(async () => null),
	setFollowed: vi.fn()
}));

import * as core from '$lib/api';
import { control as repoControl } from '../../testing/repo-store.svelte';
import GitLabSection from './GitLabSection.svelte';
import LoadedBand from './LoadedBand.svelte';
import { onboard } from './onboard.svelte';
import RepoSlot from './RepoSlot.svelte';
import RunSetupAgain from './RunSetupAgain.svelte';
import SectionsOrder from './SectionsOrder.svelte';
import { workbench } from './state.svelte';
import TokenPage from './TokenPage.svelte';
import WideHomeSection from './WideHomeSection.svelte';

const info = {
	path: 'C:\\Projects\\crx-transfers',
	name: 'crx-transfers',
	bare: false,
	lastFetched: null,
	head: { branch: 'main', detached: false, id: 'a'.repeat(40), short: 'aaaaaaa' }
};

async function settle() {
	for (let i = 0; i < 8; i += 1) await new Promise((resolve) => setTimeout(resolve, 0));
	flushSync();
}

beforeEach(() => {
	vi.clearAllMocks();
	repoControl.reset();
	workbench.clear();
});

afterEach(() => vi.unstubAllGlobals());

describe('Settings › GitLab (FEAT-181)', () => {
	it('says who is signed in, the scopes, and an expired token as a warning', async () => {
		const view = render(GitLabSection, {});
		await settle();
		expect(view.text()).toContain('Nour Hassan @n.hassan');
		expect(view.text()).toContain('api, read_repository, write_repository');
		expect(view.get('dd.warn').textContent).toBe('expired 1 Jan 2000');

		click(view.all('button').find((b) => b.textContent?.trim() === 'Replace token')!);
		expect(view.find('#gitlab-token')).not.toBeNull();
		click(view.all('button').find((b) => b.textContent?.trim() === 'Cancel')!);
		expect(view.find('#gitlab-token')).toBeNull();

		click(view.all('button').find((b) => b.textContent?.trim() === 'Disconnect')!);
		await settle();
		expect(core.forgeDisconnect).toHaveBeenCalledWith('gitlab.apps.ocp-nonprod-01.hodomain.local', 'n.hassan');
		expect(view.text()).toContain('Not connected');
		view.destroy();
	});

	it('offers Connect when nothing is connected', async () => {
		vi.mocked(core.forgeAccounts).mockResolvedValueOnce([]);
		const view = render(GitLabSection, {});
		await settle();
		click(view.all('button').find((b) => b.textContent?.trim() === 'Connect')!);
		expect(view.find('#gitlab-token')).not.toBeNull();
		expect(view.find('a[target="_blank"]')?.getAttribute('href')).toContain('/-/profile/personal_access_tokens');
		view.destroy();
	});
});

describe('the token page link (FEAT-181)', () => {
	it('copies the link and the scopes', async () => {
		const writeText = vi.fn(async () => {});
		vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText } });
		const view = render(TokenPage, { guide: true });
		click(view.all('button').find((b) => b.textContent === 'Copy link')!);
		await settle();
		expect(writeText).toHaveBeenCalledWith(expect.stringContaining('name=Workbench&scopes=api,read_repository,write_repository'));
		expect(view.text()).toContain('Copied');
		click(view.all('button').find((b) => b.textContent === 'Copy scopes')!);
		await settle();
		expect(writeText).toHaveBeenLastCalledWith('api read_repository write_repository');
		expect(view.text()).toContain('Add new token');
		view.destroy();
	});
});

describe("Home's sections (FEAT-183)", () => {
	it('moves a section and switches one off', () => {
		const onchange = vi.fn();
		const view = render(SectionsOrder, { layout: { hidden: [], order: [] }, onchange });
		click(view.get('button[aria-label="Move Calls up"]'));
		expect(onchange).toHaveBeenLastCalledWith({ hidden: [], order: ['needs', 'requests', 'pipelines', 'calls', 'issues', 'releases'] });
		click(view.get('button[aria-label="Show Releases on Home"]'));
		expect(onchange).toHaveBeenLastCalledWith({ hidden: ['releases'], order: [] });
		expect((view.get('button[aria-label="Move Needs you up"]') as HTMLButtonElement).disabled).toBe(true);
		view.destroy();

		const off = render(SectionsOrder, { layout: { hidden: ['releases'], order: [] }, onchange });
		click(off.get('button[aria-label="Show Releases on Home"]'));
		expect(onchange).toHaveBeenLastCalledWith({ hidden: [], order: [] });
		off.destroy();
	});

	it('Settings › Home asks for a connection before choosing repositories', async () => {
		vi.mocked(core.forgeAccounts).mockResolvedValueOnce([]);
		const view = render(WideHomeSection, {});
		await settle();
		expect(view.text()).toContain('Connect CIB GitLab in Settings › GitLab');
		view.destroy();
	});
});

describe('the Loaded band and the title bar slot (FEAT-183, FEAT-184)', () => {
	it('shows the working copy, the branch against upstream, and Push', async () => {
		repoControl.setInfo(info);
		const view = render(LoadedBand, { project: 'payments/crx-transfers', now: Date.now() });
		await settle();
		expect(view.text()).toContain('payments/crx-transfers');
		expect(view.text()).toMatch(/main\s*→ origin\/main/);
		expect(view.text()).toMatch(/2 to push, 1 to pull ·\s*never fetched/);
		click(view.all('button').find((b) => b.textContent?.trim() === 'Open graph')!);
		expect(goto).toHaveBeenCalledWith('/');
		view.destroy();
	});

	it('shows ahead and behind on the loaded repository', async () => {
		repoControl.setInfo(info);
		const view = render(RepoSlot, {});
		await settle();
		expect(view.get('.counts').textContent).toBe('↑2 ↓1');
		view.destroy();
	});

	it('shows the load running', () => {
		repoControl.setBusy(true);
		const view = render(RepoSlot, {});
		expect(view.find('button.empty')).toBeNull();
		view.destroy();
	});
});

describe('Run setup again (FEAT-180)', () => {
	it('clears onboarded and opens the setup', async () => {
		const view = render(RunSetupAgain, {});
		click(view.get('button'));
		await settle();
		expect(workbench.onboarded).toBe(false);
		expect(onboard.showing).toBe(true);
		expect(goto).toHaveBeenCalledWith('/welcome');
		onboard.done();
		view.destroy();
	});
});
