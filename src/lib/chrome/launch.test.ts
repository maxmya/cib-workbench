// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { render } from '../../testing/mount';
import { NOTICE, PRODUCT_NAME, SPLASH_LINE } from '$lib/brand';
import Launch from './Launch.svelte';

/**
 * Workbench's launch screen: what is opening, and the internal-use notice in
 * its approved wording.
 */
describe('the launch screen', () => {
	it('names the repository it is opening', () => {
		const view = render(Launch, { repository: 'crx-transfers' });

		expect(view.get('.launch').getAttribute('aria-label')).toBe('Opening crx-transfers');
		expect(view.get('.opening').textContent).toContain('crx-transfers');
		expect(view.get('.mark .wordmark').getAttribute('aria-label')).toBe(PRODUCT_NAME);
		expect(view.text()).toContain(SPLASH_LINE);
		view.destroy();
	});

	it('says only that it is opening when there is no repository to open', () => {
		const view = render(Launch, { repository: null });

		expect(view.get('.launch').getAttribute('aria-label')).toBe(`Opening ${PRODUCT_NAME}`);
		expect(view.get('.opening').textContent?.trim()).toBe('Opening…');
		view.destroy();
	});

	it('carries the notice, the trademark line and the licence', () => {
		const view = render(Launch, { repository: null });
		const text = view.text();

		expect(text).toContain(NOTICE.badge);
		expect(text).toContain(NOTICE.lead);
		expect(text).toContain(NOTICE.trademark);
		expect(text).toContain(NOTICE.licence);
		expect(view.get('.logo').getAttribute('alt')).toMatch(/^CIB/);
		view.destroy();
	});
});
