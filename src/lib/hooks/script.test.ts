// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { flushSync, render } from '../../testing/mount';
import HookScript from './HookScript.svelte';
import { scriptLanguage } from './script';

/** FEAT-171: a hook file is coloured as the language its first line names. */
describe('a hook script', () => {
	it('is a shell script unless it says otherwise', () => {
		expect(scriptLanguage('.husky/pre-commit', 'npx lint-staged\n')).toBe('shell');
		expect(scriptLanguage('.git/hooks/pre-push', '#!/bin/sh\nexec make test')).toBe('shell');
		expect(scriptLanguage('.git/hooks/pre-push', '#!/usr/bin/env bash\nset -e')).toBe('shell');
	});

	it('follows the program its first line names', () => {
		expect(scriptLanguage('.git/hooks/commit-msg', '#!/usr/bin/env node\nconsole.log(1)')).toBe('javascript');
		expect(scriptLanguage('.git/hooks/commit-msg', '#!/usr/bin/python3\nimport sys')).toBe('python');
	});

	it('goes by the name when it has an extension', () => {
		expect(scriptLanguage('lefthook.yml', 'pre-commit:\n  commands: {}')).toBe('yaml');
	});

	it('draws numbered, coloured lines', () => {
		const view = render(HookScript, { path: '.husky/pre-commit', text: '# lint first\nnpx lint-staged\n' });
		flushSync();

		expect(view.all('.line')).toHaveLength(2);
		expect(view.all('.number').map((number) => number.textContent)).toEqual(['1', '2']);
		expect(view.get('.tok-comment').textContent).toBe('# lint first');
		view.destroy();
	});
});
