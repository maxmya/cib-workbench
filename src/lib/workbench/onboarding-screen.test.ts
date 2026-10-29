// SPDX-License-Identifier: MIT

import { beforeEach, describe, expect, it, vi } from 'vitest';
import { click, flushSync, render } from '../../testing/mount';

const goto = vi.fn(async (_path: string) => {});
vi.mock('$app/navigation', () => ({ goto: (path: string) => goto(path) }));

vi.mock('$lib/api', () => ({
	inTauri: vi.fn(() => true),
	forgeAccounts: vi.fn(async () => []),
	setSettings: vi.fn(async () => {}),
	settings: vi.fn(async () => ({ home: { hidden: [], order: [] }, notifyEveryMinutes: 2 })),
	identity: vi.fn(async () => ({ repository: false })),
	signing: vi.fn(async () => null),
	licenses: vi.fn(async () => null),
	about: vi.fn(async () => null)
}));

vi.mock('./api', () => ({
	state: vi.fn(async () => ({ onboarded: null, followed: [], cloneInto: null })),
	setOnboarded: vi.fn(async (done: boolean) => ({ onboarded: done ? { version: 1, at: 'now' } : null, followed: [], cloneInto: null })),
	setFollowed: vi.fn(async (followed: unknown[]) => ({ onboarded: null, followed, cloneInto: null })),
	setCloneInto: vi.fn(),
	gitlabConnect: vi.fn(),
	gitlabTokenInfo: vi.fn(),
	gitlabProjects: vi.fn(async () => ({
		projects: [{ id: 7, path: 'payments/crx-transfers', name: 'crx-transfers', description: 'Transfers', lastActivity: 0, defaultBranch: 'main', httpUrl: 'https://g/p.git', webUrl: '' }],
		page: 1,
		more: false
	})),
	gitlabProjectsCached: vi.fn(async () => null)
}));

import * as core from '$lib/api';
import * as api from './api';
import Onboarding from './Onboarding.svelte';
import { onboard } from './onboard.svelte';
import { workbench } from './state.svelte';

function button(view: ReturnType<typeof render>, label: RegExp): HTMLButtonElement {
	const found = view.all('button').find((b) => label.test(b.textContent?.trim() ?? ''));
	if (!found) throw new Error(`no button ${label}`);
	return found as HTMLButtonElement;
}

async function settle() {
	for (let i = 0; i < 5; i += 1) await Promise.resolve();
	flushSync();
}

beforeEach(() => {
	vi.clearAllMocks();
	workbench.clear();
	workbench.present({ login: null });
});

describe('onboarding (FEAT-180)', () => {
	it('reaches the empty Home with Set up later, and leaves onboarded unset', async () => {
		const waiting = onboard.run(() => {});
		const view = render(Onboarding, {});
		expect(() => button(view, /^Back$/)).toThrow();
		click(button(view, /^Next$/));
		expect(button(view, /^Back$/)).toBeTruthy();
		for (let i = 0; i < 2; i += 1) click(button(view, /^Next$/));
		expect(view.text()).toContain('Connect to CIB GitLab');

		click(button(view, /^Set up later$/));
		await waiting;

		expect(goto).toHaveBeenCalledWith('/home');
		expect(api.setOnboarded).not.toHaveBeenCalled();
		expect(onboard.showing).toBe(false);
		view.destroy();
	});

	it('holds Next on the token step until CIB GitLab accepts the token, then clears the field', async () => {
		vi.mocked(api.gitlabConnect).mockResolvedValueOnce({
			name: 'Nour Hassan', login: 'n.hassan', scopes: ['api', 'read_repository', 'write_repository'], scopesKnown: true, expiresAt: null
		});
		const view = render(Onboarding, {});
		for (let i = 0; i < 3; i += 1) click(button(view, /^Next$/));
		click(button(view, /^I have my token$/));
		expect(button(view, /^Next$/).disabled).toBe(true);

		const field = view.get('#gitlab-token') as HTMLInputElement;
		field.value = 'glpat-secret';
		field.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		click(button(view, /^Check$/));
		await settle();

		expect(api.gitlabConnect).toHaveBeenCalledWith('glpat-secret');
		expect((view.get('#gitlab-token') as HTMLInputElement).value).toBe('');
		expect(view.text()).toContain('Nour Hassan');
		expect(button(view, /^Next$/).disabled).toBe(false);
		view.destroy();
	});

	it('names a refused token and keeps Next held', async () => {
		vi.mocked(api.gitlabConnect).mockRejectedValueOnce({ kind: 'unauthorized', detail: '401' });
		const view = render(Onboarding, {});
		for (let i = 0; i < 4; i += 1) click(button(view, /^(Next|I have my token)$/));
		const field = view.get('#gitlab-token') as HTMLInputElement;
		field.value = 'glpat-wrong';
		field.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		click(button(view, /^Check$/));
		await settle();

		expect(view.text()).toContain('CIB GitLab refused this token');
		expect(button(view, /^Next$/).disabled).toBe(true);
		view.destroy();
	});

	it('passes over GitLab and Token when an account is connected, and writes Home only when opened', async () => {
		workbench.present({ login: 'n.hassan' });
		const waiting = onboard.run(() => {});
		const view = render(Onboarding, {});
		expect(view.text()).toContain('Connected as @n.hassan');
		for (let i = 0; i < 3; i += 1) click(button(view, /^Next$/));
		expect(view.text()).toContain('Set up your Home');
		await settle();

		click(view.all('button[aria-pressed]').find((b) => b.textContent?.includes('payments/crx-transfers'))!);
		click(view.get('button[aria-label="Move Needs you down"]'));
		expect(api.setFollowed).not.toHaveBeenCalled();
		expect(core.setSettings).not.toHaveBeenCalled();

		click(button(view, /^Open Workbench$/));
		await waiting;
		await settle();

		expect(vi.mocked(api.setFollowed).mock.calls[0][0]).toEqual([
			expect.objectContaining({ id: 7, path: 'payments/crx-transfers' })
		]);
		expect(vi.mocked(core.setSettings).mock.calls[0][0]).toMatchObject({ home: { order: ['requests', 'needs', 'pipelines', 'issues', 'calls', 'releases'] } });
		expect(api.setOnboarded).toHaveBeenCalledWith(true);
		expect(goto).toHaveBeenCalledWith('/home');
		view.destroy();
	});
});
