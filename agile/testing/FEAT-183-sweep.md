<!-- SPDX-License-Identifier: MIT -->

# FEAT-183 — Manual sweep

**Item:** [`agile/items/FEAT-183-home-across-followed-repositories.md`](../items/FEAT-183-home-across-followed-repositories.md)

| Ticket | Preconditions | Steps | Expected result | Priority | Pass/Fail |
| --- | --- | --- | --- | --- | --- |
| SWEEP-001 | Three projects followed, nothing loaded | Open Home | Needs you, Merge requests, Pipelines, Issues, Calls and Releases fill; each row names its project. | High | |
| SWEEP-002 | A review requested in a project not followed | Home | It is in Needs you. | High | |
| SWEEP-003 | One followed project you cannot read | Home | Its error in its card; the others show. | High | |
| SWEEP-004 | A followed project loaded | *<repo> only* | Every card shows that project only. | Medium | |
| SWEEP-005 | Order set in onboarding | Home | The cards are in that order. *Customise Home* changes it. | Medium | |
| SWEEP-006 | Nothing loaded | A merge request row | Load repository opens with its project; after loading, Pull requests. | High | |
| SWEEP-007 | Home shown twice within the notification interval | Watch the requests | The second read takes projects read within the interval from the last answer; Refresh reads all. | Low | |
| SWEEP-008 | A repository loaded | Home | The Loaded band: Working copy, Branch with Push, Recent commits. | Medium | |
| SWEEP-009 | Workbench-wide Calls collections | Home's Calls card | The three used last; a row opens Calls. | Low | |
