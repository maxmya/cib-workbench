<!-- SPDX-License-Identifier: MIT -->

# FEAT-180 — Manual sweep

**Item:** [`agile/items/FEAT-180-first-launch-onboarding.md`](../items/FEAT-180-first-launch-onboarding.md)

A fresh profile on Windows: no `workbench.json`, no accounts.

| Ticket | Preconditions | Steps | Expected result | Priority | Pass/Fail |
| --- | --- | --- | --- | --- | --- |
| SWEEP-001 | Fresh profile | Start Workbench | The launch screen gives way to onboarding: no rail, no status strip, only the window buttons; the steps read Welcome · GitLab · Token · Home. | High | |
| SWEEP-002 | SWEEP-001 | Next, Back and the dots on Welcome; the arrow keys | All three pages; Back is hidden on the first. | Medium | |
| SWEEP-003 | Step 3, off the VPN | Press *Set up later* | Home, empty and not connected. Restart: onboarding again. | High | |
| SWEEP-004 | Step 3 | Paste a valid token, Check | The account shows; Next is enabled; the field is empty. | High | |
| SWEEP-005 | Step 4 | Follow three projects, move *Issues* to the top, switch *Releases* off, then quit before *Open Workbench* | Nothing is kept: the next launch starts onboarding with none followed. | Medium | |
| SWEEP-006 | Step 4 | Choose again and press *Open Workbench* | Home follows the three, Issues first, no Releases. Restart: Home, no onboarding. | High | |
| SWEEP-007 | Onboarded, connected | Settings › Behaviour › *Run setup again* | Onboarding; "Connected as @login"; Welcome goes straight to Home. | Medium | |
| SWEEP-008 | Fresh profile | Start with a repository path on the command line | Onboarding first; the repository is loaded after *Open Workbench*. | Medium | |
| SWEEP-009 | 1.x profile with an account for CIB GitLab | Upgrade and start | Onboarding passes over GitLab and Token. | Medium | |
