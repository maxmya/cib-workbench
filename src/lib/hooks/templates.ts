// SPDX-License-Identifier: MIT

/**
 * Hooks to start from (FEAT-176): the *Create hook* tab's catalogue.
 *
 * Each is a check teams add to a repository, said in a sentence — what it
 * does — and another — why — with the hook it runs in, what it needs
 * installed, and the command. How it is added depends on who runs the
 * repository's hooks:
 *
 * - **Husky**: the command is added to `.husky/<hook>`.
 * - **Spagitty** (no manager): it becomes a block, saved with the others.
 * - **lefthook** and **pre-commit**: their configuration is the person's,
 *   so the lines to paste into it are given, rather than the file rewritten.
 */

import type { HookManager, StepCategory } from '../types';
import type { Action, Stage } from './builder-types';

export interface Template {
	id: string;
	title: string;
	category: StepCategory;
	/** The git hook it runs in. */
	hook: string;
	/** What it does, in a sentence. */
	says: string;
	/** Why a team adds it. */
	why: string;
	/** What has to be installed for it to run. */
	needs?: string;
	/** The shell it runs. `$1` is the message file in `commit-msg`. */
	command: string;
	/** The block it is when Spagitty writes the hooks; a script block when absent. */
	block?: Action;
}

export const TEMPLATES: Template[] = [
	{
		id: 'lint-staged',
		title: 'Format and lint what is staged',
		category: 'format',
		hook: 'pre-commit',
		says: 'Runs the formatters and linters lint-staged is set up with, on the staged files only.',
		why: 'Every commit is formatted the same way, and a lint error is caught before review rather than in it.',
		needs: 'lint-staged in package.json, with its rules',
		command: 'npx lint-staged',
		block: { kind: 'script', command: 'npx lint-staged' }
	},
	{
		id: 'prettier',
		title: 'Prettier on staged files',
		category: 'format',
		hook: 'pre-commit',
		says: 'Checks the staged JavaScript, TypeScript, JSON, CSS and Markdown files are formatted as Prettier would.',
		why: 'No commit argues about spacing, and diffs show only what changed.',
		needs: 'prettier',
		command:
			"files=$(git diff --cached --name-only --diff-filter=ACMR -- '*.js' '*.jsx' '*.ts' '*.tsx' '*.json' '*.css' '*.scss' '*.md'); [ -z \"$files\" ] || npx prettier --check $files",
		block: { kind: 'format', preset: 'prettier', paths: ['**'], command: 'npx prettier --write', restage: false }
	},
	{
		id: 'eslint',
		title: 'ESLint on staged files',
		category: 'format',
		hook: 'pre-commit',
		says: 'Lints the staged JavaScript and TypeScript files and stops the commit on an error.',
		why: 'Unused variables, missing awaits and broken imports are caught on the machine that made them.',
		needs: 'eslint, with a configuration',
		command:
			"files=$(git diff --cached --name-only --diff-filter=ACMR -- '*.js' '*.jsx' '*.ts' '*.tsx'); [ -z \"$files\" ] || npx eslint --max-warnings=0 $files",
		block: { kind: 'format', preset: 'eslint', paths: ['**/*.{js,jsx,ts,tsx}'], command: 'npx eslint --max-warnings=0', restage: false }
	},
	{
		id: 'spotless',
		title: 'Spotless (Java)',
		category: 'format',
		hook: 'pre-commit',
		says: 'Checks the Java sources are formatted the way the build says, with Maven’s Spotless plugin.',
		why: 'The CI build fails on formatting otherwise, after the push, which is the slow way to learn it.',
		needs: 'the Spotless plugin in pom.xml, and the Maven wrapper',
		command: './mvnw -q spotless:check',
		block: { kind: 'script', command: './mvnw -q spotless:check' }
	},
	{
		id: 'no-debug',
		title: 'No console.log or debugger',
		category: 'format',
		hook: 'pre-commit',
		says: 'Stops a commit whose added lines call console.log or leave a debugger statement.',
		why: 'Debugging output left in a commit ends up in the browser console of every user.',
		command:
			"if git diff --cached -U0 -- '*.js' '*.jsx' '*.ts' '*.tsx' | grep -qE '^\\+.*(console\\.log\\(|debugger;)'; then echo 'console.log or debugger is staged.' >&2; exit 1; fi"
	},
	{
		id: 'tsc',
		title: 'TypeScript type check',
		category: 'build',
		hook: 'pre-commit',
		says: 'Type-checks the whole project without writing any output.',
		why: 'A type error in one file can break another that was not touched; this sees the whole program.',
		needs: 'typescript, and a tsconfig.json',
		command: 'npx tsc --noEmit',
		block: { kind: 'script', command: 'npx tsc --noEmit' }
	},
	{
		id: 'npm-test',
		title: 'Tests before pushing',
		category: 'tests',
		hook: 'pre-push',
		says: 'Runs the test suite before anything is sent, and stops the push when a test fails.',
		why: 'A broken build is caught on your machine, not by everyone who pulls after you.',
		needs: 'a test script in package.json',
		command: 'npm test',
		block: { kind: 'tests', paths: ['**'], command: 'npm test' }
	},
	{
		id: 'maven-verify',
		title: 'Maven verify before pushing',
		category: 'tests',
		hook: 'pre-push',
		says: 'Compiles, runs the unit tests and the checks the build has, before anything is sent.',
		why: 'The same build CI runs, on your machine, so a push is one that passes.',
		needs: 'the Maven wrapper',
		command: './mvnw -q -DskipITs verify',
		block: { kind: 'tests', paths: ['**'], command: './mvnw -q -DskipITs verify' }
	},
	{
		id: 'commitlint',
		title: 'Conventional Commits (commitlint)',
		category: 'message',
		hook: 'commit-msg',
		says: 'Checks the message reads like feat(scope): what changed, and stops the commit when it does not.',
		why: 'Release notes and version numbers can then be made from the history.',
		needs: '@commitlint/cli and a commitlint configuration',
		command: 'npx --no -- commitlint --edit "$1"',
		block: {
			kind: 'message',
			style: 'conventional',
			types: ['feat', 'fix', 'docs', 'refactor', 'perf', 'test', 'chore'],
			scopeRequired: false,
			subjectMax: 72,
			pattern: '',
			ticketPattern: ''
		}
	},
	{
		id: 'ticket',
		title: 'The message names a ticket',
		category: 'message',
		hook: 'commit-msg',
		says: 'Stops a commit whose message names no ticket, like ABC-123.',
		why: 'Every change can be traced to the work item that asked for it.',
		command:
			"grep -qE '[A-Z][A-Z0-9]+-[0-9]+' \"$1\" || { echo 'The message should name a ticket, like ABC-123.' >&2; exit 1; }",
		block: {
			kind: 'message',
			style: 'ticket',
			types: [],
			scopeRequired: false,
			subjectMax: 72,
			pattern: '^[A-Z][A-Z0-9]+-[0-9]+',
			ticketPattern: ''
		}
	},
	{
		id: 'gitleaks',
		title: 'Block secrets (gitleaks)',
		category: 'secrets',
		hook: 'pre-commit',
		says: 'Scans what is staged for keys, tokens and passwords, and stops the commit when it finds one.',
		why: 'A secret pushed once is in the history for good, and has to be rotated.',
		needs: 'gitleaks',
		command: 'gitleaks protect --staged --redact',
		block: { kind: 'secrets', patterns: [] }
	},
	{
		id: 'private-key',
		title: 'Block private keys',
		category: 'secrets',
		hook: 'pre-commit',
		says: 'Stops a commit that adds a private key.',
		why: 'Needs nothing installed, and catches the commonest leak.',
		command:
			"if git diff --cached -U0 | grep -qE '^\\+.*BEGIN [A-Z ]*PRIVATE KEY'; then echo 'A private key is staged.' >&2; exit 1; fi",
		block: { kind: 'secrets', patterns: [] }
	},
	{
		id: 'large-files',
		title: 'No files over 5 MB',
		category: 'files',
		hook: 'pre-commit',
		says: 'Stops a commit that adds or changes a file larger than 5 MB.',
		why: 'A large file in the history makes every clone slower, for ever. Use Git LFS for them.',
		command:
			"for file in $(git diff --cached --name-only --diff-filter=AM); do [ ! -f \"$file\" ] || [ $(wc -c < \"$file\") -le 5242880 ] || { echo \"$file is over 5 MB.\" >&2; exit 1; }; done",
		block: { kind: 'largeFiles', bytes: 5 * 1024 * 1024 }
	},
	{
		id: 'conflict-markers',
		title: 'No conflict markers',
		category: 'files',
		hook: 'pre-commit',
		says: 'Stops a commit that adds <<<<<<< or >>>>>>> lines left from a merge.',
		why: 'A half-resolved conflict compiles in some languages and fails at run time.',
		command:
			"if git diff --cached -U0 | grep -qE '^\\+(<<<<<<<|>>>>>>>)( |$)'; then echo 'Conflict markers are staged.' >&2; exit 1; fi"
	},
	{
		id: 'protect-main',
		title: 'No commits straight to main',
		category: 'branches',
		hook: 'pre-commit',
		says: 'Stops a commit made on main or master.',
		why: 'Changes go through a branch and a pull request, where they are reviewed.',
		command:
			"branch=$(git rev-parse --abbrev-ref HEAD); if [ \"$branch\" = main ] || [ \"$branch\" = master ]; then echo \"Commit on a branch, not on $branch.\" >&2; exit 1; fi",
		block: { kind: 'protectBranches', branches: ['main', 'master'] }
	},
	{
		id: 'branch-name',
		title: 'Branch names follow the rule',
		category: 'branches',
		hook: 'pre-push',
		says: 'Stops a push from a branch whose name does not start with feature/, fix/, hotfix/, release/ or chore/.',
		why: 'Pipelines and boards can tell what a branch is for from its name.',
		command:
			"branch=$(git rev-parse --abbrev-ref HEAD); echo \"$branch\" | grep -qE '^(feature|fix|hotfix|release|chore)/' || { echo 'Branch names start with feature/, fix/, hotfix/, release/ or chore/.' >&2; exit 1; }"
	},
	{
		id: 'install-after-pull',
		title: 'Install after a pull changed the lockfile',
		category: 'dependencies',
		hook: 'post-merge',
		says: 'After a pull or a merge, runs npm install when package-lock.json changed.',
		why: 'Nobody runs yesterday’s dependencies against today’s code and wonders why it fails.',
		needs: 'npm',
		command: "git diff --name-only HEAD@{1} HEAD | grep -q 'package-lock.json' && npm install || true"
	},
	{
		id: 'install-after-checkout',
		title: 'Install after switching to a branch with other dependencies',
		category: 'dependencies',
		hook: 'post-checkout',
		says: 'After switching branches, runs npm install when the two branches’ lockfiles differ.',
		why: 'A branch that added a package works the moment you switch to it.',
		needs: 'npm',
		command:
			"[ \"$3\" = 1 ] && git diff --name-only \"$1\" \"$2\" | grep -q 'package-lock.json' && npm install || true"
	}
];

