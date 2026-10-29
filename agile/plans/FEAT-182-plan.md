<!-- SPDX-License-Identifier: MIT -->

# FEAT-182 — Plan

**Item:** [`agile/items/FEAT-182-follow-repositories-on-cib-gitlab.md`](../items/FEAT-182-follow-repositories-on-cib-gitlab.md)
**Branch:** `feature/workbench-2.0.0`, as 2.0.0.
**Status:** implemented.

## Approach

### Where the list is kept

`workbench.json`, beside `settings.json`, not in `settings.rs` as the proposal
said. The Settings screen writes its whole settings object back on every
toggle; a copy read before onboarding finished would write the old `followed`
over the new one. A file of its own also keeps a Spagitty merge from meeting
it. `WorkbenchState { onboarded, followed, cloneInto }`; a missing or broken
file is the defaults.

`Followed { host, id, path, defaultBranch, httpUrl }`, keyed by id. The Tauri
layer forces the host to `brand::gitlab::HOST`, drops repeats and caps it at
50 (`MAX_FOLLOWED`).

### Listing

`spagitty_core::workbench::projects` reads
`/projects?membership=true&archived=false&simple=true&order_by=last_activity_at&per_page=50&page=N[&search=…]`;
`more` comes from the `Link`/`X-Next-Page` headers through
`get_conditional_json`. `gitlab_projects` saves each page under the app data
folder (`forge_views::save`), `gitlab_projects_cached` reads it back, and the
webview's `ProjectList` paints the saved page, then GitLab's. Search waits
250 ms for the typing to stop.

Every listing refreshes the followed entries it contains (`refreshed`), so a
renamed or moved project keeps its id and shows its new path.

### Where it is edited

`FollowList.svelte`, used by onboarding step 4 (held until *Open Workbench*)
and by Settings › Home under the switch (`WideHomeSection.svelte`, written as
it changes). Home's *Change* goes to Settings › Home.
