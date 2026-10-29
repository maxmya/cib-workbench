// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { back, canGoOn, FIRST, marks, next, type Rules, type Stage } from './onboarding';

const fresh: Rules = { connected: false, checked: false };

function walk(rules: Rules): Stage[] {
	const seen: Stage[] = [FIRST];
	let stage: Stage | null = FIRST;
	while ((stage = next(stage, rules)) && seen.length < 10) {
		if (JSON.stringify(stage) === JSON.stringify(seen[seen.length - 1])) break;
		seen.push(stage);
	}
	return seen;
}

describe('onboarding (FEAT-180)', () => {
	it('moves through all six pages with a checked token, then finishes', () => {
		expect(walk({ connected: false, checked: true })).toEqual([
			{ step: 1, page: 0 },
			{ step: 1, page: 1 },
			{ step: 1, page: 2 },
			{ step: 2, page: 0 },
			{ step: 3, page: 0 },
			{ step: 4, page: 0 }
		]);
		expect(next({ step: 4, page: 0 }, fresh)).toBeNull();
	});

	it('holds Next on the token step until the check passes', () => {
		expect(canGoOn({ step: 3, page: 0 }, fresh)).toBe(false);
		expect(next({ step: 3, page: 0 }, fresh)).toEqual({ step: 3, page: 0 });
		expect(canGoOn({ step: 3, page: 0 }, { connected: false, checked: true })).toBe(true);
		expect(canGoOn({ step: 2, page: 0 }, fresh)).toBe(true);
	});

	it('goes back page by page, and from GitLab to the last Welcome page', () => {
		expect(back({ step: 1, page: 0 }, fresh)).toEqual({ step: 1, page: 0 });
		expect(back({ step: 1, page: 2 }, fresh)).toEqual({ step: 1, page: 1 });
		expect(back({ step: 2, page: 0 }, fresh)).toEqual({ step: 1, page: 2 });
		expect(back({ step: 4, page: 0 }, fresh)).toEqual({ step: 3, page: 0 });
	});

	it('passes over GitLab and Token when an account is connected, both ways', () => {
		const connected: Rules = { connected: true, checked: false };
		expect(next({ step: 1, page: 2 }, connected)).toEqual({ step: 4, page: 0 });
		expect(back({ step: 4, page: 0 }, connected)).toEqual({ step: 1, page: 2 });
		expect(marks({ step: 4, page: 0 }, connected)).toEqual(['done', 'done', 'done', 'current']);
		expect(marks({ step: 1, page: 0 }, connected)).toEqual(['current', 'done', 'done', 'todo']);
	});

	it('marks steps done, current and still to do', () => {
		expect(marks({ step: 3, page: 0 }, fresh)).toEqual(['done', 'done', 'current', 'todo']);
	});
});
