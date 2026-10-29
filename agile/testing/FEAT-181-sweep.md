<!-- SPDX-License-Identifier: MIT -->

# FEAT-181 — Manual sweep

**Item:** [`agile/items/FEAT-181-cib-gitlab-built-in-and-the-token-in-the-keychain.md`](../items/FEAT-181-cib-gitlab-built-in-and-the-token-in-the-keychain.md)

On CIB's network, signed in to CIB GitLab in the browser.

| Ticket | Preconditions | Steps | Expected result | Priority | Pass/Fail |
| --- | --- | --- | --- | --- | --- |
| SWEEP-001 | Onboarding step 2 | *Open token page in your browser* | The browser opens CIB GitLab's Access Tokens page (16.6.2: the form is behind *Add new token*). | High | |
| SWEEP-002 | SWEEP-001 | Follow the guide; copy the token | The numbered steps match the page. | High | |
| SWEEP-003 | Step 2 | *Copy link*, *Copy scopes* | The URL and `api read_repository write_repository` are on the clipboard. | Low | |
| SWEEP-004 | Step 3 | Paste a token with a character missing; Check | "CIB GitLab refused this token…"; nothing in Credential Manager. | High | |
| SWEEP-005 | Step 3 | A token with `api` and `read_repository` only | It names `write_repository`; nothing saved. | High | |
| SWEEP-006 | Step 3, off the VPN | Check | It says GitLab could not be reached and the VPN may be needed. | Medium | |
| SWEEP-007 | Step 3, a locked keychain (Linux, Secret Service locked) | Check | It says the keychain refused; nothing saved. | Medium | |
| SWEEP-008 | A valid token | Check | Windows Credential Manager holds one `com.cibeg.workbench` entry; `settings.json`, `workbench.json` and `accounts.json` have no token. | High | |
| SWEEP-009 | Connected | Settings › GitLab | Address, name and @login, scopes, expiry. A token in its last 14 days shows the expiry as a warning. | Medium | |
| SWEEP-010 | Connected | *Replace token* with a new one, then *Disconnect* | Replaced in place; Disconnect removes the Credential Manager entry, then the account. | High | |
| SWEEP-011 | Settings › You | Look | No generic accounts section in Workbench. | Low | |
