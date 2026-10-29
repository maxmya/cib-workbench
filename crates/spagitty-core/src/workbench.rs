// SPDX-License-Identifier: MIT

//! Workbench-wide reads of CIB GitLab (FEAT-181, FEAT-182, FEAT-183).
//!
//! Workbench only, behind `brand::features::WORKBENCH_WIDE`, and in a file of
//! its own so that a Spagitty merge never meets it. Three things:
//!
//! - **The token** ([`connect`], [`token_info`]): proved against GitLab, its
//!   scopes read back, and only then handed to the caller to keep. Nothing
//!   returned here carries it.
//! - **The projects a person may follow** ([`projects`]): the ones they are a
//!   member of, most active first.
//! - **Home across the followed ones** ([`home_wide`]): merge requests,
//!   pipelines, assigned issues and releases, read without a clone, at most
//!   [`IN_FLIGHT`] requests at a time.
//!
//! Every request goes through `forge::http`, to the one host in
//! `brand::gitlab`. Each function takes the request as a [`Fetch`], so the
//! tests answer for GitLab and the real one is [`fetcher`].

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::brand::gitlab::SCOPES;
use crate::forge::github::timestamp;
use crate::forge::issues::{read_issue, Issue};
use crate::forge::pipelines::{run as read_run, Run};
use crate::forge::releases::{read_release, Release};
use crate::forge::{encode_segment, http, status_error, Kind};
use crate::{Error, Result};

/// What a `GET` answered: the status, the body, and whether a next page waits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub body: String,
    pub more: bool,
}

/// How every request here is made.
pub type Fetch<'a> = dyn Fn(&str) -> Result<Answer> + Sync + 'a;

/// The real request: `forge::http`, with the token, to `host` only.
pub fn fetcher<'a>(host: &'a str, token: &'a str) -> impl Fn(&str) -> Result<Answer> + Sync + 'a {
    move |url: &str| {
        let got = http::get_conditional_json(url, token, host, None)?;
        Ok(Answer {
            status: got.response.status,
            body: got.response.body,
            more: got.more,
        })
    }
}

fn api(host: &str) -> String {
    Kind::GitLab.api_base(host)
}

fn ok(answer: &Answer) -> bool {
    (200..300).contains(&answer.status)
}

/// `GET url` as JSON; a refused status is the error that says which.
fn json(fetch: &Fetch<'_>, host: &str, url: &str) -> Result<(Value, bool)> {
    let answer = fetch(url)?;
    if !ok(&answer) {
        return Err(status_error(host, answer.status, &answer.body, None));
    }
    let value = serde_json::from_str(&answer.body).map_err(|error| Error::Forge {
        host: host.to_string(),
        detail: format!("GitLab sent something that is not JSON: {error}"),
    })?;
    Ok((value, answer.more))
}

fn text(value: &Value, key: &str) -> String {
    value[key].as_str().unwrap_or_default().to_string()
}

// --- The token (FEAT-181) --------------------------------------------------

/// Who a token belongs to, and what it may do. **Never the token.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// The person's display name.
    pub name: String,
    /// Their username, which is how GitLab names them everywhere else.
    pub login: String,
    /// What the token was granted. Empty when GitLab could not say.
    pub scopes: Vec<String>,
    /// False on a GitLab older than 15.5, which has no
    /// `personal_access_tokens/self`: the login is all there is.
    pub scopes_known: bool,
    /// `YYYY-MM-DD`, or none for a token that does not expire.
    pub expires_at: Option<String>,
}

/// Why a token was not kept. One of the four the onboarding names, or other.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ConnectFailure {
    /// GitLab refused it: wrong, incomplete, expired or revoked.
    Unauthorized {
        detail: String,
    },
    /// GitLab could not be reached: off the VPN, or a proxy in the way.
    Unreachable {
        detail: String,
    },
    /// It works, and lacks scopes Workbench needs. Named, in `SCOPES` order.
    MissingScope {
        scopes: Vec<String>,
    },
    /// The system keychain refused to keep it. Nothing was saved.
    Keychain {
        detail: String,
    },
    Other {
        detail: String,
    },
}

impl From<Error> for ConnectFailure {
    fn from(error: Error) -> Self {
        match error {
            Error::ForgeUnauthorized { detail, .. } => ConnectFailure::Unauthorized { detail },
            Error::ForgeOffline { detail, .. } => ConnectFailure::Unreachable { detail },
            Error::Keychain(detail) => ConnectFailure::Keychain { detail },
            other => ConnectFailure::Other {
                detail: other.to_string(),
            },
        }
    }
}

/// GitLab's answer to a token that lacks the scope an endpoint needs.
fn insufficient_scope(answer: &Answer) -> bool {
    answer.status == 403 && answer.body.contains("insufficient_scope")
}

