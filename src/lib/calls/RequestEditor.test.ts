// SPDX-License-Identifier: MIT

/**
 * The request half of Calls (FEAT-160): the method, the URL and *Send*,
 * and each tab — Params, Headers, Body, Auth, Keep, and Docs when a scan
 * read the call's rules and answers (FEAT-179). Each edit is handed up as
 * a change to the call; these apply it to see what would be written.
 */

import { afterEach, describe, expect, it, vi } from 'vitest';
import { click, flushSync, press, render, type Mounted } from '../../testing/mount';
import RequestEditor from './RequestEditor.svelte';
import type { Call } from './types';

let view: Mounted | null = null;

afterEach(() => {
	view?.destroy();
	view = null;
	document.body.innerHTML = '';
});

function show(call: Call, more: Record<string, unknown> = {}) {
	// Applied at once, as the store does: a handler may read its event.
	const changes: Call[] = [];
	const props = {
		call,
		onchange: (change: (call: Call) => Call) => changes.push(change(call)),
		find: () => null,
		names: [],
		sending: false,
		refusal: null,
		onsend: vi.fn(),
		oncancel: vi.fn(),
		...more
	};
	view = render(RequestEditor, props);
	/** What the last edit would make of the call. */
	const last = () => changes.at(-1)!;
	return { props, changes, last };
}

const rest = (over: Partial<Call> = {}): Call => ({ id: 'c1', name: 'Get order', kind: 'rest', method: 'GET', url: '{{baseUrl}}/orders/1', ...over });
const button = (text: string) =>
	[...document.querySelectorAll('button')].find((found) => found.textContent?.replace(/\s+/g, ' ').trim().startsWith(text)) as HTMLButtonElement;
const tabs = () => view!.all('[role="tab"]').map((tab) => tab.textContent?.replace(/\s+/g, ' ').trim());
const tab = (name: string) => view!.all('[role="tab"]').find((found) => found.textContent?.startsWith(name)) as HTMLElement;
const item = (label: string) =>
	[...document.querySelectorAll('[role="menuitem"]')].find((entry) => entry.textContent?.includes(label)) as HTMLElement;

describe('the request', () => {
	it('opens on Params for a call with no body, counts what each tab holds, and has no Docs tab', () => {
		show(rest({ url: '{{baseUrl}}/orders?state=open', headers: [['Accept', 'json']] }));
		expect(tabs()).toEqual(['Params1', 'Headers1', 'Body', 'Auth', 'Keep']);
		expect(tab('Params').getAttribute('aria-selected')).toBe('true');
	});

	it('opens a call with a body on Body, and its Docs say what the code holds it to', () => {
		show(
			rest({
				method: 'POST',
				body: { type: 'json', text: '{}' },
				rules: [{ path: 'email', type: 'string', required: true }],
				responses: [{ status: 201, says: 'Created' }]
			})
		);
		expect(tab('Body').getAttribute('aria-selected')).toBe('true');
		expect(view!.text()).toContain('JSON ▾');
		click(tab('Docs'));
		expect(view!.text()).toContain('The body\'s fields');
		expect(view!.text()).toContain('201');
	});

	it('changes the method from its menu, and sends on Enter but not Shift+Enter', () => {
		const { last, props } = show(rest());
		click(button('GET'));
		click(item('POST'));
		expect(last().method).toBe('POST');

		const url = view!.get('[aria-label="URL"]');
		press(url, 'Enter', { shiftKey: true });
		expect(props.onsend).not.toHaveBeenCalled();
		press(url, 'Enter');
		expect(props.onsend).toHaveBeenCalledTimes(1);
	});

	it('says why it was not sent, offers cURL, and cancels while sending', () => {
		const oncurl = vi.fn();
		const { props } = show(rest(), { sending: true, refusal: 'Fill in {{token}} first', oncurl });
		expect(view!.get('[role="alert"]').textContent).toBe('Fill in {{token}} first');
		click(button('cURL'));
		expect(oncurl).toHaveBeenCalled();
		click(button('Cancel'));
		expect(props.oncancel).toHaveBeenCalled();
		expect(button('Send')).toBeUndefined();
	});

	it('cannot change a SOAP call’s method', () => {
		show(rest({ kind: 'soap', method: undefined }));
		expect((view!.get('.method') as HTMLButtonElement).disabled).toBe(true);
	});
});

describe('the body', () => {
	it('says a call sends none, and gives it one from the picker', () => {
		const { last } = show(rest());
		click(tab('Body'));
		expect(view!.text()).toContain('This call sends no body.');
		click(button('None ▾'));
		click(item('Form'));
		expect(last().body).toEqual({ type: 'form', fields: [] });
		click(button('None ▾'));
		click(item('XML'));
		expect(last().body).toEqual({ type: 'xml', text: '' });
	});

	it('takes a raw body’s Content-Type, and none when it is cleared', () => {
		const { last } = show(rest({ body: { type: 'raw', text: 'hi', contentType: 'text/plain' } }));
		const type = view!.get('input[placeholder="text/plain"]') as HTMLInputElement;
		type.value = 'text/csv';
		type.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		expect(last().body).toEqual({ type: 'raw', text: 'hi', contentType: 'text/csv' });
		type.value = '';
		type.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		expect(last().body?.contentType).toBeUndefined();
	});

	it('lists a form’s fields, a switched-off one too', () => {
		show(
			rest({
				body: {
					type: 'multipart',
					fields: [
						{ name: 'file', value: 'a.png', file: true },
						{ name: 'note', value: 'x', on: false }
					]
				}
			})
		);
		expect(view!.get('[aria-label="fields"]')).toBeTruthy();
		expect((view!.all('input[placeholder="path to a file"]')[0] as HTMLInputElement).value).toBe('a.png');
	});

	it('says Content-Type follows the body, on Headers', () => {
		show(rest({ body: { type: 'json', text: '{}' } }));
		click(tab('Headers'));
		expect(view!.text()).toContain('Content-Type follows the body');
	});
});

describe('authorization', () => {
	it('says nothing is added without it, and chooses a kind', () => {
		const { last } = show(rest());
		click(tab('Auth'));
		expect(view!.text()).toContain('No Authorization is added.');
		click(button('Bearer token'));
		expect(last().auth).toEqual({ type: 'bearer' });
	});

	it('shows a bearer token’s field', () => {
		show(rest({ auth: { type: 'bearer', token: '{{token}}' } }));
		click(tab('Auth'));
		expect(view!.get('[aria-label="Token"]')).toBeTruthy();
		expect(view!.text()).toContain('Write a secret as a variable');
	});

	it('shows Basic’s user and password', () => {
		show(rest({ auth: { type: 'basic', user: 'me' } }));
		click(tab('Auth'));
		expect(view!.get('[aria-label="User"]')).toBeTruthy();
		expect(view!.get('[aria-label="Password"]')).toBeTruthy();
	});

	it('sends an API key in a header or the query', () => {
		const { last } = show(rest({ auth: { type: 'apiKey', key: 'X-Api-Key', value: '{{apiKey}}', in: 'query' } }));
		click(tab('Auth'));
		expect(button('the query').getAttribute('aria-checked')).toBe('true');
		click(button('a header'));
		expect(last().auth).toEqual({ type: 'apiKey', key: 'X-Api-Key', value: '{{apiKey}}', in: 'header' });
	});
});
