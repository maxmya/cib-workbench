// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { asFailure, expiry, failureSentence, initials, keychainName, platformOf } from './gitlab';

describe('what the GitLab screens say (FEAT-181)', () => {
	it('says which of the four failures happened', () => {
		expect(failureSentence({ kind: 'unauthorized', detail: 'x' })).toMatch(/refused this token/);
		expect(failureSentence({ kind: 'unreachable', detail: 'x' })).toMatch(/VPN/);
		expect(failureSentence({ kind: 'missingScope', scopes: ['write_repository'] })).toBe(
			'This token is missing write_repository. Create one with every scope listed.'
		);
		expect(failureSentence({ kind: 'missingScope', scopes: ['api', 'read_repository', 'write_repository'] })).toMatch(
			/missing api, read_repository and write_repository\./
		);
		expect(failureSentence({ kind: 'keychain', detail: 'locked' })).toMatch(/Nothing was saved/);
	});

	it('reads what a rejected command threw, object or text', () => {
		expect(asFailure({ kind: 'keychain', detail: 'no' })).toEqual({ kind: 'keychain', detail: 'no' });
		expect(asFailure('boom')).toEqual({ kind: 'other', detail: 'boom' });
		expect(asFailure(new Error('gone'))).toEqual({ kind: 'other', detail: 'gone' });
	});

	it('names the secure store each platform has', () => {
		expect(keychainName(platformOf({ platform: 'Win32' }))).toBe('Windows Credential Manager');
		expect(keychainName(platformOf({ platform: 'MacIntel' }))).toBe('Keychain');
		expect(keychainName(platformOf({ platform: 'Linux x86_64' }))).toBe('the Secret Service');
	});

	it('warns in the last fourteen days and after', () => {
		const now = Date.parse('2026-10-28T12:00:00Z');
		expect(expiry('2027-01-26', now)).toEqual({ label: 'expires 26 Jan 2027', warn: false });
		expect(expiry('2026-11-05', now).warn).toBe(true);
		expect(expiry('2026-10-01', now)).toEqual({ label: 'expired 1 Oct 2026', warn: true });
		expect(expiry(null, now)).toEqual({ label: 'does not expire', warn: false });
	});

	it('draws initials from a name or a login', () => {
		expect(initials('Nour Hassan')).toBe('NH');
		expect(initials('n.hassan')).toBe('NH');
		expect(initials('')).toBe('?');
	});
});
