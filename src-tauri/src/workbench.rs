// SPDX-License-Identifier: MIT

//! Workbench-wide state and commands (2.0.0, FEAT-180 to FEAT-184).
//!
//! Workbench only, and a file of its own so that a Spagitty merge never meets
//! it. What it keeps is in `workbench.json`, beside `settings.json` rather than
//! inside it: the Settings screen writes its whole object back on every
//! toggle, and a copy of `followed` it read before onboarding finished would
//! write the old list over the new one.
//!
//! Every GitLab request is made in Rust, to `brand::gitlab::HOST` only, through
//! `spagitty_core::workbench`. The webview sends a token once, to
//! [`gitlab_connect`], and nothing here sends one back.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use spagitty_core::brand::gitlab::HOST;
use spagitty_core::forge::{self, Account, Kind};
use spagitty_core::workbench::{
    self as wide, ConnectFailure, Followed, ProjectPage, TokenInfo, WideHome, MAX_FOLLOWED,
};
use spagitty_core::{Error, Result};
use tauri::{AppHandle, Manager, Runtime};

use crate::{accounts, forge_views};

const FILE: &str = "workbench.json";

/// When onboarding finished, and which version of it. A later build can show
/// only the steps added since.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Onboarded {
    pub version: u32,
    /// RFC 3339.
    pub at: String,
}

/// What onboarding writes, and the steps a version of it has.
pub const ONBOARDING_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WorkbenchState {
    /// Unset until onboarding finishes. *Set up later* leaves it unset.
    pub onboarded: Option<Onboarded>,
    /// The CIB GitLab projects Home reads, by id. Never written to a repository.
    pub followed: Vec<Followed>,
    /// Where *Clone and load* clones into.
    pub clone_into: Option<String>,
}

fn file<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join(FILE))
}

/// A missing or hand-broken file is the defaults: it must never stop a launch.
fn parse(text: &str) -> WorkbenchState {
    serde_json::from_str(text).unwrap_or_default()
}

pub fn load<R: Runtime>(app: &AppHandle<R>) -> WorkbenchState {
    file(app)
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|text| parse(&text))
        .unwrap_or_default()
}

fn save<R: Runtime>(app: &AppHandle<R>, state: &WorkbenchState) -> Result<()> {
    let path =
        file(app).ok_or_else(|| Error::Config("there is no configuration directory".into()))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let text =
        serde_json::to_string_pretty(state).map_err(|error| Error::Config(error.to_string()))?;
    std::fs::write(path, text)?;
    Ok(())
}

fn update<R: Runtime>(
    app: &AppHandle<R>,
    change: impl FnOnce(&mut WorkbenchState),
) -> Result<WorkbenchState> {
    let mut state = load(app);
    change(&mut state);
    save(app, &state)?;
    Ok(state)
}

/// The list as it is kept: on the built-in host, each project once, capped.
fn kept(followed: Vec<Followed>) -> Vec<Followed> {
    let mut seen = std::collections::HashSet::new();
    followed
        .into_iter()
        .filter(|f| seen.insert(f.id))
        .map(|f| Followed {
            host: HOST.into(),
            ..f
        })
        .take(MAX_FOLLOWED)
        .collect()
}

#[tauri::command(async)]
pub fn workbench_state<R: Runtime>(app: AppHandle<R>) -> WorkbenchState {
    load(&app)
}

/// Finish onboarding (`done`), or clear it so the next launch offers it again.
#[tauri::command(async)]
pub fn workbench_set_onboarded<R: Runtime>(
    app: AppHandle<R>,
    done: bool,
    at: String,
) -> Result<WorkbenchState> {
    update(&app, |state| {
        state.onboarded = done.then_some(Onboarded {
            version: ONBOARDING_VERSION,
            at,
        });
    })
}

#[tauri::command(async)]
pub fn workbench_set_followed<R: Runtime>(
    app: AppHandle<R>,
    followed: Vec<Followed>,
) -> Result<WorkbenchState> {
    update(&app, |state| state.followed = kept(followed))
}

#[tauri::command(async)]
pub fn workbench_set_clone_into<R: Runtime>(
    app: AppHandle<R>,
    path: String,
) -> Result<WorkbenchState> {
    update(&app, |state| {
        state.clone_into = Some(path).filter(|p| !p.trim().is_empty())
    })
}

// --- The token (FEAT-181) --------------------------------------------------

