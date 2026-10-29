// SPDX-License-Identifier: MIT

/**
 * Which screens need a loaded repository (proposal § 3, FEAT-184).
 *
 * With nothing loaded their rail rows are dimmed, and reaching one opens Load
 * repository instead of drawing an empty screen. Home, Calls, Farm, Settings,
 * All repositories and the forge lists are never dimmed.
 */

export const SCOPED_ROUTES = [
	'/',
	'/diff',
	'/changes',
	'/branches',
	'/stash',
	'/rebase',
	'/conflicts',
	'/merge',
	'/review',
	'/search',
	'/reflog',
	'/tags',
	'/files',
	'/history',
	'/terminal',
	'/recipes',
	'/hooks',
	'/investigate',
	'/untrack'
] as const;

/** True when `pathname` is a repository-scoped screen. */
export function isScoped(pathname: string): boolean {
	const path = pathname.replace(/\/+$/, '') || '/';
	if (path === '/') return true;
	return SCOPED_ROUTES.some((route) => route !== '/' && (path === route || path.startsWith(`${route}/`)));
}
