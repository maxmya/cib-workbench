// SPDX-License-Identifier: MIT

//! Hooks' builder and run records (FEAT-137). The commands hold no session
//! lock while a hook runs; the repository watcher carries new output.

use crate::commands::AppState;
use serde::Serialize;
use spagitty_core::hooks::{
    self,
    blocks::{Action, Definition, Stage},
    builder,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use tauri::{AppHandle, Manager, Runtime, State};

type Result<T> = std::result::Result<T, String>;

fn repository(state: &AppState, command: &'static str) -> Result<PathBuf> {
    state.open_path(command).map_err(|e| e.to_string())
}

fn personal<R: Runtime>(app: &AppHandle<R>, repo: &Path) -> Result<PathBuf> {
    let name = format!(
        "{:016x}",
        hooks::generate::hash(repo.to_string_lossy().as_bytes())
    );
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("hooks")
        .join(name)
        .join("hooks.toml"))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Builder {
    definition: Definition,
    info: hooks::Hooks,
    runs: Vec<builder::Run>,
    staged: Vec<String>,
    installed: bool,
}

#[tauri::command(async)]
pub fn hooks_builder<R: Runtime>(app: AppHandle<R>, state: State<'_, AppState>) -> Result<Builder> {
    let repo = repository(&state, "hooks_builder")?;
    let definition = builder::load(&repo, &personal(&app, &repo)?)?;
    let info = hooks::list(&repo).map_err(|e| e.to_string())?;
    let installed = builder::installed(&repo, &definition)?;
    Ok(Builder {
        definition,
        info,
        runs: builder::runs(&repo)?,
        staged: spagitty_core::shell::hook_matches(&repo, &[]).map_err(|e| e.to_string())?,
        installed,
    })
}

#[tauri::command(async)]
pub fn hooks_save<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    definition: Definition,
    install: bool,
    imported: bool,
) -> Result<()> {
    let repo = repository(&state, "hooks_save")?;
    static SAVES: std::sync::Mutex<()> = std::sync::Mutex::new(());
    let _guard = SAVES.lock().map_err(|e| e.to_string())?;
    // Preflight ownership before saving: a refused installation must not
    // quietly save a different definition behind the screen's error.
    if install {
        builder::preflight(&repo, &definition, true, imported)?;
    }
    builder::save(&repo, &personal(&app, &repo)?, &definition)?;
    if install {
        builder::scripts(&repo, &definition, true, imported)
            .map_err(|e| format!("The definition was saved, but hooks were not installed: {e}"))?;
    }
    Ok(())
}

#[tauri::command(async)]
pub fn hooks_import(state: State<'_, AppState>) -> Result<Definition> {
    builder::import(&repository(&state, "hooks_import")?)
}

#[tauri::command(async)]
pub fn hooks_matches(state: State<'_, AppState>, paths: Vec<String>) -> Result<Vec<String>> {
    spagitty_core::shell::hook_matches(&repository(&state, "hooks_matches")?, &paths)
        .map_err(|e| e.to_string())
}

#[tauri::command(async)]
pub fn hooks_runs(state: State<'_, AppState>) -> Result<Vec<builder::Run>> {
    builder::runs(&repository(&state, "hooks_runs")?)
}

#[tauri::command(async)]
pub fn hooks_run(state: State<'_, AppState>, id: String) -> Result<builder::Run> {
    builder::run(&repository(&state, "hooks_run")?, &id)
}

