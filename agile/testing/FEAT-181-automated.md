<!-- SPDX-License-Identifier: MIT -->

# FEAT-181 — Automated tests

**Item:** [`agile/items/FEAT-181-cib-gitlab-built-in-and-the-token-in-the-keychain.md`](../items/FEAT-181-cib-gitlab-built-in-and-the-token-in-the-keychain.md)

| Where | Asserts |
| --- | --- |
| `tools/workbench-brand.test.ts` | `GITLAB` and `brand::gitlab` hold the same host, token path, token name and scopes, and the switch agrees. The token page URL is exactly `https://gitlab.apps.ocp-nonprod-01.hodomain.local/-/profile/personal_access_tokens?name=Workbench&scopes=api,read_repository,write_repository`. |
| `crates/spagitty-core/src/workbench.rs` | With a simulated GitLab: a good token is kept once and the answer carries no token; 401 keeps nothing and is `unauthorized`; a missing `write_repository` keeps nothing and names it; a 403 `insufficient_scope` on `/user` is `api` missing; an unreachable host keeps nothing and is `unreachable`; a keychain refusal is `keychain`; a GitLab without `personal_access_tokens/self` gives the login only; the failure is tagged for the webview. |
| `src/lib/workbench/gitlab.test.ts` | The sentence for each of the four failures; a rejected command read as a failure; the secure store named per platform; the expiry warning in the last 14 days and after. |
| `src/lib/workbench/onboarding-screen.test.ts` | The token field is cleared after a passed check; a refused token is named and nothing goes on. |
| `src/lib/requests/requests.test.ts` | Still three core files name the HTTP client: `workbench.rs` goes through `forge::http`. |
| `src/lib/settings/sections.test.ts` | Unchanged and passing: the generic accounts section, with the switch as it is. |

**What no test here can see:** the keychain itself, and CIB's GitLab. Those
are the sweep.
