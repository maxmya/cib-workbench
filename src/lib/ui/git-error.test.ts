// SPDX-License-Identifier: MIT

import { describe, expect, it } from 'vitest';
import { cleanGitOutput, explainGitError, failureDetail, SHOWN_LINES } from './git-error';

/** BUG-107: what git printed, said as what went wrong and what to do. */
describe("git's failures, in words", () => {
	it('says a push behind the remote needs a pull first', () => {
		const raw =
			'! [rejected] main -> main (fetch first)\nerror: failed to push some refs to \'origin\'\nhint: Updates were rejected because the remote contains work that you do not\nhint: have locally.';
		const { why, output } = explainGitError(raw);

		expect(why).toBe('The remote has commits that your branch does not. Pull first, then push again.');
		expect(output).not.toContain('hint:');
		expect(output).toContain('failed to push some refs');
	});

	it('names a protected branch, and a hook that refused', () => {
		expect(explainGitError('! [remote rejected] main -> main (protected branch hook declined)').why).toContain(
			'protects this branch'
		);
		expect(explainGitError('remote: GH006: Protected branch update failed for refs/heads/main.').why).toContain(
			'protects this branch'
		);
		expect(explainGitError('! [remote rejected] main -> main (pre-receive hook declined)').why).toContain(
			"remote's own checks"
		);
	});

	it('tells a sign-in refused from a remote that cannot be reached', () => {
		expect(explainGitError("fatal: Authentication failed for 'https://example.com/x.git/'").why).toContain(
			'did not accept your sign-in'
		);
		expect(
			explainGitError("fatal: unable to access 'https://example.com/x.git/': Could not resolve host: example.com").why
		).toContain('could not be reached');
		expect(explainGitError('The requested URL returned error: 403').why).toContain('not allowed to write');
	});

	it('says which uncommitted files a pull would overwrite', () => {
		const raw =
			'error: Your local changes to the following files would be overwritten by merge:\n\tsrc/a.ts\n\tsrc/b.ts\nPlease commit your changes or stash them before you merge.\nAborting';
		expect(explainGitError(raw).why).toBe(
			'Your uncommitted changes would be overwritten. Commit or stash them first, then try again. The files: src/a.ts, src/b.ts.'
		);
	});

	it('says a pull that cannot fast-forward needs merge or rebase', () => {
		expect(explainGitError('fatal: Not possible to fast-forward, aborting.').why).toContain('Pull with merge or with rebase');
	});

	it('says a stash with nothing to stash, and conflicts in the way', () => {
		expect(explainGitError('No local changes to save').why).toContain('nothing to stash');
		expect(explainGitError('src/a.ts: needs merge').why).toContain('conflicts still to resolve');
		expect(explainGitError("fatal: Unable to create '/r/.git/index.lock': File exists.").why).toContain('index.lock');
	});

	it('says what to do about an exit code with no words', () => {
		expect(explainGitError('git push failed (exit code: 1)').why).toContain('code 1 and gave no reason');
		expect(explainGitError('git fetch failed (exit status: 128)').why).toContain('code 128');
	});

	it('leaves the command line Spagitty ran out of what is shown', () => {
		const raw = 'git -c credential.helper= -c core.askPass= stash push -m wip failed: error: something odd happened';
		expect(cleanGitOutput(raw)).toBe('something odd happened');
	});

	it('passes a message that is not git’s through as it came', () => {
		expect(explainGitError('That is not a recipe: expected value')).toEqual({
			why: null,
			output: 'That is not a recipe: expected value'
		});
		expect(failureDetail('Claude Code is not installed.')).toEqual({ detail: 'Claude Code is not installed.', output: null });
	});

	it('cuts an unknown page of output to its first lines, and keeps the rest', () => {
		const raw = Array.from({ length: 10 }, (_, index) => `line ${index}`).join('\n');
		const { detail, output } = failureDetail(raw);

		expect(detail.split('\n')).toHaveLength(SHOWN_LINES + 1);
		expect(detail.endsWith('…')).toBe(true);
		expect(output).toBe(raw);
	});
});