/// Read who a token belongs to and what it may do.
///
/// `GET /user` for the name and login, then `GET /personal_access_tokens/self`
/// for the scopes and the expiry. A GitLab older than 15.5 answers the second
/// 404, and the login is all there is.
pub fn token_info(host: &str, fetch: &Fetch<'_>) -> std::result::Result<TokenInfo, ConnectFailure> {
    let answer = fetch(&format!("{}/user", api(host)))?;
    if insufficient_scope(&answer) {
        // Without `api` (or `read_user`) GitLab will not say who this is.
        return Err(ConnectFailure::MissingScope {
            scopes: vec![SCOPES[0].to_string()],
        });
    }
    if !ok(&answer) {
        return Err(status_error(host, answer.status, &answer.body, None).into());
    }
    let user: Value =
        serde_json::from_str(&answer.body).map_err(|error| ConnectFailure::Other {
            detail: format!("GitLab sent something that is not JSON: {error}"),
        })?;
    let login = text(&user, "username");
    if login.is_empty() {
        return Err(ConnectFailure::Other {
            detail: "GitLab did not say whose token this is".into(),
        });
    }
    let name = match text(&user, "name") {
        name if name.is_empty() => login.clone(),
        name => name,
    };

    let answer = fetch(&format!("{}/personal_access_tokens/self", api(host)))?;
    let (scopes, scopes_known, expires_at) = if ok(&answer) {
        let token: Value = serde_json::from_str(&answer.body).unwrap_or(Value::Null);
        let scopes = token["scopes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|scope| scope.as_str().map(str::to_string))
            .collect();
        (
            scopes,
            true,
            token["expires_at"].as_str().map(str::to_string),
        )
    } else if matches!(answer.status, 403 | 404) {
        (Vec::new(), false, None)
    } else {
        return Err(status_error(host, answer.status, &answer.body, None).into());
    };

    Ok(TokenInfo {
        name,
        login,
        scopes,
        scopes_known,
        expires_at,
    })
}

/// The scopes Workbench needs that the token was not granted, in `SCOPES`
/// order. None when GitLab could not say: the requests themselves will.
pub fn missing_scopes(info: &TokenInfo) -> Vec<String> {
    if !info.scopes_known {
        return Vec::new();
    }
    SCOPES
        .iter()
        .filter(|scope| !info.scopes.iter().any(|held| held == *scope))
        .map(|scope| scope.to_string())
        .collect()
}

/// Prove a token, check its scopes, and only then keep it.
///
/// `keep` is given the login and stores the token; it is called at most once,
/// and never for a token that was refused, unreachable or short of a scope.
pub fn connect(
    host: &str,
    token: &str,
    fetch: &Fetch<'_>,
    keep: impl FnOnce(&str) -> Result<()>,
) -> std::result::Result<TokenInfo, ConnectFailure> {
    if token.trim().is_empty() {
        return Err(ConnectFailure::Other {
            detail: "a token is needed".into(),
        });
    }
    let info = token_info(host, fetch)?;
    let missing = missing_scopes(&info);
    if !missing.is_empty() {
        return Err(ConnectFailure::MissingScope { scopes: missing });
    }
    keep(&info.login)?;
    Ok(info)
}

// --- Projects to follow (FEAT-182) -----------------------------------------

/// A GitLab project, as the follow list and Load repository show it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: u64,
    /// `path_with_namespace`: `group/sub/name`.
    pub path: String,
    pub name: String,
    pub description: String,
    /// Unix seconds.
    pub last_activity: i64,
    pub default_branch: Option<String>,
    /// What `git clone` is given.
    pub http_url: String,
    pub web_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPage {
    pub projects: Vec<Project>,
    pub page: u32,
    pub more: bool,
}

/// Projects per page.
pub const PROJECTS_PER_PAGE: u32 = 50;

/// The projects the person is a member of, most active first.
pub fn projects_url(host: &str, search: Option<&str>, page: u32) -> String {
    let mut url = format!(
        "{}/projects?membership=true&archived=false&simple=true&order_by=last_activity_at&per_page={PROJECTS_PER_PAGE}&page={}",
        api(host),
        page.max(1)
    );
    if let Some(search) = search.map(str::trim).filter(|s| !s.is_empty()) {
        url.push_str("&search=");
        url.push_str(&encode_segment(search));
    }
    url
}

pub fn read_project(value: &Value) -> Option<Project> {
    Some(Project {
        id: value["id"].as_u64()?,
        path: value["path_with_namespace"].as_str()?.to_string(),
        name: text(value, "name"),
        description: text(value, "description"),
        last_activity: timestamp(value["last_activity_at"].as_str()),
        default_branch: value["default_branch"].as_str().map(str::to_string),
        http_url: text(value, "http_url_to_repo"),
        web_url: text(value, "web_url"),
    })
}

/// One page of [`projects_url`].
pub fn projects(
    host: &str,
    search: Option<&str>,
    page: u32,
    fetch: &Fetch<'_>,
) -> Result<ProjectPage> {
    let (list, more) = json(fetch, host, &projects_url(host, search, page))?;
    Ok(ProjectPage {
        projects: list
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(read_project)
            .collect(),
        page: page.max(1),
        more,
    })
}

/// A project the person follows. Kept by id, so a rename or a move does not
/// lose it; the path is refreshed from the last listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Followed {
    pub host: String,
    pub id: u64,
    pub path: String,
    #[serde(default)]
    pub default_branch: Option<String>,
    #[serde(default)]
    pub http_url: Option<String>,
}

