<!-- SPDX-License-Identifier: MIT -->

# FEAT-184 — Manual sweep

**Item:** [`agile/items/FEAT-184-load-repository-instead-of-tabs.md`](../items/FEAT-184-load-repository-instead-of-tabs.md)

| Ticket | Preconditions | Steps | Expected result | Priority | Pass/Fail |
| --- | --- | --- | --- | --- | --- |
| SWEEP-001 | A 1.2 profile with five tabs open | Upgrade to 2.0.0 and start | No tab strip; Home; the last active repository is loaded. | High | |
| SWEEP-002 | SWEEP-001 | Rebuild with `FEATURES.workbenchWide` off and start | The five tabs are back. | High | |
| SWEEP-003 | Nothing loaded | Ctrl+O from Home, Calls and Settings | The dialog opens; Esc closes it; focus returns. | High | |
| SWEEP-004 | The dialog | Type, ↑/↓, Enter | A local repository loads; the title bar shows it with branch and ↑/↓. | High | |
| SWEEP-005 | A followed project not cloned | *Clone and load* | Progress in the row; then loaded, and in recents. | High | |
| SWEEP-006 | A project already cloned | Find it | Under *On this machine* only. | Medium | |
| SWEEP-007 | A loaded, on Branches with a commit selected in Graph | Load B, then A again | A's Branches; Graph keeps the commit. | High | |
| SWEEP-008 | Loaded, on Working copy | *Unload* | Nothing loaded; Home; the rail's repository rows dim. | High | |
| SWEEP-009 | Nothing loaded | Tab to a dimmed rail row, Enter | Its title says why; the dialog opens; after loading, that screen. | Medium | |
| SWEEP-010 | Nothing loaded | Back to a repository screen in history | Home with the dialog over it. | Medium | |
| SWEEP-011 | Settings › Behaviour › *Open with nothing loaded* on | Restart | Home, nothing loaded. | Medium | |
