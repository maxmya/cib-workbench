<!-- SPDX-License-Identifier: MIT -->

# FEAT-189 — The whole file on Commit, and Files at the line on screen

**Status:** Open — in review on `feature/FEAT-189-whole-file-and-files-at-the-line`, for Workbench 2.0.3.
**Screens:** Commit; Diff, Stash and Review (lines); every file list (opening).
**Raised by:** the author, 2026-10-29: "in working copy commit screen i shall be able to expand the whole file, also when i double click a file anywhere it opens in files as we want, but if this file is opened on a viewer (commit screen or elsewhere), it should open files on the same line".

## Change

### The whole file on Commit

- *changes* and *whole file* chips in Commit's header, as *unified* and *split* are on Diff. The choice is kept across files and restarts (`spagitty.changesWhole`).
- Whole file keeps the hunks as they are, with their stage, unstage and discard buttons. The unchanged lines between them, and before the first and after the last, are filled in from the file read whole.
- `working_file_lines` (a reader) reads one working-copy file whole on either side of the index. It runs the same diff as the hunks, so a line between two hunks is never one a hunk holds. A file that cannot be read whole still shows its hunks.

### Files at the line on screen

- Every line a diff viewer draws is a file row (`data-file-path`, `data-file-rev`, `data-file-line`). A double-click, or a click with Ctrl or ⌘, opens Files at that line. This covers Commit, Diff, Stash and Review.
- The line is the number in the version Files reads. On Diff and Stash that is the commit's version, or its parent's for a deleted file. On Review it is the head's, or the merge base's for a deleted file. On Commit it is the old side, because Files reads the branch. A line missing from that version takes the number of the line after it.
- A file opened from a list while a viewer shows it at the same revision opens at the viewer's first line in view (`lineInView`). This applies to a double-click, a modified click and the **Open in Files** menu.
- The Diff and Stash file lists now open in Files on a double-click, like every other list.
- Review's `+` comment button ignores a double-click, so writing a comment does not leave the room.

## Acceptance criteria

1. On Commit, *whole file* shows every line of the selected file, with the hunks in place and their buttons working. *changes* shows the hunks alone.
2. Double-clicking a diff line on Commit, Diff, Stash or Review opens Files at that line.
3. Double-clicking a file in a list while its diff is on screen opens Files at the diff's top line. With no diff of it on screen, Files opens at the top.
