#!/usr/bin/env node
// SPDX-License-Identifier: MIT

/**
 * Workbench is MIT. Spagitty's files say `SPDX-License-Identifier:
 * GPL-3.0-or-later`, and every upstream merge brings new ones, so this
 * rewrites them to MIT. Spagitty's sole author licenses Workbench under MIT;
 * Spagitty itself stays GPL-3.0-or-later.
 *
 *     node tools/brand-spdx.mjs           # rewrites them
 *     node tools/brand-spdx.mjs --check   # fails if any is left
 *
 * It reads only files git tracks, so nothing vendored, downloaded or
 * generated (node_modules, multiplexer/bundle, THIRD-PARTY-LICENSES.md) is
 * touched, and only the exact identifier line Spagitty writes: a third-party
 * file carries its own licence's identifier, never this one. The identifier
 * is replaced wherever it is written, so a generator that writes a header
 * (`tools/sweep/index.ts`) writes MIT too.
 */

import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const GPL = 'SPDX-License-Identifier: GPL-3.0-or-later';
const MIT = 'SPDX-License-Identifier: MIT';

/** Files that quote the GPL identifier as a subject rather than carry it. */
export const QUOTES = new Set([
	'design_handoff_cib_workbench/WORKBENCH_PLAN.md',
	'tools/brand-spdx.mjs',
	'tools/brand-spdx.test.ts'
]);

/** Tracked files that still say GPL, quotes aside. */
export function remaining(root = process.cwd()) {
	let listed;
	try {
		listed = execFileSync('git', ['grep', '-l', '-F', GPL], { cwd: root, encoding: 'utf8' });
	} catch (error) {
		// `git grep` exits 1 when nothing matches: that is the clean case.
		if (error.status === 1) return [];
		throw error;
	}
	return listed
		.split('\n')
		.filter(Boolean)
		.filter((path) => !QUOTES.has(path));
}

function main(argv) {
	const left = remaining();
	if (argv.includes('--check')) {
		if (left.length) {
			console.error(`${left.length} files still say GPL-3.0-or-later; run node tools/brand-spdx.mjs:`);
			for (const path of left.slice(0, 20)) console.error(`  ${path}`);
			return 1;
		}
		console.log('every first-party file is MIT');
		return 0;
	}
	for (const path of left) {
		writeFileSync(path, readFileSync(path, 'utf8').split(GPL).join(MIT));
	}
	console.log(`rewrote ${left.length} files to MIT`);
	return 0;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
	process.exitCode = main(process.argv.slice(2));
}
