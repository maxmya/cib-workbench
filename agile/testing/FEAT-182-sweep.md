<!-- SPDX-License-Identifier: MIT -->

# FEAT-182 — Manual sweep

**Item:** [`agile/items/FEAT-182-follow-repositories-on-cib-gitlab.md`](../items/FEAT-182-follow-repositories-on-cib-gitlab.md)

| Ticket | Preconditions | Steps | Expected result | Priority | Pass/Fail |
| --- | --- | --- | --- | --- | --- |
| SWEEP-001 | Connected, member of more than 50 projects | Onboarding step 4 | Most active first; *More* reads the next page. | High | |
| SWEEP-002 | SWEEP-001 | Type a search | The list reads once the typing stops. | Medium | |
| SWEEP-003 | Projects listed once | Leave and come back | The list paints at once, then refreshes. | Medium | |
| SWEEP-004 | A followed project | Rename it on GitLab, then open Settings › Home | Still followed, with its new path. | High | |
| SWEEP-005 | Follow nothing | Open Workbench | Home's empty state. | Medium | |
| SWEEP-006 | Following projects, a repository loaded | Look in its `.spagitty/` | Nothing about followed projects. | Medium | |
| SWEEP-007 | 50 followed | Try a 51st | The row cannot be pressed; the count says at most 50. | Low | |
