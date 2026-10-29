// SPDX-License-Identifier: MIT

/**
 * Open a file in Files from any screen.
 *
 * A row that names a file says so with `data-file-path`, and the revision
 * it is showing with `data-file-rev`. The window's right-click menu
 * (`FileMenu.svelte`) reads them and offers **Open in Files** at that
 * revision. A file Files cannot read, one not in a commit yet, carries
 * `data-file-absent` with the reason, and the entry is shown with it.
 *
 * Every such row also opens on a double-click, or a click with Ctrl (⌘ on a
 * Mac), and a row that does nothing else when pressed — a name in a list —
 * opens on a plain click: it says so with `data-file-open="click"` (FEAT-150).
 *
 * A screen whose rows already have a menu of their own puts the same entry
 * in it with `openInFiles`.
 *
 * **At the line on screen** (FEAT-189). Every line a diff viewer draws is a
 * row too, with `data-file-line` the number Files has it at, so a line opens
 * where it is. A file opened from a list while a viewer shows it opens at the
 * viewer's first line in view (`lineInView`).
 */

import { goto } from '$app/navigation';
import type { DiffLine } from '$lib/types';

export interface FileTarget {
	path: string;
	/** Any revision git can read. Files' own revision when left out. */
	rev?: string;
	line?: number;
}

/** The address that opens `path` in Files. */
export function filesHref({ path, rev, line }: FileTarget): string {
	const query = new URLSearchParams({ path });
	if (rev) query.set('rev', rev);
	if (line !== undefined && line > 0) query.set('line', String(line));
	return `/files?${query}`;
}

export function openInFiles(target: FileTarget): Promise<void> {
	return goto(filesHref(target));
}

/**
 * The revision a commit's file is read at. A file the commit deleted is not
 * in it, so it is read as the commit's parent left it.
 */
export function revOfChange(commit: string, status: string): string {
	return status === 'deleted' ? `${commit}^` : commit;
}

/**
 * A pull request's file, as the review room has it: at the fetched head, or
 * where the head left the target when the pull request deletes it.
 */
export function pullFileAttrs(
	path: string,
	status: string,
	head: { head: string; mergeBase: string } | null,
	opening: FileOpening = {}
) {
	if (!head) return fileAttrs(path, null, 'Not fetched');
	return fileAttrs(path, status === 'deleted' ? head.mergeBase : head.head, undefined, opening);
}

/**
 * A working-copy file, as Files can read it: at the branch, since Files
 * reads commits and not the working copy. A file new since then is not in
 * one yet.
 */
export function workingFileAttrs(
	path: string,
	status: string,
	head: string | null,
	opening: FileOpening = {}
) {
	const fresh = status === 'added' || status === 'untracked';
	return fileAttrs(path, head, fresh ? 'Not in a commit yet' : undefined, opening);
}

/**
 * Each of a diff's lines by its number in one version of the file, the one
 * Files reads (FEAT-189). A line that version does not have — one added,
 * read in the old; one removed, read in the new — takes the number of the
 * line it comes before there.
 *
 * The lines run from the start of a file or of a hunk: a hunk that starts
 * without a line of context starts the file.
 */
export function numbersIn(lines: readonly DiffLine[], version: 'old' | 'new'): number[] {
	let last = 0;
	return lines.map((line) => {
		const own = line[version];
		if (own !== null) last = own;
		return own ?? last + 1;
	});
}

/** How a marked row opens, beyond the right-click menu (FEAT-150). */
export interface FileOpening {
	/** A plain click opens it: the row does nothing else when pressed. */
	click?: boolean;
	/** The line to open at, 1-based. */
	line?: number;
}

/**
 * The attributes that make a row openable in Files: spread onto the row.
 * Without a revision, `absent` says why there is nothing to read.
 */
export function fileAttrs(
	path: string,
	rev: string | null | undefined,
	absent?: string,
	opening: FileOpening = {}
) {
	return {
		'data-file-path': path,
		'data-file-rev': rev ?? undefined,
		'data-file-absent': absent || (rev ? undefined : 'Not in a commit'),
		'data-file-open': opening.click ? 'click' : undefined,
		'data-file-line': opening.line && opening.line > 0 ? String(opening.line) : undefined
	};
}

/** What a marked row asks to open, or null when it is not one or cannot be read. */
export function targetOf(row: HTMLElement): FileTarget | null {
	const path = row.dataset.filePath;
	if (!path || row.dataset.fileAbsent) return null;
	const line = Number(row.dataset.fileLine);
	return {
		path,
		rev: row.dataset.fileRev || undefined,
		line: Number.isInteger(line) && line > 0 ? line : undefined
	};
}

/**
 * The line a viewer on screen shows `target`'s file at: the first of its
 * marked lines in view at the same revision, or undefined when no viewer
 * has the file open. Lines inside `from`, the row being opened, are not a
 * viewer of it.
 */
export function lineInView(target: FileTarget, from?: Element): number | undefined {
	let clip: { top: number; bottom: number } | null = null;
	for (const row of document.querySelectorAll<HTMLElement>('[data-file-line]')) {
		if (row.dataset.filePath !== target.path || from?.contains(row)) continue;
		const at = targetOf(row);
		if (!at?.line || at.rev !== target.rev) continue;
		clip ??= clipOf(row);
		const rect = row.getBoundingClientRect();
		const middle = (rect.top + rect.bottom) / 2;
		if (middle >= clip.top && middle < clip.bottom) return at.line;
	}
	return undefined;
}

const CLIPS = new Set(['auto', 'scroll', 'hidden', 'clip']);

/** Where `row` can be seen: the window, cut by every scrolling box round it. */
function clipOf(row: Element): { top: number; bottom: number } {
	let top = 0;
	let bottom = window.innerHeight;
	for (let box = row.parentElement; box; box = box.parentElement) {
		if (!CLIPS.has(getComputedStyle(box).overflowY)) continue;
		const rect = box.getBoundingClientRect();
		top = Math.max(top, rect.top);
		bottom = Math.min(bottom, rect.bottom);
	}
	return { top, bottom };
}
