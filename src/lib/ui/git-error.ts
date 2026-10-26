// SPDX-License-Identifier: MIT

/**
 * git's failures, in words (BUG-107).
 *
 * A push, a pull or a stash that failed said what git printed: sometimes a
 * page of it, `hint:` lines and all, behind the whole command line Spagitty
 * ran; sometimes nothing but *git push failed (exit code: 1)*. Neither tells
 * somebody who pressed a button what went wrong or what to do next.
 *
 * `explainGitError` reads git's text for the failures people meet every day
 * and says each in a sentence: what happened, and what to do. git's own
 * lines are kept, cleaned of hints and of the command line, for whoever
 * wants them; a failure this does not recognise is said in git's words, cut
 * to its first lines.
 *
 * The rules match git's own phrases, not words that could be anyone's, so a
 * message that is not from git passes through as it came.
 */

export interface Explained {
	/** What went wrong and what to do, or `null` when no rule knows it. */
	why: string | null;
	/** git's own lines, cleaned: no hints, no command line. `null` when there were none. */
	output: string | null;
}

/** How many of git's lines a notice shows when no rule explains them. */
export const SHOWN_LINES = 4;

interface Rule {
	test: RegExp;
	say: (match: RegExpMatchArray, text: string) => string;
}

/** Files git listed under a line ending `:`, indented by a tab. */
function listedFiles(text: string): string[] {
	return text
		.split('\n')
		.filter((line) => /^\t\S/.test(line))
		.map((line) => line.trim())
		.slice(0, 5);
}

function withFiles(sentence: string, text: string): string {
	const files = listedFiles(text);
	return files.length ? `${sentence} ${files.length === 1 ? 'The file' : 'The files'}: ${files.join(', ')}.` : sentence;
}

