// SPDX-License-Identifier: MIT

/**
 * The repository's hooks as the order they run in (FEAT-176).
 *
 * The backend reads each hook into its steps (`hooks::steps`); this lays
 * them out the way a person meets them. Git runs hooks at moments — a
 * commit, a push, a merge — and at each moment in an order of its own:
 * `pre-commit`, then `prepare-commit-msg`, then `commit-msg`, then
 * `post-commit`. Within a moment the steps are numbered straight through,
 * so *3* is the third thing that runs when you commit, whichever hook it
 * is in. Hooks Spagitty writes from blocks are read from the blocks.
 */

import type { HookStep, Hooks, StepCategory } from '../types';
import type { Block, Definition } from './builder-types';

export type Moment = 'commit' | 'push' | 'merge' | 'checkout' | 'rebase' | 'other';

/** Git's moments, in the order they come in a day's work, each with its hooks in git's order. */
export const MOMENTS: { id: Moment; label: string; hooks: string[] }[] = [
	{ id: 'commit', label: 'When you commit', hooks: ['pre-commit', 'prepare-commit-msg', 'commit-msg', 'post-commit'] },
	{ id: 'push', label: 'When you push', hooks: ['pre-push'] },
	{ id: 'merge', label: 'When you merge or pull', hooks: ['pre-merge-commit', 'post-merge'] },
	{ id: 'checkout', label: 'When you switch branches', hooks: ['post-checkout'] },
	{ id: 'rebase', label: 'When you rebase or amend', hooks: ['pre-rebase', 'post-rewrite'] }
];

/** When each hook runs, and whether it can stop what git is doing. */
export const HOOK_SAYS: Record<string, string> = {
	'pre-commit': 'Before the commit is made. A failure stops the commit.',
	'prepare-commit-msg': 'Before the message is written, to fill it in.',
	'commit-msg': 'On the message. A failure stops the commit.',
	'post-commit': 'After the commit is made. It cannot stop it.',
	'pre-push': 'Before anything is sent. A failure stops the push.',
	'pre-merge-commit': 'Before a merge commit is made. A failure stops the merge.',
	'post-merge': 'After a merge or a pull has finished.',
	'post-checkout': 'After switching branches or checking files out.',
	'pre-rebase': 'Before a rebase starts. A failure stops it.',
	'post-rewrite': 'After an amend or a rebase rewrote commits.'
};

/** Every kind, in the order the filter lists them, with its colour. */
export const CATEGORIES: { id: StepCategory; label: string; tone: string }[] = [
	{ id: 'format', label: 'Format & lint', tone: 'var(--warn)' },
	{ id: 'tests', label: 'Tests', tone: 'var(--lane-5)' },
	{ id: 'build', label: 'Build & types', tone: 'var(--lane-1)' },
	{ id: 'message', label: 'Commit message', tone: 'var(--lane-4)' },
	{ id: 'secrets', label: 'Secrets', tone: 'var(--danger)' },
	{ id: 'files', label: 'Files', tone: 'var(--lane-2)' },
	{ id: 'branches', label: 'Branches', tone: 'var(--lane-3)' },
	{ id: 'dependencies', label: 'Dependencies', tone: 'var(--agent)' },
	{ id: 'other', label: 'Other', tone: 'var(--muted)' }
];

export function categoryOf(id: StepCategory): { id: StepCategory; label: string; tone: string } {
	return CATEGORIES.find((each) => each.id === id) ?? CATEGORIES[CATEGORIES.length - 1];
}

/** A step on the screen: one the backend read from a file, or one of Spagitty's blocks. */
export interface Shown extends HookStep {
	key: string;
	/** The block it is, when Spagitty runs it. */
	block: string | null;
	/** Switched off: the block or its stage is not enabled. */
	off: boolean;
}

/** A script block's kind, from its command's words: the commonest few. */
function guess(command: string): StepCategory {
	const words = command.toLowerCase().split(/[^a-z0-9:_-]+/);
	const has = (...wanted: string[]) => words.some((word) => wanted.some((want) => word === want || word.startsWith(`${want}:`)));
	if (has('gitleaks', 'detect-secrets', 'trufflehog')) return 'secrets';
	if (has('test', 'tests', 'jest', 'vitest', 'pytest', 'verify', 'e2e')) return 'tests';
	if (has('tsc', 'typecheck', 'build', 'compile', 'mypy')) return 'build';
	if (has('lint', 'eslint', 'prettier', 'format', 'fmt', 'ruff', 'black', 'lint-staged', 'spotless')) return 'format';
	if (has('install', 'ci')) return 'dependencies';
	return 'other';
}

