// SPDX-License-Identifier: MIT

import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('$lib/api', () => ({
	inTauri: vi.fn(() => true),
	prepareUpdate: vi.fn(),
	restartToUpdate: vi.fn()
}));

vi.mock('$lib/settings/store.svelte', () => ({
	settings: { checkForUpdate: vi.fn(() => Promise.resolve()) }
}));

import * as api from '$lib/api';
import { settings } from '$lib/settings/store.svelte';
import { resetUpdates, updates } from './store.svelte';

const prepare = vi.mocked(api.prepareUpdate);
const restart = vi.mocked(api.restartToUpdate);
const inTauri = vi.mocked(api.inTauri);

beforeEach(() => {
	vi.clearAllMocks();
	inTauri.mockReturnValue(true);
	resetUpdates();
});

describe('updates', () => {
	it('downloads a newer release and waits for a restart', async () => {
		prepare.mockImplementationOnce(async (progress) => {
			progress(512, 1024);
			expect(updates.phase).toBe('downloading');
			expect(updates.percent).toBe(50);
			return { state: 'ready', version: '1.4.0', current: '1.3.1' };
		});

		await updates.check();

		expect(updates.phase).toBe('ready');
		expect(updates.version).toBe('1.4.0');
		expect(restart).not.toHaveBeenCalled();
	});

	it('installs only when asked', async () => {
		prepare.mockResolvedValueOnce({ state: 'ready', version: '1.4.0', current: '1.3.1' });
		restart.mockResolvedValueOnce(undefined);
		await updates.check();

		await updates.restart();

		expect(restart).toHaveBeenCalledOnce();
		expect(updates.phase).toBe('restarting');
	});

	it('does not download twice once an update is waiting', async () => {
		prepare.mockResolvedValueOnce({ state: 'ready', version: '1.4.0', current: '1.3.1' });
		await updates.check();
		await updates.check();

		expect(prepare).toHaveBeenCalledOnce();
	});

	it('says when it is up to date', async () => {
		prepare.mockResolvedValueOnce({ state: 'current', version: '1.3.1' });
		await updates.check();

		expect(updates.phase).toBe('current');
		expect(settings.checkForUpdate).not.toHaveBeenCalled();
	});

	it('falls back to the notice in a build that cannot install updates', async () => {
		prepare.mockResolvedValueOnce({ state: 'unsupported', reason: 'A development build is not updated in place.' });
		await updates.check();

		expect(updates.phase).toBe('unsupported');
		expect(updates.reason).toContain('development build');
		expect(settings.checkForUpdate).toHaveBeenCalledOnce();
	});

	it('keeps a failed check on screen and can try again', async () => {
		prepare.mockRejectedValueOnce('github.com: could not connect');
		await updates.check();

		expect(updates.phase).toBe('idle');
		expect(updates.error).toContain('could not connect');

		prepare.mockResolvedValueOnce({ state: 'current', version: '1.3.1' });
		await updates.check();
		expect(updates.error).toBeNull();
	});

	it('asks nothing outside the app', async () => {
		inTauri.mockReturnValue(false);
		await updates.check();

		expect(prepare).not.toHaveBeenCalled();
	});

	it('does nothing on restart when nothing is downloaded', async () => {
		await updates.restart();

		expect(restart).not.toHaveBeenCalled();
	});
});