/** The first that matches says it. Order matters: specific before general. */
const RULES: Rule[] = [
	{
		test: /\[remote rejected\][^\n]*\((?:protected branch hook declined|.*protected branch.*)\)|GH006: Protected branch|You are not allowed to push code to protected branches/i,
		say: () =>
			'The remote protects this branch, so it cannot be pushed to directly. Push to a branch of your own and open a pull request.'
	},
	{
		test: /\[remote rejected\][^\n]*\(pre-receive hook declined\)/,
		say: () => "The remote's own checks refused the push. What they said is below."
	},
	{
		test: /\[rejected\][^\n]*\((?:fetch first|non-fast-forward)\)|Updates were rejected because the (?:remote contains work|tip of your current branch is behind)/,
		say: () => 'The remote has commits that your branch does not. Pull first, then push again.'
	},
	{
		test: /\[rejected\][^\n]*\(stale info\)/,
		say: () => 'The remote branch moved since it was last fetched, so the push was refused to keep what is there. Fetch, look, then push again.'
	},
	{
		test: /has no upstream branch|no upstream configured|There is no tracking information for the current branch/,
		say: () =>
			'This branch is not on the remote yet, so there is nothing to pull from and no place set to push to. Push it once to create it there.'
	},
	{
		test: /src refspec \S+ does not match any/,
		say: () => 'There is nothing to push: the branch has no commits yet, or no branch has that name.'
	},
	{
		test: /Authentication failed|could not read Username|could not read Password|HTTP Basic: Access denied|Invalid username or (?:password|token)|The requested URL returned error: 401|Permission denied \((?:publickey|keyboard-interactive)/i,
		say: () =>
			'The remote did not accept your sign-in. Check the account in Settings › Accounts, your token, or your SSH key.'
	},
	{
		test: /Permission to \S+ denied|The requested URL returned error: 403|You don't have permission|insufficient permission/i,
		say: () => 'Your account can read this repository but is not allowed to write to it.'
	},
	{
		test: /Repository not found|does not appear to be a git repository|The project you were looking for could not be found/,
		say: () => 'The remote repository was not found, or your account cannot see it. Check its address under Remotes.'
	},
	{
		test: /Could not resolve host|Failed to connect to|Connection (?:timed out|refused)|Network is unreachable|Operation timed out|unable to access '[^']*': (?:Could not|Failed|SSL|OpenSSL|schannel)/i,
		say: () => 'The remote could not be reached. Check your network or VPN, then try again.'
	},
	{
		test: /The requested URL returned error: 5\d\d|Internal Server Error|RPC failed; HTTP 5\d\d/,
		say: () => 'The remote had a problem of its own. Try again in a moment.'
	},
	{
		test: /exceeds (?:GitHub's )?file size limit|this exceeds .* size limit|File \S+ is [\d.]+ MB; this exceeds|pack exceeds maximum allowed size/i,
		say: () => 'A file in what you are pushing is larger than the remote allows.'
	},
	{
		test: /Your local changes to the following files would be overwritten/,
		say: (_, text) => withFiles('Your uncommitted changes would be overwritten. Commit or stash them first, then try again.', text)
	},
	{
		test: /untracked working tree files would be (?:overwritten|removed)/,
		say: (_, text) => withFiles('Files that git does not track would be overwritten. Move or delete them, then try again.', text)
	},
	{
		test: /Not possible to fast-forward|have diverged|diverging branches|Need to specify how to reconcile divergent branches/,
		say: () =>
			'Your branch and the remote have both moved on, so the remote cannot simply be added on top. Pull with merge or with rebase instead.'
	},
	{
		test: /refusing to merge unrelated histories/,
		say: () => 'The two branches share no history, so git will not merge them.'
	},
	{
		test: /^CONFLICT|Automatic merge failed|could not apply [0-9a-f]+|Resolve all conflicts manually/m,
		say: () => 'git stopped on conflicts. Resolve them in Conflicts, then continue.'
	},
	{
		test: /needs merge|you need to resolve your current index first|is unmerged|unmerged files/,
		say: () => 'There are conflicts still to resolve. Resolve them in Conflicts first.'
	},
	{
		test: /index\.lock': File exists|Another git process seems to be running/,
		say: () =>
			'Another git program is working in this repository. Wait for it to finish; if none is running, delete .git/index.lock.'
	},
	{
		test: /No local changes to save/,
		say: () => 'There is nothing to stash: the working copy has no changes.'
	},
	{
		test: /No stash entries found|is not a stash reference|No stash found/,
		say: () => 'That stash is not there any more.'
	},
	{
		test: /Please (?:tell me who you are|set your user\.name)|Author identity unknown|empty ident name/,
		say: () => 'git does not know who you are. Set your name and email in Settings › You.'
	},
	{
		test: /You are not currently on a branch|HEAD detached/,
		say: () => 'No branch is checked out, so there is nothing to pull into or push from. Switch to a branch first.'
	},
	{
		test: /\bfailed \(exit (?:code|status):? ?(\d+)\)\s*$/,
		say: (match) =>
			`git stopped with code ${match[1]} and gave no reason. Turn on Settings › Behaviour › Show the git command behind each action to see what ran.`
	}
];

/** git's text without its hints, its prefixes or the command line Spagitty ran. */
export function cleanGitOutput(raw: string): string | null {
	const lines = raw
		.replace(/\r\n?/g, '\n')
		// `git -c credential.helper=… push --porcelain origin main failed: ` — the
		// command line is ours, not the person's, and can be most of the text.
		.replace(/^git\s.*?\sfailed:\s*/s, (prefix) => (prefix.length < 2000 ? '' : prefix))
		.split('\n')
		.filter((line) => !/^\s*hint:/.test(line))
		.map((line) => line.replace(/^(?:remote: )?(?:error|fatal): /, '').replace(/\s+$/, ''))
		.filter((line, index, all) => line !== '' || (index > 0 && all[index - 1] !== ''));
	const text = lines.join('\n').trim();
	return text === '' ? null : text;
}

export function explainGitError(raw: string): Explained {
	const output = cleanGitOutput(raw);
	for (const rule of RULES) {
		const match = raw.match(rule.test);
		if (match) return { why: rule.say(match, raw), output };
	}
	return { why: null, output };
}

/**
 * What a notice says about a failure: a sentence when one is known and git's
 * first lines when not, with git's whole text kept when it says more.
 */
export function failureDetail(raw: string): { detail: string; output: string | null } {
	const { why, output } = explainGitError(raw);
	if (why) return { detail: why, output };
	const text = output ?? raw.trim();
	const lines = text.split('\n');
	if (lines.length <= SHOWN_LINES) return { detail: text, output: null };
	return { detail: `${lines.slice(0, SHOWN_LINES).join('\n')}\n…`, output: text };
}