/// How many projects can be followed. Home reads each of them, twice.
pub const MAX_FOLLOWED: usize = 50;

/// `followed` with each path, default branch and clone address brought up to
/// date from `listed`. `None` when nothing changed.
pub fn refreshed(followed: &[Followed], listed: &[Project]) -> Option<Vec<Followed>> {
    let mut changed = false;
    let next = followed
        .iter()
        .map(|held| match listed.iter().find(|p| p.id == held.id) {
            Some(project) => {
                let fresh = Followed {
                    host: held.host.clone(),
                    id: held.id,
                    path: project.path.clone(),
                    default_branch: project.default_branch.clone(),
                    http_url: Some(project.http_url.clone()).filter(|u| !u.is_empty()),
                };
                changed |= fresh != *held;
                fresh
            }
            None => held.clone(),
        })
        .collect();
    changed.then_some(next)
}

// --- Home across followed projects (FEAT-183) ------------------------------

/// Requests in flight at once, whatever is followed.
pub const IN_FLIGHT: usize = 4;

/// A merge request, as Home lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergeRequest {
    pub id: u64,
    pub iid: u64,
    pub title: String,
    pub author: String,
    /// Unix seconds.
    pub updated: i64,
    pub source_branch: String,
    pub target_branch: String,
    pub draft: bool,
    /// The person opened it.
    pub mine: bool,
    /// The person is one of its reviewers.
    pub review_requested: bool,
    pub conflicts: bool,
    pub url: String,
}

/// A row and the project it belongs to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InProject<T> {
    pub project_id: u64,
    /// `group/name`.
    pub project: String,
    pub item: T,
}

/// What one card could not read. `project_id` 0 is a read across the host.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectError {
    pub project_id: u64,
    pub project: String,
    /// The card it belongs to: `requests`, `pipelines`, `issues` or `releases`.
    pub card: String,
    pub detail: String,
}

/// Everything Home's Workbench-wide cards show. No field carries the token.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WideHome {
    /// Open merge requests the person opened or reviews, anywhere on the host.
    pub requests: Vec<InProject<MergeRequest>>,
    /// The latest run on each followed project's default branch, and on the
    /// person's own merge request branches there.
    pub pipelines: Vec<InProject<Run>>,
    /// Open issues assigned to the person, anywhere on the host.
    pub issues: Vec<InProject<Issue>>,
    /// The latest release of each followed project that has one.
    pub releases: Vec<InProject<Release>>,
    pub errors: Vec<ProjectError>,
    /// When each followed project was last read, unix seconds, by id.
    pub read: HashMap<u64, i64>,
    /// When the reads across the host were last made.
    pub read_wide: i64,
}

/// What a read of Home is asked for.
pub struct Ask<'a> {
    pub host: &'a str,
    pub me: &'a str,
    pub followed: &'a [Followed],
    /// The last answer. A project read within `fresh` seconds of `now` is
    /// taken from it rather than read again.
    pub prior: Option<&'a WideHome>,
    pub now: i64,
    /// Zero reads everything.
    pub fresh: i64,
}

/// The project a host-wide row names: `references.full` is `group/name!12`
/// or `group/name#12`.
fn project_of(value: &Value, mark: char) -> String {
    value["references"]["full"]
        .as_str()
        .and_then(|full| full.rsplit_once(mark))
        .map(|(project, _)| project.to_string())
        .unwrap_or_default()
}

pub fn read_merge_request(value: &Value, me: &str) -> Option<InProject<MergeRequest>> {
    let is_me = |login: Option<&str>| {
        login.is_some_and(|login| !me.is_empty() && login.eq_ignore_ascii_case(me))
    };
    let title = text(value, "title");
    Some(InProject {
        project_id: value["project_id"].as_u64()?,
        project: project_of(value, '!'),
        item: MergeRequest {
            id: value["id"].as_u64()?,
            iid: value["iid"].as_u64()?,
            draft: value["draft"]
                .as_bool()
                .or_else(|| value["work_in_progress"].as_bool())
                .unwrap_or(false)
                || title.starts_with("Draft:"),
            title,
            author: text(&value["author"], "username"),
            updated: timestamp(value["updated_at"].as_str()),
            source_branch: text(value, "source_branch"),
            target_branch: text(value, "target_branch"),
            mine: is_me(value["author"]["username"].as_str()),
            review_requested: value["reviewers"]
                .as_array()
                .into_iter()
                .flatten()
                .any(|reviewer| is_me(reviewer["username"].as_str())),
            conflicts: value["has_conflicts"].as_bool().unwrap_or(false),
            url: text(value, "web_url"),
        },
    })
}