/// The account connected for the built-in host, if any.
fn account<R: Runtime>(app: &AppHandle<R>) -> Option<Account> {
    accounts::for_host(&accounts::load(app), HOST).cloned()
}

/// The stored token. `None` when nothing is connected; an error when the
/// keychain will not give it up.
fn token<R: Runtime>(app: &AppHandle<R>) -> Result<Option<(Account, String)>> {
    let Some(account) = account(app) else {
        return Ok(None);
    };
    match forge::keychain::read(&account.host, &account.user)? {
        Some(token) => Ok(Some((account, token))),
        None => Ok(None),
    }
}

async fn off_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> std::result::Result<T, String> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|error| format!("the request could not be run: {error}"))
}

/// Check a token against CIB GitLab, refuse one short of a scope, and only
/// then keep it in the keychain (proposal § 6.4).
///
/// `forge_connect` is Spagitty's and stays as it is; this is the same path
/// with the scope check in front. Another account on the host is
/// disconnected, keychain first: Workbench talks to one GitLab, as one person.
#[tauri::command]
pub async fn gitlab_connect<R: Runtime>(
    app: AppHandle<R>,
    token: String,
) -> std::result::Result<TokenInfo, ConnectFailure> {
    let held = token.trim().to_string();
    let checked = {
        let held = held.clone();
        off_thread(move || {
            let fetch = wide::fetcher(HOST, &held);
            let mut login = None;
            let info = wide::connect(HOST, &held, &fetch, |user| {
                forge::keychain::store(HOST, user, &held)?;
                login = Some(user.to_string());
                Ok(())
            })?;
            Ok::<_, ConnectFailure>((info, login))
        })
        .await
        .map_err(|detail| ConnectFailure::Other { detail })??
    };
    let (info, login) = checked;
    let user = login.unwrap_or_else(|| info.login.clone());
    for other in accounts::load(&app)
        .into_iter()
        .filter(|a| a.host == HOST && a.user != user)
    {
        let _ = forge::keychain::forget(&other.host, &other.user);
        accounts::forget(&app, &other.host, &other.user);
    }
    accounts::remember(
        &app,
        Account {
            kind: Kind::GitLab,
            host: HOST.into(),
            user,
        },
    );
    Ok(info)
}

/// Who the stored token belongs to, read from GitLab. `None` when no account
/// is connected for the built-in host.
#[tauri::command]
pub async fn gitlab_token_info<R: Runtime>(
    app: AppHandle<R>,
) -> std::result::Result<Option<TokenInfo>, ConnectFailure> {
    let Some((_, token)) = token(&app)? else {
        return Ok(None);
    };
    off_thread(move || wide::token_info(HOST, &wide::fetcher(HOST, &token)).map(Some))
        .await
        .map_err(|detail| ConnectFailure::Other { detail })?
}

fn connected<R: Runtime>(app: &AppHandle<R>) -> Result<(Account, String)> {
    token(app)?.ok_or_else(|| Error::ForgeUnauthorized {
        host: HOST.into(),
        detail: "connect your CIB GitLab account in Settings › GitLab".into(),
    })
}

// --- Projects (FEAT-182) ---------------------------------------------------

fn cache<R: Runtime>(app: &AppHandle<R>, name: &str) -> Result<PathBuf> {
    let key = spagitty_core::hooks::generate::hash(name.as_bytes());
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| Error::Config(error.to_string()))?
        .join("workbench")
        .join(format!("{key:016x}.json")))
}

fn projects_key(search: &Option<String>, page: u32) -> String {
    format!(
        "projects:{}:{page}",
        search.as_deref().unwrap_or("").trim().to_lowercase()
    )
}

/// The last answer for this search and page, to paint before reading again.
#[tauri::command(async)]
pub fn gitlab_projects_cached<R: Runtime>(
    app: AppHandle<R>,
    search: Option<String>,
    page: u32,
) -> Option<ProjectPage> {
    forge_views::cached(&cache(&app, &projects_key(&search, page)).ok()?)
}

