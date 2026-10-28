// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import type { HookStep, Hooks } from '../types';
import { makeBlock, type Definition } from './builder-types';
import { allSteps, arrange, counts, ordinal } from './order';
import { lefthookSnippet, preCommitSnippet, present, TEMPLATES, wayFor } from './templates';

const step = (hook: string, order: number, name: string, category: HookStep['category']): HookStep => ({
	hook,
	order,
	name,
	command: name,
	category,
	file: `.husky/${hook}`,
	line: order,
	together: false
});

const info = (steps: HookStep[]): Hooks => ({ dir: '.husky/_', manager: 'husky', config: null, hooks: [], enabled: true, steps });

/** FEAT-176: what runs, in the order git runs it. */
describe('the order hooks run in', () => {
	const steps = allSteps(
		info([
			step('pre-push', 1, 'npm test', 'tests'),
			step('commit-msg', 1, 'commitlint', 'message'),
			step('pre-commit', 1, 'lint-staged', 'format'),
			step('pre-commit', 2, 'tsc', 'build'),
			step('post-merge', 1, 'npm ci', 'dependencies')
		]),
		null,
		false
	);

	it('lays the moments out as a day’s work meets them, and each moment’s hooks in git’s order', () => {
		const laid = arrange(steps);
		expect(laid.map((moment) => moment.moment)).toEqual(['commit', 'push', 'merge']);
		expect(laid[0].hooks.map((hook) => hook.hook)).toEqual(['pre-commit', 'commit-msg']);
	});

	it('numbers a moment’s steps straight through its hooks', () => {
		const commit = arrange(steps)[0];
		expect(commit.hooks.flatMap((hook) => hook.steps.map((each) => [each.name, each.place]))).toEqual([
			['lint-staged', 1],
			['tsc', 2],
			['commitlint', 3]
		]);
	});

	it('keeps a step’s place when the list is filtered to one kind', () => {
		const laid = arrange(steps, 'message');
		expect(laid).toHaveLength(1);
		expect(laid[0].hooks[0].steps[0].place).toBe(3);
	});

	it('counts each kind for the filter', () => {
		const tally = counts(steps);
		expect(tally.all).toBe(5);
		expect(tally.format).toBe(1);
		expect(tally.secrets).toBe(0);
	});

	it('takes Spagitty’s blocks as steps when it writes the hooks, and marks the ones switched off', () => {
		const definition: Definition = { version: 1, shared: true, activeStages: ['pre-commit'], blocks: [makeBlock('secrets'), makeBlock('tests', 'pre-push')] };
		const blocks = allSteps(null, definition, true);
		expect(blocks.map((each) => [each.hook, each.category, each.off])).toEqual([
			['pre-commit', 'secrets', false],
			['pre-push', 'tests', true]
		]);
		expect(blocks[0].file).toBe('.spagitty/hooks.toml');
		// A step that is off has no place, and does not take one from the steps after it.
		expect(arrange(blocks)[1].hooks[0].steps[0].place).toBe(0);
	});

	it('says a place as English does', () => {
		expect([1, 2, 3, 4, 11, 12, 13, 21, 22, 103].map(ordinal)).toEqual(['1st', '2nd', '3rd', '4th', '11th', '12th', '13th', '21st', '22nd', '103rd']);
	});
});

describe('hooks to start from', () => {
	const commitlint = TEMPLATES.find((template) => template.id === 'commitlint')!;
	const tests = TEMPLATES.find((template) => template.id === 'npm-test')!;

	it('are each in one kind and one hook, explained', () => {
		for (const template of TEMPLATES) {
			expect(template.says.length, template.id).toBeGreaterThan(20);
			expect(template.why.length, template.id).toBeGreaterThan(20);
			expect(template.command.trim(), template.id).not.toBe('');
		}
		expect(new Set(TEMPLATES.map((template) => template.id)).size).toBe(TEMPLATES.length);
	});

	it('are added the way the repository’s manager keeps its hooks', () => {
		expect(wayFor('husky', tests)).toBe('husky');
		expect(wayFor('git', tests)).toBe('blocks');
		expect(wayFor('lefthook', tests)).toBe('lefthook');
		expect(wayFor('preCommit', tests)).toBe('preCommit');
		// Spagitty's blocks run in three hooks; post-merge is not one.
		expect(wayFor('git', TEMPLATES.find((template) => template.hook === 'post-merge')!)).toBe('none');
	});

	it('give lefthook its own placeholder for the message file, quoted for YAML', () => {
		expect(lefthookSnippet(commitlint)).toBe('commit-msg:\n  commands:\n    commitlint:\n      run: "npx --no -- commitlint --edit {1}"\n');
	});

	it('give pre-commit a local hook the shell runs, with the message file passed for commit-msg', () => {
		const snippet = preCommitSnippet(commitlint);
		expect(snippet).toContain('      - id: commitlint');
		expect(snippet).toContain('        entry: sh -c "npx --no -- commitlint --edit \\"$1\\"" --');
		expect(snippet).toContain('        pass_filenames: true');
		expect(snippet).toContain('        stages: [commit-msg]');
		expect(preCommitSnippet(tests)).toContain('pass_filenames: false');
	});

	it('know when the repository already runs one', () => {
		expect(present(tests, ['npx lint-staged', 'npm  test'])).toBe(true);
		expect(present(tests, ['npm run test:e2e'])).toBe(false);
	});
});
