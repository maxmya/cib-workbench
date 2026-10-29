// SPDX-License-Identifier: MIT

/**
 * What the GitLab screens say (FEAT-181): pure, so the sentences are tested
 * without a window.
 */

import type { ConnectFailure } from './types';

/** What each scope is for, as the token step and Settings › GitLab list them. */
export const SCOPE_USES: Record<string, string> = {
	api: 'merge requests, pipelines, issues',
	read_repository: 'clone and fetch',
	write_repository: 'push'
};

/** Whatever a rejected command threw, as a failure with a kind. */
export function asFailure(error: unknown): ConnectFailure {
	if (error && typeof error === 'object' && 'kind' in error) return error as ConnectFailure;
	return { kind: 'other', detail: error instanceof Error ? error.message : String(error) };
}

/** The one sentence a failed check shows. */
export function failureSentence(failure: ConnectFailure): string {
	switch (failure.kind) {
		case 'unauthorized':
			return "CIB GitLab refused this token. Check you copied all of it, and that it hasn't expired.";
		case 'unreachable':
			return 'Could not reach CIB GitLab. You may need the VPN, or the proxy.';
		case 'missingScope':
			return `This token is missing ${list(failure.scopes)}. Create one with every scope listed.`;
		case 'keychain':
			return "Your system's keychain refused the token. Nothing was saved.";
		case 'other':
			return failure.detail;
	}
}

function list(items: string[]): string {
	if (items.length <= 1) return items[0] ?? 'a scope';
	return `${items.slice(0, -1).join(', ')} and ${items[items.length - 1]}`;
}

export type Platform = 'windows' | 'mac' | 'linux';

export function platformOf(source: { userAgent?: string; platform?: string } | undefined): Platform {
	const text = `${source?.platform ?? ''} ${source?.userAgent ?? ''}`;
	if (/Win/i.test(text)) return 'windows';
	if (/Mac|iPhone|iPad/.test(text)) return 'mac';
	return 'linux';
}

/** Where the token is kept, by the name the platform gives it. */
export function keychainName(platform: Platform): string {
	return { windows: 'Windows Credential Manager', mac: 'Keychain', linux: 'the Secret Service' }[platform];
}

export interface Expiry {
	/** `expires 26 Jan 2027`, `expired 2 Oct 2026` or `does not expire`. */
	label: string;
	/** Inside its last 14 days, or already past. */
	warn: boolean;
}

const DAY = 86_400_000;

export function expiry(expiresAt: string | null, now: number): Expiry {
	if (!expiresAt) return { label: 'does not expire', warn: false };
	const at = Date.parse(`${expiresAt}T00:00:00Z`);
	if (Number.isNaN(at)) return { label: `expires ${expiresAt}`, warn: false };
	const day = new Date(at).toLocaleDateString('en-GB', {
		day: 'numeric',
		month: 'short',
		year: 'numeric',
		timeZone: 'UTC'
	});
	if (at <= now) return { label: `expired ${day}`, warn: true };
	return { label: `expires ${day}`, warn: at - now <= 14 * DAY };
}

/** Initials for an avatar with no picture. */
export function initials(name: string): string {
	const words = name.trim().split(/[\s._-]+/).filter(Boolean);
	return ((words[0]?.[0] ?? '') + (words[1]?.[0] ?? '')).toUpperCase() || '?';
}
