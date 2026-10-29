<!-- SPDX-License-Identifier: MIT -->

# FEAT-181 — CIB GitLab built in, and the token kept in the keychain

**Status:** Done — on `feature/workbench-2.0.0`, as Workbench 2.0.0, from the proposal's § 6. The manual sweep is still to run.
**Branch:** `feature/workbench-2.0.0`, with the other four of FEAT-180 to FEAT-184.
**Screens:** 2D steps 2–3, Settings › GitLab (new).
**Raised by:** the author, 2026-10-28: "save our company GitLab link and allow them to open the token export URL from the app; then they paste the token and we tell them it's in a secured store."

## Change

- `GITLAB = { host, tokenPath, tokenName, scopes }` in `brand.ts` and `brand.rs`, checked equal by `tools/workbench-brand.test.ts`. The values:
  - the host is `gitlab.apps.ocp-nonprod-01.hodomain.local`, GitLab EE 16.6.2;
  - the token page is `/-/profile/personal_access_tokens`, to be confirmed signed in before this merges (proposal § 6.1);
  - the scopes are `api`, `read_repository` and `write_repository`.
- `FEATURES.workbenchWide` in both files. `brand.rs` gains its first `FEATURES`.
- One function builds the token page URL: `https://<host><tokenPath>?name=Workbench&scopes=…`.
- *Open token page in your browser* is an `<a target="_blank">` through `safeUrl`, as *Open on host ↗* is in Issues and Releases; no new command or crate. *Copy link* is next to it.
- The token is checked and stored through a Workbench-only `gitlab_connect(token)`. It reads `token_info`, refuses a missing scope by name, and only then stores through the path `forge_connect` uses. `forge_connect` itself is not changed.
- `gitlab::token_info` reads `/personal_access_tokens/self` and `/user` and returns the name, login, scopes and expiry, never the token. A 404 on an older GitLab falls back to the login only.
- Settings › GitLab shows the address (read-only), who is signed in, the scopes, the expiry (with a warning in its last 14 days), *Replace token*, *Open token page* and *Disconnect*. In Workbench it stands in for the generic forge-accounts section.

## Acceptance criteria

1. The token page URL is exactly as specified (unit test).
2. A valid token is written to the keychain once and the result contains no token. 401, a missing required scope, an unreachable host and a keychain refusal each write nothing and say which of the four happened.
3. The webview clears the token field after a successful check, and no store keeps it.
4. Settings › GitLab *Disconnect* removes the keychain entry before the account (as `forge_disconnect` does).
5. With `FEATURES.workbenchWide` off, the generic accounts section is unchanged, and `forge_connect`'s tests pass as they are.
6. Signed in to CIB's GitLab, the token page link opens the token form with the name and scopes filled in.

## As built

- The token page path, `/-/profile/personal_access_tokens`, was confirmed signed in on 2026-10-28; `/-/user_settings/personal_access_tokens` is a 404 on 16.6.2.
- **Criterion 6 does not hold on 16.6.2, by the author's decision.** The form is behind *Add new token* and GitLab does not fill in the name or scopes from the query. The author decided on 2026-10-28 that guiding the person is enough, so onboarding step 2 carries a numbered guide and *Copy scopes*. The query string stays in the link for a GitLab that honours it.
- The checks live in `crates/spagitty-core/src/workbench.rs` rather than in `forge/gitlab.rs`, so that a Spagitty merge never meets them.
- `gitlab_connect` disconnects any other account on the built-in host, keychain first: one GitLab, one person.
- Plan: [`FEAT-181-plan.md`](../plans/FEAT-181-plan.md). Tests: [`FEAT-181-automated.md`](../testing/FEAT-181-automated.md). Sweep: [`FEAT-181-sweep.md`](../testing/FEAT-181-sweep.md).
