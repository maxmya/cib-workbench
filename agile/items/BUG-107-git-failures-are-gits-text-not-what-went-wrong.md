<!-- SPDX-License-Identifier: MIT -->

# BUG-107 — git's failures are git's text, not what went wrong

**Status:** Fixed — in the working tree on `release/1.0.1`, uncommitted. Covered by `ui/git-error.test.ts` and the toolbar's checkout test.
**Branch:** none yet.
**Screens:** the toolbar, the notice and the notification center: every push, pull, fetch, stash and switch.
**Raised by:** the author, 2026-10-26: "when git throws an error we display it on the horizontal rail ! … we should have better feedback and handling for the errors and not say to user git exit with 1 !! explain more what is wrong also in notification i get huge exceptions".

## Cause

A failure was shown as git printed it: a page with `hint:` lines, behind the whole command line Spagitty ran, and run together on one line across the toolbar; or, when git said nothing, *git push failed (exit code: 1)*. The notification center kept the first line, which was often the command line.

## Fix

- `explainGitError` reads git's text for the failures people meet every day and says each as what happened and what to do: a push behind the remote, a protected branch, a hook that refused, no upstream, a sign-in refused, no write access, a remote not found or not reachable, uncommitted changes in the way (naming the files), a pull that cannot fast-forward, conflicts, `index.lock`, nothing to stash, no identity, a detached HEAD, and an exit code with no words. The rules match git's own phrases, so a message that is not git's passes through.
- `notice.failed` says the sentence, and keeps git's lines, cleaned of hints and of the command line, behind *Show git's output*. With no rule to apply, it shows git's first four lines. The notification center keeps the sentence.
- The toolbar says *Push failed · why?*, *Fetch failed · why?* or *Switch failed · why?*: the why on hover, the notice on a click.
