<!-- SPDX-License-Identifier: MIT -->

# FEAT-181 — CIB GitLab built in, and the token kept in the keychain

**Status:** Backlog — proposed in `docs/proposals/workbench-wide-home-and-load-repository.md` § 6.
**Branch:** none yet.
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