const BLOCK_CATEGORY: Record<Block['action']['kind'], StepCategory | null> = {
	tests: 'tests',
	format: 'format',
	message: 'message',
	script: null,
	secrets: 'secrets',
	largeFiles: 'files',
	protectBranches: 'branches'
};

/** What a block runs, said as a command. */
function commandOf(block: Block): string {
	const action = block.action;
	switch (action.kind) {
		case 'tests':
		case 'format':
		case 'script':
			return action.command;
		case 'message':
			return action.style === 'conventional' ? 'Conventional Commits' : action.pattern || action.style;
		case 'secrets':
			return 'Keys and tokens in the staged diff';
		case 'largeFiles':
			return `Files over ${Math.round(action.bytes / 1024 / 1024)} MB`;
		case 'protectBranches':
			return `No direct commit or push to ${action.branches.join(', ')}`;
	}
}

/** Spagitty's blocks as steps, numbered within their stage. */
export function blockSteps(definition: Definition | null): Shown[] {
	if (!definition) return [];
	const numbered = new Map<string, number>();
	return definition.blocks.map((block) => {
		const order = (numbered.get(block.stage) ?? 0) + 1;
		numbered.set(block.stage, order);
		const command = commandOf(block);
		return {
			key: `block:${block.id}`,
			block: block.id,
			hook: block.stage,
			order,
			name: block.label,
			command,
			category: BLOCK_CATEGORY[block.action.kind] ?? guess(command),
			file: definition.shared ? '.spagitty/hooks.toml' : 'Personal hooks',
			line: null,
			together: false,
			off: !block.enabled || !definition.activeStages.includes(block.stage)
		};
	});
}

/** Every step the repository runs: what the backend read, and Spagitty's blocks where it writes the hooks. */
export function allSteps(info: Hooks | null, definition: Definition | null, blocks: boolean): Shown[] {
	const read: Shown[] = (info?.steps ?? []).map((step) => ({
		...step,
		key: `${step.file}:${step.hook}:${step.order}`,
		block: null,
		off: !(info?.enabled ?? true)
	}));
	return blocks ? [...read, ...blockSteps(definition)] : read;
}

export interface Laid {
	moment: Moment;
	label: string;
	hooks: { hook: string; says: string; steps: (Shown & { place: number })[] }[];
}

/**
 * The steps by moment, each moment's hooks in git's order, and each step
 * with its `place`: where it comes in the moment, counting from 1 through
 * every hook before it. Only `filter`'s kind is kept; the places are the
 * steps' own, so a filtered list still says *3rd*.
 */
export function arrange(steps: Shown[], filter: StepCategory | 'all' = 'all'): Laid[] {
	const known = new Set(MOMENTS.flatMap((moment) => moment.hooks));
	const rest = [...new Set(steps.map((step) => step.hook).filter((hook) => !known.has(hook)))];
	const moments = [...MOMENTS, { id: 'other' as const, label: 'At other times', hooks: rest }];
	const out: Laid[] = [];
	for (const moment of moments) {
		let place = 0;
		const hooks: Laid['hooks'] = [];
		for (const hook of moment.hooks) {
			const mine = steps
				.filter((step) => step.hook === hook)
				// Read from a file first, then Spagitty's blocks, each in its own order.
				.sort((a, b) => Number(a.block !== null) - Number(b.block !== null) || a.order - b.order)
				.map((step) => ({ ...step, place: step.off ? 0 : ++place }));
			const kept = mine.filter((step) => filter === 'all' || step.category === filter);
			if (kept.length) hooks.push({ hook, says: HOOK_SAYS[hook] ?? '', steps: kept });
		}
		if (hooks.length) out.push({ moment: moment.id, label: moment.label, hooks });
	}
	return out;
}

/** How many steps of each kind there are, for the filter. */
export function counts(steps: Shown[]): Record<StepCategory | 'all', number> {
	const out = { all: steps.length } as Record<StepCategory | 'all', number>;
	for (const category of CATEGORIES) out[category.id] = steps.filter((step) => step.category === category.id).length;
	return out;
}

/** `1st`, `2nd`, `3rd`, `11th`. */
export function ordinal(n: number): string {
	const tens = n % 100;
	if (tens >= 11 && tens <= 13) return `${n}th`;
	return `${n}${['th', 'st', 'nd', 'rd'][n % 10] ?? 'th'}`;
}
