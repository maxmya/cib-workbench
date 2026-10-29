<!-- SPDX-License-Identifier: MIT -->

# Proposal: Onboarding, a Workbench-wide Home, and Load repository instead of tabs

**Prepared:** 2026-10-28  
**Target:** Workbench 2.0.0, set by the author on 2026-10-28. The tab strip going is a change in how the application is used, so it is a major version.  
**Status:** FEAT-180 to FEAT-184 built on `feature/workbench-2.0.0` as 2.0.0, 2026-10-29; FEAT-185 is backlog. Where the build differs from this proposal, each item's *As built* says so. The author's answers to the questions are in [section 15](#15-questions-for-the-author).  
**Raised by:** the author, 2026-10-28: "now this software is repo oriented, but a lot of features should not be repo oriented … 1 – when one installs it the first time, they get onboarded to a screen with Next buttons that explains what this software is. 2 – save our company GitLab link and allow them to open the token export URL from the app. 3 – then they paste the token and we tell them it's in a secured store and press Next. 4 – allow them to order the Home screen, which is renewed to fetch [selected repos or empty]. 5 – LOAD A REPO FLOW — instead of multiple tabs, a load-repo function that makes the app load a specific repo to act upon, or work generally with calls and pipelines of selected repos."  
**Screens:** 2D Onboarding (new), 2A Home (changed), 2E Load repository (new, a dialog), 1K Settings › GitLab (new) and Settings › Home (changed), chrome (title bar, rail; the tab strip goes).  
**Designs:** [`design_handoff_onboarding_home/`](../../design_handoff_onboarding_home/), listed in [section 14](#14-designs).

---

## 1. Outcome

Workbench stops being a window onto one repository and becomes a window onto the person's work at CIB GitLab, which can also work on one repository when they load it.

- **First launch** walks the person through four steps: what Workbench is (three short pages), connecting to CIB GitLab (the address is built in, and one button opens GitLab's token page with the name and scopes filled in), pasting the token (checked against GitLab, then kept in the OS keychain), and setting up Home (which repositories to follow, and the order of Home's sections).
- **Home** reads every followed repository from GitLab, whether or not it is cloned: what needs you, merge requests, pipelines, issues assigned to you, releases, plus Workbench-wide Calls. Nothing has to be loaded for any of it.
- **Load repository** (Ctrl+O, or the title bar) replaces the tab strip. One repository is loaded at a time, from this machine or straight from GitLab (cloned first). Loading another replaces it. Repository tools (Graph, Working copy, Branches, Rebase, Conflicts…) act on the loaded one; with none loaded they wait on the rail, dimmed, and choosing one opens Load repository.

## 2. Rules that shape the rest

1. **Workbench stays mergeable with Spagitty.** `docs/workbench-sync.md` applies Spagitty's diffs onto Workbench again and again. Everything here is behind one switch in the brand seam, `FEATURES.workbenchWide` (`src/lib/brand.ts`, mirrored in `crates/spagitty-core/src/brand.rs`), with new code in new Workbench-only files and the smallest possible hooks into shared files. The Spagitty path, tabs included, still works with the switch off, and its tests still pass. Add a row for each new hook to the table in `docs/workbench-sync.md`.
2. **The backend still holds one repository.** `AppState.session` is unchanged. A tab was already only "a place to go back to" (`workspace.svelte.ts`); Load repository makes the interface say what the backend already does.
3. **The token goes to the OS keychain and nowhere else.** Use the existing `forge_connect` → `forge::keychain::store` path. The webview sends the token once and then forgets it: it is not kept in a store, logged, or echoed back.
4. **Every GitLab request is made in Rust**, through the existing forge HTTP client (`forge/http.rs`, which already trusts the corporate CA from BUG-042). The webview never holds an HTTP client or a token.
5. **One GitLab.** The host comes from the brand seam. The connect screen does not offer GitHub or Bitbucket in Workbench, and the token page link is built only from that host. This limits what this proposal adds; the links the app already opens (a host link in Issues, Releases and Pull requests, through `safeUrl`) are unchanged.
6. **Home adds no timer of its own.** It reads when shown, on Refresh, and on the clock the forge watcher (`forge/watch.rs`) already runs. Saved results paint first and are revalidated, as Issues and Releases already do (`forge/snapshot.rs`).
7. **No new brand copy without the author.** The designs' wording is proposed UI copy. Taglines and descriptions of the product stay the approved ones in `docs/branding.md` (the splash line is reused as it is). If the author wants a new sentence treated as brand copy, it goes into `docs/branding.md` first.
8. **No new dependency without saying why** (AGENTS.md). Opening a URL in the browser should reuse whatever the app already uses; see [6.3](#63-opening-the-token-page).
9. **Labels only where needed; key names in `Ctrl` form**, as everywhere else in the app.

## 3. Words used here

| Word | Means |
| --- | --- |
| **Followed repository** | A CIB GitLab project the person chose for Home. Kept in Workbench's settings by project id and `path_with_namespace`. It need not be cloned. |
| **Loaded repository** | The one repository the backend has open (`AppState.session`). Zero or one. |
| **Workbench-wide** | A screen or card that works with nothing loaded. Home, Calls (FEAT-170), Farm, Settings; later the forge lists ([FEAT-185](#13-backlog-items)). |
| **Repository-scoped** | A screen that needs a loaded repository: Graph, Diff, Working copy, Branches, Stash, Rebase, Conflicts, Merger, Review, Log, Reflog, Tags, Files, Terminal, Recipes, Hooks, Investigate, Untrack, Submodules, Worktrees. |
| **Onboarding** | The first-launch sequence, 2D. |

## 4. Screens, codes and the rail

| Code | Screen | Route | Rail | Item |
| --- | --- | --- | --- | --- |
| 2D | Onboarding | `/welcome` | no (full window, no chrome) | FEAT-180 |
| 2A | Home (changed) | `/home` | yes, top | FEAT-183 |
| 2E | Load repository | dialog, Ctrl+O | no | FEAT-184 |
| 1K | Settings › GitLab (new section) | `/settings#gitlab` | — | FEAT-181 |
| 1K | Settings › Home (followed repositories added) | `/settings#home` | — | FEAT-182 |

With `FEATURES.workbenchWide` on, the rail keeps its groups (FEAT-169). Repository-scoped rows draw dimmed with `aria-disabled="true"` when nothing is loaded. They stay focusable, their title says "<Screen> — load a repository first", and activating one opens 2E and goes to that screen once the load finishes. Calls, Farm, Home and Settings are never dimmed.

## 5. Onboarding (2D) — FEAT-180

### 5.1 When it shows

- On launch, when the setting `onboarded` (new, `src-tauri/src/settings.rs`, default `None`) is unset, **and** `FEATURES.workbenchWide` is on.
- **Skip steps 2–3** when an account for the brand's GitLab host is already connected (`forge_accounts`). That covers people upgrading from 1.x who already connected one. Show a line instead: "Connected as @login".
- When it finishes, write `onboarded = { version: 1, at: <RFC 3339> }`. The version lets a later build show only the new steps.
- Steps 2 and 3 carry *Set up later*. Someone off the VPN, or with a locked keychain, must still be able to get in. It goes to the empty Home, leaves `onboarded` unset so the next launch offers setup again, and Settings › GitLab shows "Not connected" with *Connect*.
- Settings › Behaviour gets *Run setup again*, which clears `onboarded` and goes to `/welcome`.
- A repository path given on the command line still loads. Onboarding runs first, and the load happens after it finishes.

### 5.2 The shell

Full window, no rail, no title bar tools: only the window buttons and drag area stay (the window is undecorated, `TitleBar.svelte`). Then:

- At the top, "CIB" in spaced small capitals, the wordmark, and the four steps (*Welcome · GitLab · Token · Home*): done in green with a tick, current with a ring. This is the Farm wizard's `Steps` pattern from `design_handoff_cib_workbench`, recoloured to the corrected palette.
- At the bottom, *Back* (hidden on the first page), the page dots on Welcome only, and *Next*. Below a hairline, the CIB reversed logo (`BRAND_ASSETS.logoReversed`, picked by mode, never by a filter) and the approved notice (`NOTICE.badge`, `NOTICE.lead`).

### 5.3 Steps

1. **Welcome**, three pages with Next / Back / dots:
   - the large wordmark, `SPLASH_LINE`, and one paragraph on what Workbench does;
   - *Home follows your repositories*: Needs you, pipelines and merge requests, Calls;
   - *Load a repository when you need one*: Ctrl+O, repository tools act on it, switching replaces it.
2. **GitLab.** A read-only, labelled field shows the address from the brand seam with "Set in this build". Below it:
   - *Open token page in your browser*, which opens the URL from [6.2](#62-the-token-page-url);
   - *Copy link*;
   - the URL itself, in mono, so it can be read and checked;
   - the three scopes, each with what it is for.

   A side panel lists what to do on that page and warns: "GitLab shows the token only once." *Next* is labelled *I have my token*.
3. **Token.** A password field (show/hide), and *Check*. Check calls `forge_connect(kind: GitLab, host: GITLAB.host, token)`:
   - On success, show the name, @login, scopes and expiry ([6.4](#64-reading-the-token-back)). The secure-store panel changes from "It goes into your system's secure store" to "Saved to your system's secure store". *Next* enables.
   - On failure, *Next* stays disabled, and *Set up later* ([5.1](#51-when-it-shows)) is the way on. Unauthorized → "CIB GitLab refused this token. Check you copied all of it, and that it hasn't expired." Unreachable → say so, and that a VPN or proxy may be needed. Missing scope → name the scope. Keychain unavailable → say the keychain refused, and that nothing was saved.

   The panel names the store per platform: Windows Credential Manager, macOS Keychain, the Secret Service on Linux. It also says the token is never written to Workbench's settings or logs, is sent only to the GitLab host, and can be replaced in Settings › GitLab.
4. **Home.** Two cards:
   - **Repositories to follow.** Search, then the list from [7.2](#72-listing-projects), most active first. Each row is a toggle button (`aria-pressed`). *Start empty* clears the selection.
   - **Home sections.** The cards in order. Each row has a grip, its number, a name and a line of description, up/down buttons (`aria-label` "Move X up"), and a switch (`role="switch"`). This writes the existing `HomeLayout` (FEAT-148), so Settings › Home and *Customise Home* edit the same thing.

   *Open Workbench* finishes and goes to Home, which shows the empty state when nothing is followed.

## 6. CIB GitLab in the brand seam — FEAT-181

### 6.1 Values

`src/lib/brand.ts` and `crates/spagitty-core/src/brand.rs`, the same values in both, checked by `tools/workbench-brand.test.ts`:

```ts
export const GITLAB = {
  /** Lower case, no scheme. */
  host: 'gitlab.apps.ocp-nonprod-01.hodomain.local',
  /** CIB runs GitLab Enterprise Edition 16.6.2. */
  tokenPath: '/-/profile/personal_access_tokens',
  tokenName: 'Workbench',
  scopes: ['api', 'read_repository', 'write_repository']
};
export const FEATURES = { …, workbenchWide: true };
```

The host is the one the author's installed Workbench is connected to, and the author confirmed on 2026-10-28 that it is the one to build in. The GitLab is Enterprise Edition 16.6.2 (`/help`), so `personal_access_tokens/self` ([6.4](#64-reading-the-token-back)) exists. Newer GitLab releases moved the token page under `/-/user_settings/`. The author opened both while signed in on 2026-10-28: `/-/profile/personal_access_tokens` is the Access Tokens page and `/-/user_settings/personal_access_tokens` is a 404. On 16.6.2 the form is behind *Add new token* and is not filled in from the query, so onboarding guides the person through it (the author's decision).

`brand.rs` has no `FEATURES` today, only `CHECKS_FOR_UPDATES`. FEAT-181 adds the Rust mirror. The comment on `FEATURES` in `brand.ts` ("What Workbench leaves out of Spagitty") is reworded to cover a switch that adds something.

Scopes: `api` for merge requests, pipelines, issues and releases; `read_repository` to clone and fetch over HTTPS; `write_repository` to push. The author chose the full set (section 15). A read-only token would hide the write actions people expect.

### 6.2 The token page URL

`https://<host><tokenPath>?name=Workbench&scopes=api,read_repository,write_repository`. GitLab prefills the name and scopes from these query parameters. One pure function builds it (`brand.ts` or a new `src/lib/gitlab/token-page.ts`), with a test for the exact string.

### 6.3 Opening the token page

Use the path the app already has: an `<a target="_blank" rel="noopener noreferrer">` whose `href` goes through `safeUrl` (`src/lib/ui/markdown.ts`), as *Open on host ↗* does in Issues and Releases. The `href` is the pure function from [6.2](#62-the-token-page-url), built from `GITLAB.host`, so no new command and no crate are needed. *Copy link* uses the existing clipboard path, as Releases' *Copy link* does.

### 6.4 Reading the token back

`forge_connect` already proves the token and reads the login (`identify_account` → `gitlab::whoami`). Add `gitlab::token_info(host, token)`, which reads `GET /api/v4/personal_access_tokens/self` (scopes, `expires_at`; GitLab ≥ 15.5) and `GET /api/v4/user` (`name`, `username`, `avatar_url`). Return it as `TokenInfo { name, login, scopes, expires_at }`; it never contains the token. If the endpoint returns 404 on an older GitLab, show the login only.

`forge_connect` is Spagitty's, so do not put the scope check in it. Add a Workbench-only command, `gitlab_connect(token)`, in a new file. It calls `token_info` first. If a needed scope is missing, it fails with a sentence that names the scope and stores nothing. Otherwise it calls the same path `forge_connect` uses. Onboarding and Settings › GitLab call `gitlab_connect`; the generic accounts section keeps calling `forge_connect`.

### 6.5 Settings › GitLab

A new section, Workbench only. It shows:

- the address (read-only);
- "Signed in as Name @login";
- the scopes;
- the expiry, with a warning in its last 14 days;
- *Replace token* (the Token step inline), *Open token page*, and *Disconnect* (`forge_disconnect`, which clears the keychain first, as it does today).

In Workbench this section stands in for the generic forge-accounts section. The generic one stays in the code for the Spagitty path.

## 7. Followed repositories — FEAT-182

### 7.1 Model

In Workbench's settings (`settings.rs`): `followed: Vec<Followed>`, where `Followed { host, id: u64, path: String }`. Keyed by GitLab project id, so a rename or move does not lose it; `path` is refreshed from the last listing. The list is never written into any repository.

### 7.2 Listing projects

A new Rust command, `gitlab_projects(search: Option<String>, page: u32)`, calls `GET /api/v4/projects?membership=true&archived=false&simple=true&order_by=last_activity_at&per_page=50[&search=…]`. It returns `{ id, path_with_namespace, description, last_activity_at, default_branch, http_url_to_repo }`. Follow the `Link`/`X-Next-Page` header for paging. Search is debounced in the webview and runs in Rust. Results are cached through `forge/snapshot.rs`, so the step paints immediately on a second visit.

### 7.3 Where it is edited

Onboarding step 4, Settings › Home (the same list with search), and Home's *Change* link next to "Following N repositories".

## 8. Home across followed repositories — FEAT-183

### 8.1 What the cards read

| Card (id) | Source | Needs a loaded repository |
| --- | --- | --- |
| Needs you (`needs`) | The forge watcher's account-wide read (`watch::watch(GitLab, host, token, me)`, which already covers *review requested*, *changes requested* and *assigned*), plus failed pipelines from the Pipelines read and issues assigned to you. Kinds and colours stay in `notifications/kinds.ts`. | no |
| Merge requests (`requests`, relabelled in Workbench) | `gitlab::involved_merge_requests(host, token, me)` (host-wide already), filtered to followed projects | no |
| Pipelines (`pipelines`) | For each followed project, the latest run on the default branch: `GET /projects/:id/pipelines?ref=<default_branch>&per_page=1`, a lighter read than `forge::pipelines::page`, which Pipelines (1Y) keeps for its full list. Show it, plus runs on the person's own open MR branches. A namespace with subgroups (`group/sub/name`) is already one encoded segment (`Repo::encoded_slug`). | no |
| Issues (`issues`, new) | `GET /api/v4/issues?scope=assigned_to_me&state=opened&per_page=20`, filtered to followed projects | no |
| Calls (`calls`, new) | The Workbench-wide collections store (FEAT-170); the three most recently used | no |
| Releases (`releases`) | For each followed project, `GET /projects/:id/releases?per_page=1` | no |
| Working copy, Branches, Recent commits, Commits last 16 weeks, Farm | `home_overview` and the existing cards, unchanged | **yes**: shown in the *Loaded* band |

- Fan-out is capped at 4 requests in flight. A project that fails shows its own error line in its card and never blanks the whole card.
- The cap limits how many requests run at once, not how many there are. Pipelines and Releases are each one request per followed project, so following 30 projects means about 60 requests every time Home is read. Saved results paint first (rule 6), and a project's results read within the forge watcher's last interval are not read again. Following is capped at 50 projects; the list says so when the cap is reached.
- Every row carries its project as a mono chip. A row opens the existing screen. If that screen is repository-scoped and the project is not loaded, 2E opens with the project selected.

### 8.2 Layout

- **Header:** "Home", "Following N repositories on CIB GitLab · updated <when> · *Change*", and *Customise Home*. With a repository loaded, add a segmented control: *All followed* | *<repo> only*. It is session state, not stored, and filters every Workbench-wide card to that project.
- **Loaded band**, only when something is loaded: the project path, *Open graph*, *Commit…*, then Working copy, Branch (ahead/behind against upstream, last fetched, *Push*) and Latest commits. Its cards come from the existing card components.
- **The cards**, in `HomeLayout` order. Needs you and Releases are full width; the others share rows. They stack at phone width.
- **Empty state**, when nothing is followed: "You're not following any repositories", with *Choose repositories* and *Load a repository*. The Calls card still shows below it.

### 8.3 Card ids

Extend `HomeCardId` with `issues` and `calls`. `inHomeOrder` already places unknown and new ids sensibly. Under `FEATURES.workbenchWide`, the repository cards (`working`, `branches`, `commits`, `heat`, `farm`) move into the Loaded band and drop out of the ordered list, so the order the person sets in onboarding is the order of Workbench-wide cards only.

## 9. Load repository instead of tabs — FEAT-184

### 9.1 The model

- `workspace.svelte.ts` keeps `places` (route and selected commit per path) exactly as it is. With the switch on, `tabs` is not shown and `MAX_TABS` stops mattering: recents come from the backend's `recent_repos` (`recents.rs`), as All repositories already does.
- `active` becomes "the loaded repository". `workspace.rest()` (FEAT-172) is already "nothing shown".
- `RepoTabs.svelte` is not rendered when `FEATURES.workbenchWide` is on. Keep the component for the Spagitty path; do not delete it.

### 9.2 The title bar slot

- **Nothing loaded:** a dashed button reading "No repository loaded · **Load repository** `Ctrl+O`".
- **Loaded:** a chip with the project path (mono), the branch, and ↑ahead ↓behind, then *Switch* (opens 2E) and an icon button *Unload <name>*.

This replaces the repository picker in `Toolbar.svelte` / `TitleBar.svelte` under the switch. The rest of the toolbar (Fetch, Push, Commit…) is enabled only while something is loaded.

### 9.3 The dialog (2E)

- A modal: `role="dialog"`, `aria-modal`, focus trapped, Esc closes, ↑/↓ move, Enter loads. These are handlers on the dialog, not global keydown handlers.
- Ctrl+O is registered where the app's other shortcuts and the command palette commands are registered (`src/lib/palette/commands.ts`).
- One search field: "Find a repository on this machine or on CIB GitLab". Two groups:
  - **On this machine:** `recent_repos`, with path, branch and when last loaded. A GitLab project that is already cloned (matched by the FEAT-087 "which recent repository is a clone of slug on host" command) shows here, not twice.
  - **On CIB GitLab:** followed projects first (marked *Followed*), then search results from `gitlab_projects`. Action: *Clone and load*, with "Clones into <folder>\<name>". The folder comes from the clone screen's existing default.
- Footer: *Open a folder…* (the existing native picker) and "Loading replaces the repository you have loaded. Home keeps what you follow."

### 9.4 Loading, replacing, unloading

- **Load** = `workspace.remember(current, place)` for the one being left, then the existing `repo.open(path)` (which calls `open_repo` and `recents::remember`), then restore that path's place. The row shows a progress line while it runs. A large history fills progressively, as it does today.
- **Clone and load** = the existing clone flow (`clone/store.svelte.ts`, `clone::plan`) with its progress shown in the row, then Load.
- **Unload** = `close_repo`, then `workspace.rest()`. The window goes to Home if it was on a repository-scoped screen.
- A repository-scoped route reached with nothing loaded (a deep link, the back button) opens 2E over Home rather than drawing an empty screen.

### 9.5 Launch

`session.ts`'s `resumeSession` order stays: command line, then FEAT-172's setting, then the last session. Under the switch:

- the window always lands on Home;
- the last loaded repository is reloaded unless *Open with nothing loaded* is on (FEAT-172's setting, reworded under the switch);
- onboarding ([5.1](#51-when-it-shows)) runs before any of it.

### 9.6 Migration

The stored `spagitty.workspace` is read as it is: `places` survive, `active` becomes the repository to reload, and `tabs` is ignored without being deleted, so turning the switch off brings the strip back. Nothing is lost on upgrade.

## 10. The network boundary

Every new request goes through `forge/http.rs` to `GITLAB.host` only. The commands added are:

- `gitlab_connect`, which checks the scopes and then stores the token ([6.4](#64-reading-the-token-back));
- `gitlab_projects`;
- `gitlab_token_info`, for Settings › GitLab, reading the stored token;
- `home_workbench_wide`, a single read that returns every Workbench-wide card's rows plus per-project errors, so the webview makes one call.

None of them takes a token from the webview except `gitlab_connect`. None of them logs a token or a URL that contains one. Tests assert that `TokenInfo` and every response type have no field that could carry the token.

## 11. Accessibility and the designs' rules

- Real controls throughout: toggle buttons with `aria-pressed`, `role="switch"` buttons, labelled inputs. Dimmed rail rows use `aria-disabled` and stay focusable.
- Contrast: muted text is 64–68% ink on `#111214`; the accent text is `#4D8FD6`; primary buttons are `#0C529A` with white text. All pass 4.5:1.
- Status lines use `role="status"`. The token result does not depend on colour alone: it has a text sentence and an icon.

## 12. Testing

Automated (vitest and `cargo test`):

- The token page URL builder (exact string). `GITLAB` values match between `brand.ts` and `brand.rs`.
- `gitlab_connect` against a mocked GitLab: success writes the keychain once and returns no token; 401, a missing scope and a keychain error each write nothing and produce the right sentence. `forge_connect` is unchanged and its tests pass as they are.
- Onboarding: *Set up later* reaches the empty Home and leaves `onboarded` unset.
- `token_info` on 15.5+ and on a 404 (older GitLab).
- `gitlab_projects` paging and search; `Followed` survives a rename (same id, new path).
- `home_workbench_wide`: fan-out cap, a per-project error, filtering to followed, the *<repo> only* filter.
- `HomeLayout` with the new ids; under the switch, repository cards leave the ordered list.
- `resumeSession` under the switch: onboarding first; reload the last loaded repository; FEAT-172 on → nothing loaded; a command-line path → loaded after onboarding. Assert the sequence, as the existing tests do.
- Migration: an old `spagitty.workspace` reads without loss; turning the switch off brings the tabs back.
- Rail: a dimmed row activates 2E, then navigates after the load.
- With the switch **off**, the existing suite passes unchanged.

Manual sweep (each item's `agile/testing/FEAT-18x-sweep.md`): a fresh profile on Windows; the token page opens on CIB's GitLab 16.6.2 with the fields prefilled; a wrong token; a token with no `api` scope; a locked keychain; *Set up later* off the VPN; follow nothing and follow three; reorder Home; load local, clone and load, switch, unload; Ctrl+O from every screen; a repository-scoped screen with nothing loaded. FEAT-184 gets its own sweep: a 1.2 profile with five tabs open, upgraded, then the switch turned off to bring the strip back.

## 13. Backlog items

Order: FEAT-181 → FEAT-182 → FEAT-184 → FEAT-183 → FEAT-180 make 2.0.0. FEAT-185 follows after they have shipped (section 15). Onboarding comes after the pieces it calls exist; it is mostly a shell around them.

The identifiers proposed first, FEAT-173 to FEAT-178, were already taken, by work that shipped in 1.2.0. These are the next free ones.

| ID | Title | Screens | Depends on |
| --- | --- | --- | --- |
| FEAT-180 | First-launch onboarding | 2D, 1K Behaviour | 181, 182, 183 (layout) |
| FEAT-181 | CIB GitLab built in, and the token kept in the keychain | 1K GitLab, 2D steps 2–3 | — |
| FEAT-182 | Follow repositories on CIB GitLab | 1K Home, 2D step 4 | 181 |
| FEAT-183 | Home across followed repositories | 2A | 182 |
| FEAT-184 | Load repository instead of tabs | 2E, chrome, rail | — (182 for the GitLab group) |
| FEAT-185 | Merge requests, Pipelines and Issues across followed repositories | 1H, 1Y, 1W | 182, 184 |

Each one has its item document and its row in `agile/README.md` (the index is checked by `tools/record.test.ts`). Its plan and testing documents are written when its work starts, as `agile/README.md` says for backlog items. In the same change as the work it also gets its changelog entry in `CHANGELOG.workbench.md` under `Unreleased`, and the `docs/screens.md` sections for 2D, 2E and the changed 2A.

## 14. Designs

`design_handoff_onboarding_home/source/` holds Design Component sources and `canvas.json`. **Treat the markup as a layout spec only.** Build with the repo's components and tokens (`src/app.css`, `themes.ts`, `Btn.svelte`, `Icon.svelte`, `Menu.svelte`), not the DC runtime.

| File | Artboard | What it shows |
| --- | --- | --- |
| `Main.dc.html` | Onboarding · 1 Welcome | The whole onboarding as one interactive component; the `step` prop sets the first step |
| `OnboardGitLab.dc.html` | Onboarding · 2 GitLab | `Main` at step 2 |
| `OnboardToken.dc.html` | Onboarding · 3 Token | `Main` at step 3, checked |
| `OnboardHome.dc.html` | Onboarding · 4 Your Home | `Main` at step 4 |
| `Home.dc.html` | Home · following, nothing loaded | The Home component; the `start` prop is `following`, `empty`, `picker` or `loaded` |
| `HomeEmpty.dc.html` | Home · following nothing | `Home` with `start="empty"` |
| `LoadRepo.dc.html` | Load a repository | `Home` with `start="picker"` |
| `HomeLoaded.dc.html` | Home · crx-transfers loaded | `Home` with `start="loaded"` |

**Fidelity:** high for structure, copy and the meaning of each colour. Night only; Day follows from the existing tokens. Every repository, person, count and time is made up. `[COMPANY GITLAB URL]` is a placeholder. Images are `/_blob/<id>`: `8ed2904807391f8815df61b310da6e3b` is `assets/brand/cib/cib-globe-round.png`, and `a3e65eef4d83f5b2b8873571038ea10e` is `assets/brand/cib/cib-logo-reversed.png`. Use them through `BRAND_ASSETS`.

Palette used (the 2026-10-26 corrections):

| Role | Value |
| --- | --- |
| Ground | `#0B0B0C` |
| Panel | `#111214` |
| Raised | `#17181B` |
| Field | `#1B1C20` |
| Ink | `#ECEEF1`, muted at 64–68% |
| Accent text | `#4D8FD6` |
| Primary | `#0C529A` with white text |
| OK | `#5DBE8A` |
| Warning | `#E2A64E` |
| Danger | `#EC6A82` |
| Ambient | CIB-blue radial glow `rgba(12,82,154,0.22–0.28)` |

Fonts: Sora 600 for display, the system UI face for text, the configured mono for code (the designs use JetBrains Mono). Map every value onto the existing custom properties rather than adding new literals.

## 15. Questions for the author

Answered by the author on 2026-10-28.

1. **The GitLab address and version.** `gitlab.apps.ocp-nonprod-01.hodomain.local`, GitLab Enterprise Edition 16.6.2. It is built into the app. The token page path still needs one signed-in check ([6.1](#61-values)).
2. **Scopes.** The full set: `api`, `read_repository`, `write_repository`.
3. **Needs you.** Everything the account is involved in, each row chipped by project. A review request in an unfollowed project still needs you.
4. **Launch.** Reload the last loaded repository by default. FEAT-172's setting, reworded, starts with nothing loaded.
5. **Tabs.** Removed only under `FEATURES.workbenchWide`. `RepoTabs.svelte` stays for the Spagitty path, which keeps the merges clean.
6. **FEAT-185** comes after FEAT-180 to FEAT-184 have shipped.
7. **Onboarding copy.** The designs' wording is UI copy, not brand copy. Descriptions of the product come only from `docs/branding.md`.