/** The stages Spagitty's own hooks have; a template for another hook is added another way. */
export const BLOCK_STAGES: Stage[] = ['pre-commit', 'commit-msg', 'pre-push'];

/** How a template is added where `manager` runs the hooks. */
export type Way = 'husky' | 'blocks' | 'lefthook' | 'preCommit' | 'none';

export function wayFor(manager: HookManager, template: Template): Way {
	switch (manager) {
		case 'husky':
			return 'husky';
		case 'lefthook':
			return 'lefthook';
		case 'preCommit':
			return 'preCommit';
		case 'git':
			return BLOCK_STAGES.includes(template.hook as Stage) ? 'blocks' : 'none';
	}
}

/** A string as YAML's double-quoted scalar. */
function quoted(text: string): string {
	return `"${text.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"`;
}

/** The lines to add to `lefthook.yml`. lefthook passes git's arguments as `{1}`. */
export function lefthookSnippet(template: Template): string {
	const run = template.command.replace(/"\$1"/g, '{1}').replace(/\$1\b/g, '{1}');
	return `${template.hook}:\n  commands:\n    ${template.id}:\n      run: ${quoted(run)}\n`;
}

/** The lines to add under `repos:` in `.pre-commit-config.yaml`: a local hook run by the shell. */
export function preCommitSnippet(template: Template): string {
	const message = template.hook === 'commit-msg' || template.hook === 'prepare-commit-msg';
	return [
		'  - repo: local',
		'    hooks:',
		`      - id: ${template.id}`,
		`        name: ${quoted(template.title)}`,
		`        entry: sh -c ${quoted(template.command)} --`,
		'        language: system',
		`        pass_filenames: ${message ? 'true' : 'false'}`,
		`        stages: [${template.hook}]`,
		''
	].join('\n');
}

/** True when the repository already runs this template's command. */
export function present(template: Template, commands: string[]): boolean {
	const wanted = template.command.replace(/\s+/g, ' ').trim();
	return commands.some((command) => command.replace(/\s+/g, ' ').trim() === wanted);
}
