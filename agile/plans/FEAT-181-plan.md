<!-- SPDX-License-Identifier: MIT -->

# FEAT-181 — Plan

**Item:** [`agile/items/FEAT-181-cib-gitlab-built-in-and-the-token-in-the-keychain.md`](../items/FEAT-181-cib-gitlab-built-in-and-the-token-in-the-keychain.md)
**Branch:** `feature/workbench-2.0.0`, with FEAT-180, FEAT-182, FEAT-183 and FEAT-184, as 2.0.0.
**Status:** implemented.

## Approach

### The brand seam

`GITLAB` and `tokenPageUrl()` in `src/lib/brand.ts`; `brand::gitlab` and
`brand::features::WORKBENCH_WIDE` in `crates/spagitty-core/src/brand.rs`.
`tools/workbench-brand.test.ts` reads the Rust constants and compares them,
and pins the exact token page URL. `FEATURES`' comment now covers a switch
that adds something.

The token path is `/-/profile/personal_access_tokens`, which the author opened
signed in on 2026-10-28; `/-/user_settings/…` is a 404 on 16.6.2. GitLab
16.6.2 keeps the create form behind *Add new token* and does not prefill it,
so the query string is kept for a later GitLab and onboarding carries a
numbered guide (author's decision: "it doesn't matter if nothing is
prefilled, as long as the app guides the person").

### One Workbench-only core file

`crates/spagitty-core/src/workbench.rs` holds `token_info`, `connect` and
`missing_scopes`. Every request is a `Fetch` closure, so the tests answer for
GitLab; the real one is `fetcher`, over `forge::http::get_conditional_json`
(the file does not name the HTTP client, which `requests.test.ts` checks).

- `token_info` reads `GET /user`, then `GET /personal_access_tokens/self`.
  A 403/404 on the second is an older GitLab: the login only,
  `scopesKnown: false`. A 403 `insufficient_scope` on `/user` is the `api`
  scope missing.
- `connect` refuses an empty token, reads `token_info`, refuses a missing
  scope by name, and only then calls `keep(login)` — once.
- `ConnectFailure` is tagged (`unauthorized`, `unreachable`, `missingScope`,
  `keychain`, `other`), so the webview names which of the four happened.

### Tauri

`src-tauri/src/workbench.rs`: `gitlab_connect(token)` runs `connect` off the
main thread with `keychain::store` as `keep`, then remembers the account as
`forge_connect` does. Another account on the built-in host is disconnected,
keychain first. `gitlab_token_info()` reads the stored token. `forge_connect`
is not touched.

### Webview

`src/lib/workbench/`: `gitlab.ts` (sentences, expiry, keychain names),
`state.svelte.ts` (`connect` keeps the answer only), `TokenPage.svelte` (an
`<a target="_blank">` through `safeUrl`, *Copy link*, the URL, the scopes),
`TokenField.svelte` (clears itself after a passed check), and
`GitLabSection.svelte` for Settings › GitLab. Under the switch Settings › You
no longer draws the generic accounts section.

## Order

1. Brand seam and its test.
2. Core `workbench.rs` with tests.
3. Tauri commands.
4. Webview pieces and Settings › GitLab.

## Risks

- The token page link is opened by the same anchor the app uses for host
  links. If a platform's webview stops sending `_blank` to the browser, every
  host link breaks with it.
- A token without `read_user` and with `api` works; one with only
  `read_repository` is refused by `/user` with `insufficient_scope`, named as
  `api` missing.
