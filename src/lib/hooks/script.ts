// SPDX-License-Identifier: MIT

import { detectLanguage } from '$lib/diff/highlight';

/**
 * The language a hook file is written in, for its colour (FEAT-171).
 *
 * A hook is usually a file with no extension — `.husky/pre-commit`,
 * `.git/hooks/pre-push` — so the name says nothing and the first line says
 * everything: `#!/bin/sh`, `#!/usr/bin/env node`. Without either, it is a
 * shell script, which is what git runs a hook as.
 */
export function scriptLanguage(path: string, text: string): string {
	const named = detectLanguage(path);
	if (named !== 'plain') return named;
	const first = text.split('\n', 1)[0] ?? '';
	const bang = /^#!\s*(?:\S*\/)?(?:env\s+(?:-\S+\s+)*)?(\S+)/.exec(first);
	const program = bang?.[1].replace(/[\d.]+$/, '') ?? '';
	if (/^(?:node|deno|bun)$/.test(program)) return 'javascript';
	if (/^(?:python|pypy)$/.test(program)) return 'python';
	if (program === 'ruby') return 'ruby';
	return 'shell';
}