#[tauri::command(async)]
pub fn hooks_log(state: State<'_, AppState>, id: String, block: String) -> Result<String> {
    builder::log(&repository(&state, "hooks_log")?, &id, &block)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tried {
    passed: bool,
    output: String,
}

#[tauri::command(async)]
pub fn hooks_dry_run(
    state: State<'_, AppState>,
    definition: Definition,
    stage: Stage,
) -> Result<Tried> {
    if stage == Stage::CommitMsg {
        return Err("Use Try a message for commit-msg.".into());
    }
    let repo = repository(&state, "hooks_dry_run")?;
    let directory = builder::scripts(&repo, &definition, false, false)?;
    Ok(tried(spagitty_core::shell::hook_run(
        &repo,
        &directory,
        stage.name(),
        None,
    )))
}

fn tried(result: spagitty_core::Result<String>) -> Tried {
    match result {
        Ok(output) => Tried {
            passed: true,
            output,
        },
        Err(e) => Tried {
            passed: false,
            output: e.to_string(),
        },
    }
}

#[tauri::command(async)]
pub fn hooks_try_message(
    state: State<'_, AppState>,
    definition: Definition,
    message: String,
) -> Result<Tried> {
    if message.len() > 65536 || message.contains('\0') {
        return Err("The message is too long or contains a NUL character.".into());
    }
    let repo = repository(&state, "hooks_try_message")?;
    let mut definition = definition;
    definition.blocks.retain(|block| {
        block.stage == Stage::CommitMsg && matches!(block.action, Action::Message { .. })
    });
    if definition.blocks.is_empty() {
        return Err("This definition has no commit-message rule to try.".into());
    }
    let directory = builder::scripts(&repo, &definition, false, false)?;
    static NEXT: AtomicU64 = AtomicU64::new(1);
    let temporary = builder::folder(&repo)?.join(format!(
        "message-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&temporary).map_err(|e| e.to_string())?;
    let file = temporary.join("try-message");
    std::fs::write(&file, message).map_err(|e| e.to_string())?;
    let result = tried(spagitty_core::shell::hook_run(
        &repo,
        &directory,
        "commit-msg",
        Some(&file),
    ));
    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_dir(&temporary);
    Ok(result)
}

#[tauri::command(async)]
pub fn hooks_stop(state: State<'_, AppState>, id: String) -> Result<()> {
    builder::stop(&repository(&state, "hooks_stop")?, &id)
}

#[tauri::command(async)]
pub fn hooks_undo_restage(state: State<'_, AppState>, id: String) -> Result<()> {
    let repo = repository(&state, "hooks_undo_restage")?;
    let run = builder::run(&repo, &id)?;
    if run.exit.is_none() {
        return Err("Wait until the hook run ends before undoing its restage.".into());
    }
    for event in run
        .events
        .iter()
        .rev()
        .filter(|event| event["t"] == "restaged")
    {
        let field = |name: &str| {
            event[name]
                .as_str()
                .ok_or_else(|| "This record has no restage backup.".to_string())
        };
        let file = event["files"][0]
            .as_str()
            .ok_or("This record has no restaged path.")?;
        spagitty_core::shell::hook_restore_index(
            &repo,
            file,
            field("before")?,
            field("after")?,
            field("mode")?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{commands, testing};
    use spagitty_core::{
        fixture::Fixture,
        hooks::blocks::{Action, Block},
    };

    #[test]
    fn every_read_and_write_requires_an_open_repository() {
        let app = testing::app();
        assert!(hooks_builder(app.handle().clone(), app.state()).is_err());
        assert!(hooks_runs(app.state()).is_err());
        assert!(hooks_import(app.state()).is_err());
        assert!(hooks_matches(app.state(), Vec::new()).is_err());
    }

    #[test]
    fn the_commands_install_dry_run_and_try_the_generated_message_rules() {
        let fixture = Fixture::linear(1);
        let app = testing::app();
        commands::open_repo(
            app.handle().clone(),
            app.state(),
            fixture.path().to_path_buf(),
            None,
        )
        .expect("open");
        let mut definition = Definition::default();
        definition.blocks.push(Block {
            id: "message".into(),
            label: "Message".into(),
            stage: Stage::CommitMsg,
            enabled: true,
            warn: false,
            folder: ".".into(),
            timeout_seconds: 30,
            skip_markdown: false,
            skip_wip: false,
            action: Action::Message {
                style: "conventional".into(),
                types: vec!["feat".into(), "fix".into()],
                scope_required: false,
                subject_max: 72,
                pattern: String::new(),
                ticket_pattern: String::new(),
            },
        });
        hooks_save(
            app.handle().clone(),
            app.state(),
            definition.clone(),
            true,
            false,
        )
        .expect("save");
        assert!(
            hooks_builder(app.handle().clone(), app.state())
                .expect("builder")
                .installed
        );
        assert!(
            hooks_try_message(app.state(), definition.clone(), "feat: add a thing".into())
                .expect("try")
                .passed
        );
        assert!(
            !hooks_try_message(app.state(), definition.clone(), "wrong".into())
                .expect("try")
                .passed
        );
        let mut strict = definition.clone();
        if let Action::Message {
            scope_required,
            ticket_pattern,
            ..
        } = &mut strict.blocks[0].action
        {
            *scope_required = true;
            *ticket_pattern = "^Refs: APP-[0-9]+$".into();
        }
        for (text, passed) in [
            ("feat(api): add a thing\n\nRefs: APP-12", true),
            ("feat: add a thing\n\nRefs: APP-12", false),
            (
                "feat(api): add a thing\n\nRefs: APP-12\n\nFurther explanation",
                false,
            ),
        ] {
            assert_eq!(
                hooks_try_message(app.state(), strict.clone(), text.into())
                    .expect("strict message")
                    .passed,
                passed,
                "{text}"
            );
        }
        assert!(
            hooks_dry_run(app.state(), definition, Stage::PreCommit)
                .expect("dry run")
                .passed
        );
        commands::close_repo(app.state());
    }
}

// --- Reading, explaining and editing what a manager runs (FEAT-171) --------

/// The most a hook file written from the screen may hold.
const MAX_WRITE: usize = 256 * 1024;

/// A file the screen may explain or write: one of the repository's hook
/// scripts, or its manager's configuration, and nothing else. Answers the
/// file and what it is, for the agent's prompt.
fn hook_file(repo: &Path, path: &str) -> Result<(PathBuf, String, hooks::Hooks)> {
    let info = hooks::list(repo).map_err(|e| e.to_string())?;
    let what = info
        .hooks
        .iter()
        .find(|hook| hook.path == path)
        .map(|hook| format!("the {} hook", hook.name))
        .or_else(|| {
            info.config
                .as_ref()
                .filter(|config| config.path == path)
                .map(|_| "the hook manager's configuration".to_string())
        })
        .ok_or_else(|| format!("{path} is not one of this repository's hook files"))?;
    let full = if Path::new(path).is_absolute() {
        PathBuf::from(path)
    } else {
        repo.join(path)
    };
    Ok((full, what, info))
}

/// An agent the screen can ask, by id, with its name.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Askable {
    id: String,
    name: String,
}

/// The agents this repository allows and that can be reached.
#[tauri::command(async)]
pub fn hooks_agents<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
) -> Result<Vec<Askable>> {
    let repo = repository(&state, "hooks_agents")?;
    Ok(crate::agents::askable(&app, &repo.to_string_lossy())
        .into_iter()
        .map(|id| Askable {
            name: crate::agents::name_of(&app, &id).unwrap_or_else(|| id.clone()),
            id,
        })
        .collect())
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Explanation {
    /// Who explained it, by name.
    agent: String,
    text: String,
}

/// The question an agent is asked about a hook file.
fn explain_prompt(what: &str, path: &str, manager: &str, text: &str) -> String {
    format!(
        "Explain {what} in this git repository to someone who did not write it. \
         It is {path}, and {manager} runs it.\n\n\
         In short, plain sentences: when it runs; what each command in it does; \
         what makes it stop a commit or a push; and anything that would surprise \
         the person committing. End with one line on how to skip it once, if it \
         can be skipped. No headings, no preamble.\n\n\
         ```\n{text}\n```"
    )
}

/// Ask an agent what a hook file does. It is given the file and runs nothing.
#[tauri::command(async)]
pub fn hooks_explain<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, AppState>,
    path: String,
    agent: Option<String>,
) -> Result<Explanation> {
    let repo = repository(&state, "hooks_explain")?;
    let (full, what, info) = hook_file(&repo, &path)?;
    let text =
        std::fs::read_to_string(&full).map_err(|e| format!("{path} could not be read: {e}"))?;
    let key = repo.to_string_lossy().into_owned();
    let id = match agent {
        Some(id) => id,
        None => crate::agents::askable(&app, &key).into_iter().next().ok_or(
            "No agent can be asked here. Set one up in Settings › Agents, and allow it for this repository.",
        )?,
    };
    let manager = match info.manager {
        hooks::Manager::Git => "git",
        hooks::Manager::Husky => "Husky",
        hooks::Manager::Lefthook => "lefthook",
        hooks::Manager::PreCommit => "pre-commit",
    };
    let said = crate::agents::ask(
        &app,
        &key,
        &id,
        "You explain one file. You run no command and edit no file.",
        &explain_prompt(&what, &path, manager, &text),
    )?;
    Ok(Explanation {
        agent: crate::agents::name_of(&app, &id).unwrap_or(id),
        text: said.trim().to_string(),
    })
}

/// Write a hook file as edited on the screen. Its permissions stay as they
/// were, so a script that ran still runs.
#[tauri::command(async)]
pub fn hooks_write(state: State<'_, AppState>, path: String, text: String) -> Result<()> {
    let repo = repository(&state, "hooks_write")?;
    let (full, _, _) = hook_file(&repo, &path)?;
    if text.len() > MAX_WRITE {
        return Err(format!(
            "{path} would be over 256 KiB; edit it in your editor"
        ));
    }
    if !full.is_file() {
        return Err(format!("{path} is not there any more"));
    }
    std::fs::write(&full, text).map_err(|e| format!("{path} could not be written: {e}"))
}

#[cfg(test)]
mod managed_tests {
    use super::*;

    #[test]
    fn the_prompt_names_the_file_and_carries_it() {
        let prompt = explain_prompt(
            "the pre-commit hook",
            ".husky/pre-commit",
            "Husky",
            "npx lint-staged",
        );
        assert!(prompt.contains(".husky/pre-commit"), "{prompt}");
        assert!(prompt.contains("Husky runs it"), "{prompt}");
        assert!(prompt.contains("```\nnpx lint-staged\n```"), "{prompt}");
    }

    #[test]
    fn only_a_hook_file_may_be_explained_or_written() {
        let dir = std::env::temp_dir().join(format!("spagitty-hook-file-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("dir");
        let ok = std::process::Command::new("git")
            .args(["init", "-q"])
            .current_dir(&dir)
            .status()
            .is_ok_and(|status| status.success());
        if !ok {
            return;
        }
        let refused = hook_file(&dir, "Cargo.toml").expect_err("not a hook file");
        assert!(
            refused.contains("not one of this repository's hook files"),
            "{refused}"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
