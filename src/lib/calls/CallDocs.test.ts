// SPDX-License-Identifier: MIT

/**
 * A call's *Docs* (FEAT-179): the body's fields with their rules in words,
 * and each answer with where it was read from and its example.
 */

import { afterEach, describe, expect, it } from 'vitest';
import { click, flushSync, render, type Mounted } from '../../testing/mount';
import CallDocs from './CallDocs.svelte';

let view: Mounted | null = null;

afterEach(() => {
	view?.destroy();
	view = null;
	document.body.innerHTML = '';
});

const rows = () => [...document.querySelectorAll('tr')].map((row) => [...row.querySelectorAll('td')].map((cell) => cell.textContent));

describe('Docs', () => {
	it('lists the fields with their rules, and the answers', () => {
		view = render(CallDocs, {
			rules: [
				{ path: 'email', type: 'string', required: true, format: 'email' },
				{ path: 'lines[].qty', type: 'integer', minimum: 1 },
				{ path: 'note', type: 'string' }
			],
			responses: [
				{ status: 201, says: 'Created', from: 'the return type', body: '{"id":"1"}' },
				{ status: 404, says: 'Order not found', from: 'throws OrderNotFoundException' }
			]
		});
		expect(rows()).toEqual([
			['email', 'string', 'required · email'],
			['lines[].qty', 'integer', 'at least 1'],
			['note', 'string', '—']
		]);
		const answers = [...document.querySelectorAll('.answer')].map((line) => line.textContent?.replace(/\s+/g, ' ').trim());
		expect(answers).toEqual(['201 Created the return type Example', '404 Order not found throws OrderNotFoundException']);
		expect(document.querySelector('.status')?.classList.contains('ok')).toBe(true);
	});

	it('opens an answer’s example, set out', () => {
		view = render(CallDocs, { rules: [], responses: [{ status: 400, says: 'Bad request', body: '{"error":"email"}' }] });
		expect(document.querySelector('pre')).toBeNull();
		expect(document.body.textContent).not.toContain("The body's fields");
		click(document.querySelector('.example') as HTMLElement);
		flushSync();
		expect(document.querySelector('pre')?.textContent).toBe('{\n  "error": "email"\n}');
		click(document.querySelector('.example') as HTMLElement);
		flushSync();
		expect(document.querySelector('pre')).toBeNull();
	});

	it('shows an example that is not JSON as it was written', () => {
		view = render(CallDocs, { rules: [], responses: [{ status: 500, says: 'Internal server error', body: 'upstream timed out' }] });
		expect(document.querySelector('.status')?.classList.contains('danger')).toBe(true);
		expect(document.querySelector('.from')).toBeNull();
		click(document.querySelector('.example') as HTMLElement);
		flushSync();
		expect(document.querySelector('pre')?.textContent).toBe('upstream timed out');
		expect(document.querySelector('.example')?.textContent).toBe('Hide example');
	});
});