fn read_assigned(value: &Value) -> Option<InProject<Issue>> {
    Some(InProject {
        project_id: value["project_id"].as_u64()?,
        project: project_of(value, '#'),
        item: read_issue(value, Kind::GitLab)?,
    })
}

/// One request a project's cards need.
enum Job<'a> {
    /// The latest run on a branch; `None` is the latest on any.
    Pipeline(&'a Followed, Option<String>),
    Release(&'a Followed),
}

/// What a [`Job`] found.
enum Found {
    Run(InProject<Run>),
    Release(InProject<Release>),
    Nothing,
    Failed(ProjectError),
}

fn first(list: &Value) -> Option<&Value> {
    list.as_array().and_then(|rows| rows.first())
}

fn work(job: &Job<'_>, host: &str, fetch: &Fetch<'_>) -> Found {
    let (followed, url, card) = match job {
        Job::Pipeline(followed, branch) => {
            let mut url = format!(
                "{}/projects/{}/pipelines?per_page=1",
                api(host),
                followed.id
            );
            if let Some(branch) = branch {
                url.push_str("&ref=");
                url.push_str(&encode_segment(branch));
            }
            (*followed, url, "pipelines")
        }
        Job::Release(followed) => (
            *followed,
            format!("{}/projects/{}/releases?per_page=1", api(host), followed.id),
            "releases",
        ),
    };
    let failed = |detail: String| {
        Found::Failed(ProjectError {
            project_id: followed.id,
            project: followed.path.clone(),
            card: card.into(),
            detail,
        })
    };
    let answer = match fetch(&url) {
        Ok(answer) => answer,
        Err(error) => return failed(error.to_string()),
    };
    // A project with CI or releases switched off answers 403 or 404: it has
    // none, which is not a failure to show.
    if matches!(answer.status, 403 | 404) {
        return Found::Nothing;
    }
    if !ok(&answer) {
        return failed(status_error(host, answer.status, &answer.body, None).to_string());
    }
    let Ok(list) = serde_json::from_str::<Value>(&answer.body) else {
        return failed("GitLab sent something that is not JSON".into());
    };
    let Some(row) = first(&list) else {
        return Found::Nothing;
    };
    let (project_id, project) = (followed.id, followed.path.clone());
    match job {
        Job::Pipeline(..) => Found::Run(InProject {
            project_id,
            project,
            item: read_run(row, false),
        }),
        Job::Release(..) => Found::Release(InProject {
            project_id,
            project,
            item: read_release(row, Kind::GitLab),
        }),
    }
}

/// Run `jobs` with at most [`IN_FLIGHT`] in flight, answers in job order.
fn fan_out(jobs: &[Job<'_>], host: &str, fetch: &Fetch<'_>) -> Vec<Found> {
    let next = AtomicUsize::new(0);
    let found: Mutex<Vec<(usize, Found)>> = Mutex::new(Vec::with_capacity(jobs.len()));
    std::thread::scope(|scope| {
        for _ in 0..IN_FLIGHT.min(jobs.len()) {
            scope.spawn(|| loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                let Some(job) = jobs.get(index) else { break };
                let answer = work(job, host, fetch);
                found.lock().expect("home rows").push((index, answer));
            });
        }
    });
    let mut found = found.into_inner().expect("home rows");
    found.sort_by_key(|(index, _)| *index);
    found.into_iter().map(|(_, answer)| answer).collect()
}

/// A person's own merge request branches, per project: at most two each.
const OWN_BRANCHES: usize = 2;

/// Read Home's Workbench-wide cards.
///
/// Never fails as a whole: what one read could not get is a [`ProjectError`]
/// in its card, and the rest is still shown.
pub fn home_wide(ask: &Ask<'_>, fetch: &Fetch<'_>) -> WideHome {
    let host = ask.host;
    let fresh = |at: Option<&i64>| at.is_some_and(|at| ask.fresh > 0 && ask.now - at < ask.fresh);
    let mut home = WideHome::default();

    // Across the host: the person's merge requests and review requests, and
    // the issues assigned to them, followed or not (Needs you).
    let prior_wide = ask.prior.filter(|prior| fresh(Some(&prior.read_wide)));
    if let Some(prior) = prior_wide {
        home.requests = prior.requests.clone();
        home.issues = prior.issues.clone();
        home.errors
            .extend(prior.errors.iter().filter(|e| e.project_id == 0).cloned());
        home.read_wide = prior.read_wide;
    } else {
        home.read_wide = ask.now;
        let list = format!("{}/merge_requests", api(host));
        let order = "state=opened&order_by=updated_at&sort=desc&per_page=30";
        let mut urls = vec![format!("{list}?scope=created_by_me&{order}")];
        if !ask.me.is_empty() {
            urls.push(format!(
                "{list}?scope=all&reviewer_username={}&{order}",
                encode_segment(ask.me)
            ));
        }
        let mut seen = HashSet::new();
        for url in urls {
            match json(fetch, host, &url) {
                Ok((rows, _)) => {
                    for row in rows.as_array().into_iter().flatten() {
                        if let Some(request) = read_merge_request(row, ask.me) {
                            if seen.insert(request.item.id) {
                                home.requests.push(request);
                            }
                        }
                    }
                }
                Err(error) => home.errors.push(ProjectError {
                    project_id: 0,
                    project: String::new(),
                    card: "requests".into(),
                    detail: error.to_string(),
                }),
            }
        }
        let issues = format!(
            "{}/issues?scope=assigned_to_me&state=opened&order_by=updated_at&per_page=20",
            api(host)
        );
        match json(fetch, host, &issues) {
            Ok((rows, _)) => {
                home.issues = rows
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(read_assigned)
                    .collect();
            }
            Err(error) => home.errors.push(ProjectError {
                project_id: 0,
                project: String::new(),
                card: "issues".into(),
                detail: error.to_string(),
            }),
        }
    }

    // Per followed project: its latest runs and its latest release.
    let mut jobs = Vec::new();
    for followed in ask.followed.iter().take(MAX_FOLLOWED) {
        let prior = ask
            .prior
            .filter(|prior| fresh(prior.read.get(&followed.id)));
        if let Some(prior) = prior {
            let same = |id: u64| id == followed.id;
            home.pipelines.extend(
                prior
                    .pipelines
                    .iter()
                    .filter(|r| same(r.project_id))
                    .cloned(),
            );
            home.releases.extend(
                prior
                    .releases
                    .iter()
                    .filter(|r| same(r.project_id))
                    .cloned(),
            );
            home.errors
                .extend(prior.errors.iter().filter(|e| same(e.project_id)).cloned());
            home.read.insert(followed.id, prior.read[&followed.id]);
            continue;
        }
        home.read.insert(followed.id, ask.now);
        jobs.push(Job::Pipeline(followed, followed.default_branch.clone()));
        let mut own: Vec<String> = Vec::new();
        for request in home
            .requests
            .iter()
            .filter(|r| r.project_id == followed.id && r.item.mine)
        {
            let branch = &request.item.source_branch;
            if own.len() < OWN_BRANCHES
                && !own.contains(branch)
                && Some(branch) != followed.default_branch.as_ref()
            {
                own.push(branch.clone());
            }
        }
        for branch in own {
            jobs.push(Job::Pipeline(followed, Some(branch)));
        }
        jobs.push(Job::Release(followed));
    }

    for found in fan_out(&jobs, host, fetch) {
        match found {
            Found::Run(run) => home.pipelines.push(run),
            Found::Release(release) => home.releases.push(release),
            Found::Failed(error) => home.errors.push(error),
            Found::Nothing => {}
        }
    }
    home
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::AtomicUsize;

    const HOST: &str = "gitlab.example.com";

    fn answer(status: u16, body: Value) -> Result<Answer> {
        Ok(Answer {
            status,
            body: body.to_string(),
            more: false,
        })
    }

    /// Answers by the first route whose fragment the URL contains.
    fn gitlab(routes: Vec<(&'static str, u16, Value)>) -> impl Fn(&str) -> Result<Answer> + Sync {
        move |url: &str| {
            for (fragment, status, body) in &routes {
                if url.contains(fragment) {
                    return answer(*status, body.clone());
                }
            }
            answer(404, json!({ "message": "404 Not found" }))
        }
    }

    fn user() -> (&'static str, u16, Value) {
        (
            "/api/v4/user",
            200,
            json!({ "username": "n.hassan", "name": "Nour Hassan" }),
        )
    }

    fn token(scopes: &[&str]) -> (&'static str, u16, Value) {
        (
            "/personal_access_tokens/self",
            200,
            json!({ "scopes": scopes, "expires_at": "2027-01-26", "name": "Workbench" }),
        )
    }

    #[test]
    fn a_good_token_is_kept_once_and_the_answer_carries_no_token() {
        let fetch = gitlab(vec![
            user(),
            token(&["api", "read_repository", "write_repository"]),
        ]);
        let kept = AtomicUsize::new(0);

        let info = connect(HOST, "glpat-secret", &fetch, |login| {
            assert_eq!(login, "n.hassan");
            kept.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .unwrap();

        assert_eq!(kept.load(Ordering::SeqCst), 1);
        assert_eq!(info.login, "n.hassan");
        assert_eq!(info.name, "Nour Hassan");
        assert_eq!(info.expires_at.as_deref(), Some("2027-01-26"));
        let text = serde_json::to_string(&info).unwrap();
        assert!(!text.contains("glpat-secret"), "{text}");
        assert!(!text.to_lowercase().contains("token\""), "{text}");
    }

    #[test]
    fn a_refused_token_is_not_kept_and_says_so() {
        let fetch = gitlab(vec![(
            "/api/v4/user",
            401,
            json!({ "message": "401 Unauthorized" }),
        )]);
        let failure = connect(HOST, "glpat-wrong", &fetch, |_| {
            panic!("kept a refused token")
        })
        .unwrap_err();
        assert!(
            matches!(failure, ConnectFailure::Unauthorized { .. }),
            "{failure:?}"
        );
    }

    #[test]
    fn a_token_short_of_a_scope_is_not_kept_and_names_it() {
        let fetch = gitlab(vec![user(), token(&["api", "read_repository"])]);
        let failure = connect(HOST, "glpat-short", &fetch, |_| {
            panic!("kept a short token")
        })
        .unwrap_err();
        assert_eq!(
            failure,
            ConnectFailure::MissingScope {
                scopes: vec!["write_repository".into()]
            }
        );
    }

    #[test]
    fn a_token_without_api_is_named_as_missing_api() {
        let fetch = gitlab(vec![(
            "/api/v4/user",
            403,
            json!({ "error": "insufficient_scope", "error_description": "The request requires higher privileges" }),
        )]);
        let failure = connect(HOST, "glpat-read", &fetch, |_| panic!("kept")).unwrap_err();
        assert_eq!(
            failure,
            ConnectFailure::MissingScope {
                scopes: vec!["api".into()]
            }
        );
    }

    #[test]
    fn an_unreachable_gitlab_keeps_nothing() {
        let fetch = |_: &str| -> Result<Answer> {
            Err(Error::ForgeOffline {
                host: HOST.into(),
                detail: "dns error".into(),
            })
        };
        let failure = connect(HOST, "glpat-x", &fetch, |_| panic!("kept")).unwrap_err();
        assert!(
            matches!(failure, ConnectFailure::Unreachable { .. }),
            "{failure:?}"
        );
    }

    #[test]
    fn a_keychain_refusal_is_its_own_failure() {
        let fetch = gitlab(vec![user(), token(&SCOPES)]);
        let failure = connect(HOST, "glpat-x", &fetch, |_| {
            Err(Error::Keychain("locked".into()))
        })
        .unwrap_err();
        assert_eq!(
            failure,
            ConnectFailure::Keychain {
                detail: "locked".into()
            }
        );
    }

    #[test]
    fn an_older_gitlab_without_the_token_endpoint_gives_the_login_only() {
        let fetch = gitlab(vec![user()]);
        let info = token_info(HOST, &fetch).unwrap();
        assert_eq!(info.login, "n.hassan");
        assert!(!info.scopes_known);
        assert!(info.scopes.is_empty());
        assert!(missing_scopes(&info).is_empty());
    }

    #[test]
    fn the_failure_is_tagged_for_the_webview() {
        let text = serde_json::to_string(&ConnectFailure::MissingScope {
            scopes: vec!["api".into()],
        })
        .unwrap();
        assert_eq!(text, r#"{"kind":"missingScope","scopes":["api"]}"#);
    }

    #[test]
    fn projects_are_listed_by_membership_most_active_first_and_searched_in_the_url() {
        let url = projects_url(HOST, Some("crx transfers"), 2);
        assert_eq!(
            url,
            "https://gitlab.example.com/api/v4/projects?membership=true&archived=false&simple=true&order_by=last_activity_at&per_page=50&page=2&search=crx%20transfers"
        );
        assert!(!projects_url(HOST, Some("  "), 1).contains("search"));
    }

    #[test]
    fn a_page_of_projects_reads_and_says_whether_more_follow() {
        let fetch = |_: &str| -> Result<Answer> {
            Ok(Answer {
                status: 200,
                body: json!([{
                    "id": 7, "path_with_namespace": "payments/crx-transfers", "name": "crx-transfers",
                    "description": null, "last_activity_at": "2026-10-28T10:00:00Z",
                    "default_branch": "main", "http_url_to_repo": "https://gitlab.example.com/payments/crx-transfers.git",
                    "web_url": "https://gitlab.example.com/payments/crx-transfers"
                }, { "id": "not a number" }])
                .to_string(),
                more: true,
            })
        };
        let page = projects(HOST, None, 1, &fetch).unwrap();
        assert!(page.more);
        assert_eq!(page.projects.len(), 1);
        assert_eq!(page.projects[0].path, "payments/crx-transfers");
        assert_eq!(page.projects[0].default_branch.as_deref(), Some("main"));
        assert!(page.projects[0].last_activity > 0);
    }

    fn followed(id: u64, path: &str) -> Followed {
        Followed {
            host: HOST.into(),
            id,
            path: path.into(),
            default_branch: Some("main".into()),
            http_url: None,
        }
    }

    fn project(id: u64, path: &str) -> Project {
        Project {
            id,
            path: path.into(),
            name: String::new(),
            description: String::new(),
            last_activity: 0,
            default_branch: Some("main".into()),
            http_url: format!("https://{HOST}/{path}.git"),
            web_url: String::new(),
        }
    }

    #[test]
    fn a_renamed_project_stays_followed_under_its_new_path() {
        let held = vec![followed(7, "payments/crx"), followed(9, "cards/api")];
        let next = refreshed(&held, &[project(7, "payments/crx-transfers")]).unwrap();
        assert_eq!(next[0].id, 7);
        assert_eq!(next[0].path, "payments/crx-transfers");
        assert_eq!(next[1], held[1]);
        assert!(refreshed(&next, &[project(7, "payments/crx-transfers")]).is_none());
    }

    fn requests_route() -> (&'static str, u16, Value) {
        (
            "/merge_requests?scope=created_by_me",
            200,
            json!([{
                "id": 100, "iid": 12, "project_id": 7, "title": "Retry acks", "author": { "username": "n.hassan" },
                "updated_at": "2026-10-28T10:00:00Z", "source_branch": "retry-acks", "target_branch": "main",
                "reviewers": [], "references": { "full": "payments/crx-transfers!12" }, "web_url": "https://x/12"
            }]),
        )
    }

    fn reviews_route() -> (&'static str, u16, Value) {
        (
            "reviewer_username=n.hassan",
            200,
            json!([{
                "id": 200, "iid": 4, "project_id": 99, "title": "Elsewhere", "author": { "username": "omar" },
                "updated_at": "2026-10-28T09:00:00Z", "source_branch": "b", "target_branch": "main",
                "reviewers": [{ "username": "N.Hassan" }], "references": { "full": "other/thing!4" }
            }]),
        )
    }

    fn ask<'a>(
        followed: &'a [Followed],
        prior: Option<&'a WideHome>,
        now: i64,
        fresh: i64,
    ) -> Ask<'a> {
        Ask {
            host: HOST,
            me: "n.hassan",
            followed,
            prior,
            now,
            fresh,
        }
    }

    #[test]
    fn home_reads_requests_across_the_host_and_runs_and_releases_per_followed_project() {
        let fetch = gitlab(vec![
            requests_route(),
            reviews_route(),
            (
                "/issues?scope=assigned_to_me",
                200,
                json!([{ "id": 5, "iid": 88, "project_id": 9, "title": "Token refresh", "state": "opened",
                         "references": { "full": "channels/mobile-gateway#88" } }]),
            ),
            (
                "/projects/7/pipelines?per_page=1&ref=main",
                200,
                json!([{ "id": 1, "ref": "main", "status": "failed" }]),
            ),
            (
                "/projects/7/pipelines?per_page=1&ref=retry-acks",
                200,
                json!([{ "id": 2, "ref": "retry-acks", "status": "success" }]),
            ),
            (
                "/projects/7/releases",
                200,
                json!([{ "tag_name": "v2.14.0", "name": "2.14" }]),
            ),
        ]);
        let list = [followed(7, "payments/crx-transfers")];

        let home = home_wide(&ask(&list, None, 1_000, 0), &fetch);

        assert_eq!(home.requests.len(), 2);
        let review = home.requests.iter().find(|r| r.item.id == 200).unwrap();
        assert!(
            review.item.review_requested,
            "usernames compare without case"
        );
        assert_eq!(review.project, "other/thing");
        assert!(
            home.requests
                .iter()
                .find(|r| r.item.id == 100)
                .unwrap()
                .item
                .mine
        );
        assert_eq!(home.issues[0].project, "channels/mobile-gateway");
        assert_eq!(
            home.pipelines.len(),
            2,
            "the default branch and the person's own"
        );
        assert_eq!(home.pipelines[0].item.result, "failed");
        assert_eq!(home.pipelines[0].project, "payments/crx-transfers");
        assert_eq!(home.releases[0].item.tag, "v2.14.0");
        assert!(home.errors.is_empty(), "{:?}", home.errors);
        assert_eq!(home.read.get(&7), Some(&1_000));
    }

    #[test]
    fn one_failing_project_is_its_own_error_and_the_others_still_show() {
        let fetch = gitlab(vec![
            requests_route(),
            (
                "/projects/7/pipelines",
                500,
                json!({ "message": "500 Internal Server Error" }),
            ),
            ("/projects/7/releases", 500, json!({})),
            (
                "/projects/9/pipelines",
                200,
                json!([{ "id": 3, "ref": "main", "status": "success" }]),
            ),
            (
                "/projects/9/releases",
                403,
                json!({ "message": "403 Forbidden" }),
            ),
            ("/issues", 200, json!([])),
        ]);
        let list = [
            followed(7, "payments/crx-transfers"),
            followed(9, "cards/api"),
        ];

        let home = home_wide(&ask(&list, None, 1_000, 0), &fetch);

        assert_eq!(home.pipelines.len(), 1);
        assert_eq!(home.pipelines[0].project_id, 9);
        let mine: Vec<_> = home.errors.iter().filter(|e| e.project_id == 7).collect();
        // The default branch, the person's own branch, and the release.
        assert_eq!(mine.len(), 3);
        assert!(mine.iter().any(|e| e.card == "pipelines"));
        assert!(mine.iter().any(|e| e.card == "releases"));
        assert!(
            home.errors.iter().all(|e| e.project_id != 9),
            "403 is no releases, not a failure"
        );
    }

    #[test]
    fn no_more_than_four_requests_are_in_flight() {
        let now = AtomicUsize::new(0);
        let most = AtomicUsize::new(0);
        let fetch = |_: &str| -> Result<Answer> {
            let here = now.fetch_add(1, Ordering::SeqCst) + 1;
            most.fetch_max(here, Ordering::SeqCst);
            std::thread::sleep(std::time::Duration::from_millis(15));
            now.fetch_sub(1, Ordering::SeqCst);
            answer(200, json!([]))
        };
        let list: Vec<Followed> = (1..=12).map(|id| followed(id, "group/p")).collect();

        home_wide(&ask(&list, None, 1_000, 0), &fetch);

        assert!(
            most.load(Ordering::SeqCst) <= IN_FLIGHT,
            "{}",
            most.load(Ordering::SeqCst)
        );
        assert!(most.load(Ordering::SeqCst) > 1, "the reads should overlap");
    }

    #[test]
    fn a_project_read_within_the_last_interval_is_not_read_again() {
        let asked = Mutex::new(Vec::<String>::new());
        let fetch = |url: &str| -> Result<Answer> {
            asked.lock().unwrap().push(url.to_string());
            answer(
                200,
                json!([{ "id": 1, "ref": "main", "status": "success", "tag_name": "v1" }]),
            )
        };
        let list = [followed(7, "a/b"), followed(9, "c/d")];
        let first = home_wide(&ask(&list, None, 1_000, 0), &fetch);
        asked.lock().unwrap().clear();

        let mut prior = first.clone();
        prior.read.insert(9, 100); // read long ago
        let second = home_wide(&ask(&list, Some(&prior), 1_060, 120), &fetch);

        let urls = asked.lock().unwrap().clone();
        assert!(urls.iter().all(|u| !u.contains("/projects/7/")), "{urls:?}");
        assert!(urls.iter().any(|u| u.contains("/projects/9/")), "{urls:?}");
        assert!(
            urls.iter().all(|u| !u.contains("/merge_requests")),
            "host-wide reads were fresh"
        );
        assert_eq!(
            second
                .pipelines
                .iter()
                .filter(|r| r.project_id == 7)
                .count(),
            1
        );
        assert_eq!(second.read[&7], 1_000);
        assert_eq!(second.read[&9], 1_060);
    }

    #[test]
    fn only_followed_projects_are_read_one_by_one_and_no_more_than_fifty() {
        let asked = Mutex::new(Vec::<String>::new());
        let fetch = |url: &str| -> Result<Answer> {
            asked.lock().unwrap().push(url.to_string());
            answer(200, json!([]))
        };
        let list: Vec<Followed> = (1..=60).map(|id| followed(id, "g/p")).collect();
        let home = home_wide(&ask(&list, None, 1, 0), &fetch);
        assert_eq!(home.read.len(), MAX_FOLLOWED);
        let per_project = asked
            .lock()
            .unwrap()
            .iter()
            .filter(|u| u.contains("/projects/"))
            .count();
        assert_eq!(per_project, MAX_FOLLOWED * 2);
    }

    #[test]
    fn nothing_returned_has_a_field_that_could_carry_the_token() {
        let home = WideHome::default();
        let text = serde_json::to_string(&home).unwrap();
        for word in ["token", "secret", "password", "authorization"] {
            assert!(!text.to_lowercase().contains(word), "{text}");
        }
        let page = ProjectPage {
            projects: vec![project(1, "a/b")],
            page: 1,
            more: false,
        };
        let text = serde_json::to_string(&page).unwrap();
        assert!(!text.to_lowercase().contains("token"), "{text}");
    }
}