/// One page of the projects the person is a member of. Followed paths are
/// brought up to date from it, so a renamed project shows its new path.
#[tauri::command]
pub async fn gitlab_projects<R: Runtime>(
    app: AppHandle<R>,
    search: Option<String>,
    page: u32,
) -> Result<ProjectPage> {
    let (_, token) = connected(&app)?;
    let asked = search.clone();
    let found = off_thread(move || {
        wide::projects(HOST, asked.as_deref(), page, &wide::fetcher(HOST, &token))
    })
    .await
    .map_err(|detail| Error::Forge {
        host: HOST.into(),
        detail,
    })??;
    if let Ok(path) = cache(&app, &projects_key(&search, page)) {
        let _ = forge_views::save(&path, &found);
    }
    let state = load(&app);
    if let Some(next) = wide::refreshed(&state.followed, &found.projects) {
        let _ = update(&app, |state| state.followed = next);
    }
    Ok(found)
}

/// A repository on this machine that is a clone of a CIB GitLab project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalClone {
    pub path: String,
    /// `group/name`, as GitLab names it.
    pub project: String,
}

/// Which remembered repositories are clones of CIB GitLab projects, in one
/// read: Load repository shows a cloned project under *On this machine*,
/// not twice (FEAT-184). The FEAT-087 lookup, for every project at once.
#[tauri::command(async)]
pub fn gitlab_local_clones<R: Runtime>(app: AppHandle<R>) -> Vec<LocalClone> {
    let known = accounts::load(&app);
    crate::recents::load(&app)
        .into_iter()
        .filter_map(|path| {
            let repository = spagitty_core::repo::open(&path).ok()?;
            let found = forge::identify_repo_with(&repository, &known)
                .ok()
                .flatten()?;
            (found.host == HOST).then(|| LocalClone {
                path: path.to_string_lossy().into_owned(),
                project: found.slug(),
            })
        })
        .collect()
}

// --- Home (FEAT-183) -------------------------------------------------------

/// Home's last answer, painted before it is read again.
#[tauri::command(async)]
pub fn home_workbench_wide_cached<R: Runtime>(app: AppHandle<R>) -> Option<WideHome> {
    forge_views::cached(&cache(&app, "home").ok()?)
}

/// Every Workbench-wide card's rows, in one call.
///
/// Unless `force`, a project read within the forge watcher's interval is
/// taken from the last answer rather than read again.
#[tauri::command]
pub async fn home_workbench_wide<R: Runtime>(app: AppHandle<R>, force: bool) -> Result<WideHome> {
    let (account, token) = connected(&app)?;
    let followed = load(&app).followed;
    let prior: Option<WideHome> = cache(&app, "home")
        .ok()
        .and_then(|path| forge_views::cached(&path));
    let fresh = if force {
        0
    } else {
        i64::from(crate::settings::load(&app).notify_every_minutes) * 60
    };
    let home = off_thread(move || {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let ask = wide::Ask {
            host: HOST,
            me: &account.user,
            followed: &followed,
            prior: prior.as_ref(),
            now,
            fresh,
        };
        wide::home_wide(&ask, &wide::fetcher(HOST, &token))
    })
    .await
    .map_err(|detail| Error::Forge {
        host: HOST.into(),
        detail,
    })?;
    if let Ok(path) = cache(&app, "home") {
        let _ = forge_views::save(&path, &home);
    }
    Ok(home)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn followed(id: u64) -> Followed {
        Followed {
            host: "elsewhere.example.com".into(),
            id,
            path: format!("g/p{id}"),
            default_branch: None,
            http_url: None,
        }
    }

    #[test]
    fn a_missing_or_broken_file_is_the_defaults() {
        assert_eq!(parse(""), WorkbenchState::default());
        assert_eq!(parse("{"), WorkbenchState::default());
        assert!(parse("{}").onboarded.is_none());
    }

    #[test]
    fn the_file_keeps_what_it_is_given_and_carries_no_token() {
        let state = WorkbenchState {
            onboarded: Some(Onboarded {
                version: 1,
                at: "2026-10-28T12:00:00Z".into(),
            }),
            followed: vec![followed(7)],
            clone_into: Some("C:\\Projects".into()),
        };
        let text = serde_json::to_string(&state).unwrap();
        assert_eq!(parse(&text), state);
        assert!(!text.to_lowercase().contains("token"), "{text}");
    }

    #[test]
    fn the_followed_list_is_on_the_built_in_host_once_each_and_capped() {
        let mut list: Vec<Followed> = (1..=60).map(followed).collect();
        list.insert(1, followed(1));
        let kept = kept(list);
        assert_eq!(kept.len(), MAX_FOLLOWED);
        assert!(kept.iter().all(|f| f.host == HOST));
        assert_eq!(kept.iter().filter(|f| f.id == 1).count(), 1);
    }
}
