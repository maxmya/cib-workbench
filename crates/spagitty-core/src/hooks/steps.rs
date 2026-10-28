// SPDX-License-Identifier: MIT

//! What a repository's hooks run, step by step (FEAT-176).
//!
//! The Hooks screen showed each hook's file and left the reading to the
//! person. This reads the file the way a person would: each command a hook
//! script runs, each command lefthook's configuration lists, each hook a
//! pre-commit configuration names — as a step, numbered in the order it
//! runs within its hook, with the line it is written on and what kind of
//! check it is.
//!
//! # Order
//!
//! Within a hook, a script runs top to bottom; lefthook runs its commands by
//! `priority` where they have one and as written where they do not, all at
//! once when the hook says `parallel: true`; pre-commit runs its hooks as
//! written. Which hook runs first — `pre-commit` before `commit-msg`, both
//! before `pre-push` — is git's own order, and the screen lays the hooks out
//! in it.
//!
//! # Kind
//!
//! A step's category is read from the words of its command and its name:
//! `eslint`, `prettier` and `lint-staged` are formatting and linting,
//! `jest` and `npm test` are tests, `gitleaks` is secrets. The rules are
//! ordered, so `npm run test:lint` is a test and `commitlint` in a
//! `commit-msg` hook is about the message. What no rule knows is *Other*.

use serde::Serialize;
use yaml_rust2::{Yaml, YamlLoader};

use super::{generate, Hooks, Manager, NAMES};

/// What kind of check a step is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Category {
    /// Formatters and linters.
    Format,
    Tests,
    /// Type checks, compiles and builds.
    Build,
    /// The commit message's form.
    Message,
    Secrets,
    /// The files themselves: size, whitespace, conflict markers, valid YAML.
    Files,
    /// Which branch may be committed or pushed to.
    Branches,
    /// Installing what the code needs.
    Dependencies,
    Other,
}

/// One thing a hook runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// The git hook it runs in: `pre-commit`, `commit-msg`, …
    pub hook: String,
    /// Its place in that hook, from 1.
    pub order: u32,
    /// What it is called: a lefthook command's or pre-commit hook's name, or
    /// the program a script line runs.
    pub name: String,
    /// What it runs.
    pub command: String,
    pub category: Category,
    /// The file it is written in, as [`super::Hook::path`] or the
    /// configuration's path.
    pub file: String,
    /// The line it is written on, from 1.
    pub line: Option<u32>,
    /// Runs at the same time as the other steps of its hook.
    pub together: bool,
}

/// Every step the repository's hooks run, hook by hook in the order the
/// hooks are listed, and in each hook in the order the steps run. Hooks
/// Spagitty wrote from blocks are left to the blocks.
pub fn steps(info: &Hooks) -> Vec<Step> {
    let mut out = Vec::new();
    match (info.manager, &info.config) {
        (Manager::Lefthook, Some(config)) => lefthook(&config.path, &config.text, &mut out),
        (Manager::PreCommit, Some(config)) => pre_commit(&config.path, &config.text, &mut out),
        _ => {}
    }
    for hook in &info.hooks {
        if hook.script.lines().any(|line| line == generate::MARK)
            || runs_manager(info.manager, &hook.script)
        {
            continue;
        }
        for (order, (line, command)) in script_steps(&hook.script).into_iter().enumerate() {
            out.push(Step {
                hook: hook.name.clone(),
                order: order as u32 + 1,
                name: program(&command),
                category: category(&hook.name, &command, ""),
                command,
                file: hook.path.clone(),
                line: Some(line),
                together: false,
            });
        }
    }
    out
}

/// A lefthook or pre-commit stub: what it runs is in the configuration.
fn runs_manager(manager: Manager, script: &str) -> bool {
    match manager {
        Manager::Lefthook => script.contains("lefthook"),
        Manager::PreCommit => script.contains("pre-commit") || script.contains("pre_commit"),
        Manager::Git | Manager::Husky => false,
    }
}

/// The commands a shell script runs, with their lines: what is left once
/// the shebang, comments, blank lines, the shell's own words (`then`, `fi`),
/// settings (`set -e`, `export`), messages (`echo`) and a manager's own
/// boilerplate are taken out. A line ending in `\` is joined to the next.
pub fn script_steps(script: &str) -> Vec<(u32, String)> {
    let mut out = Vec::new();
    let mut pending: Option<(u32, String)> = None;
    for (index, raw) in script.lines().enumerate() {
        let number = index as u32 + 1;
        let line = raw.trim();
        let (start, mut text) = match pending.take() {
            Some((start, before)) => (start, format!("{before} {line}")),
            None => (number, line.to_string()),
        };
        if let Some(open) = text.strip_suffix('\\') {
            pending = Some((start, open.trim_end().to_string()));
            continue;
        }
        text = command_of(&text);
        if !text.is_empty() {
            out.push((start, text));
        }
    }
    if let Some((start, text)) = pending {
        let text = command_of(&text);
        if !text.is_empty() {
            out.push((start, text));
        }
    }
    out
}

/// The command on one line, or nothing when the line runs none.
fn command_of(line: &str) -> String {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return String::new();
    }
    // Husky's helper, in every version's spelling.
    if line.contains("husky.sh") || line.ends_with("/h\"") || line.contains("_/h\"") {
        return String::new();
    }
    let first = line.split_whitespace().next().unwrap_or("");
    const STRUCTURE: &[&str] = &[
        "then", "fi", "else", "do", "done", "esac", ";;", "{", "}", "(", ")", "set", "export",
        "local", "readonly", "unset", "shift", "return", "exit", "echo", "printf", "cd", "true",
        "false", ":", "trap", "source", ".", "case", "function",
    ];
    if STRUCTURE.contains(&first)
        || first.ends_with("()")
        || first.ends_with(')') && !line.contains(' ')
    {
        return String::new();
    }
    // An assignment alone is not a step: `FILES=$(git diff …)`.
    if first.contains('=') && !first.starts_with('-') && line.split_whitespace().count() == 1 {
        return String::new();
    }
    let mut text = line.to_string();
    // `if ! npm test; then` runs `npm test`.
    for prefix in ["if ", "elif ", "while ", "until "] {
        if let Some(rest) = text.strip_prefix(prefix) {
            text = rest.trim().to_string();
        }
    }
    if let Some(rest) = text.strip_prefix("! ") {
        text = rest.trim().to_string();
    }
    for suffix in ["; then", ";then", "; do", ";do"] {
        if let Some(rest) = text.strip_suffix(suffix) {
            text = rest.trim().to_string();
        }
    }
    let first = text.split_whitespace().next().unwrap_or("");
    if first.starts_with('[') || first == "test" && text.contains(" -") {
        // A condition, not a command.
        return String::new();
    }
    if ["echo", "printf", "exit"].contains(&first) {
        return String::new();
    }
    text
}

/// The program a command runs, as a short name: `npx lint-staged` is
/// `lint-staged`, `npm run test:unit` is `test:unit`, `./mvnw verify` is
/// `mvnw verify`.
pub fn program(command: &str) -> String {
    let words: Vec<&str> = command
        .split_whitespace()
        .filter(|word| !word.contains('=') || word.starts_with('-'))
        .collect();
    let mut at = 0;
    while at < words.len()
        && ["npx", "exec", "env", "command", "sudo", "time", "dotenv"].contains(&words[at])
    {
        at += 1;
    }
    let Some(first) = words.get(at) else {
        return command.to_string();
    };
    let base = first.rsplit(['/', '\\']).next().unwrap_or(first);
    let next = words.get(at + 1).copied();
    match base {
        "npm" | "pnpm" | "yarn" | "bun" => match next {
            Some("run") | Some("run-script") | Some("exec") | Some("dlx") => words
                .get(at + 2)
                .map(|word| word.to_string())
                .unwrap_or_else(|| base.to_string()),
            Some(word) if !word.starts_with('-') => format!("{base} {word}"),
            _ => base.to_string(),
        },
        "mvnw" | "mvn" | "gradlew" | "gradle" | "cargo" | "go" | "dotnet" | "make" | "uv"
        | "poetry" => match next {
            Some(word) if !word.starts_with('-') => format!("{base} {word}"),
            _ => base.to_string(),
        },
        _ => base.to_string(),
    }
}

/// The words a command is made of, lower-cased, without quotes or shell
/// punctuation.
fn words_of(text: &str) -> Vec<String> {
    text.to_ascii_lowercase()
        .split(|c: char| c.is_whitespace() || "\"'`;|&(){}<>=,".contains(c))
        .filter(|word| !word.is_empty())
        .map(|word| word.rsplit(['/', '\\']).next().unwrap_or(word).to_string())
        .collect()
}

/// What kind of step `command` is, run from `hook`, called `name`.
pub fn category(hook: &str, command: &str, name: &str) -> Category {
    let text = format!("{command} {name}").to_ascii_lowercase();
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let words = words_of(&text);
    let word = |wanted: &[&str]| {
        words.iter().any(|word| {
            wanted.iter().any(|want| {
                word == want
                    || word.starts_with(&format!("{want}:"))
                    || word.starts_with(&format!("{want}-"))
                    || word.ends_with(&format!(":{want}"))
            })
        })
    };
    let phrase = |wanted: &[&str]| wanted.iter().any(|want| flat.contains(want));

    if word(&[
        "gitleaks",
        "detect-secrets",
        "trufflehog",
        "git-secrets",
        "secretlint",
        "ggshield",
        "talisman",
        "detect-private-key",
        "detect-aws-credentials",
        "secrets",
    ]) {
        return Category::Secrets;
    }
    if word(&[
        "commitlint",
        "commitizen",
        "gitlint",
        "conventional-pre-commit",
        "commit-msg-linter",
        "validate-commit-msg",
    ]) || phrase(&["cz check"])
        || (hook == "commit-msg" || hook == "prepare-commit-msg")
    {
        return Category::Message;
    }
    if phrase(&[
        "npm install",
        "npm ci",
        "npm i ",
        "pnpm install",
        "pnpm i ",
        "yarn install",
        "bun install",
        "pip install",
        "poetry install",
        "uv sync",
        "bundle install",
        "composer install",
        "go mod download",
        "go mod tidy",
    ]) || flat == "yarn"
        || flat.ends_with(" npm i")
        || flat.ends_with(" pnpm i")
        || flat == "npm i"
        || flat == "pnpm i"
    {
        return Category::Dependencies;
    }
    if word(&[
        "check-added-large-files",
        "trailing-whitespace",
        "end-of-file-fixer",
        "check-yaml",
        "check-json",
        "check-toml",
        "check-xml",
        "check-merge-conflict",
        "check-case-conflict",
        "mixed-line-ending",
        "check-symlinks",
        "check-executables-have-shebangs",
        "large-files",
        "editorconfig-checker",
    ]) {
        return Category::Files;
    }
    if word(&[
        "no-commit-to-branch",
        "protect-branch",
        "protected-branches",
        "branch-name",
        "validate-branch-name",
    ]) || phrase(&[
        "rev-parse --abbrev-ref",
        "symbolic-ref",
        "branch --show-current",
    ]) {
        return Category::Branches;
    }
    if word(&[
        "test",
        "tests",
        "jest",
        "vitest",
        "mocha",
        "pytest",
        "karma",
        "cypress",
        "playwright",
        "phpunit",
        "rspec",
        "ava",
        "tap",
        "unit",
        "e2e",
        "spec",
        "verify",
        "nextest",
    ]) {
        return Category::Tests;
    }
    if word(&[
        "tsc",
        "typecheck",
        "type-check",
        "types",
        "mypy",
        "pyright",
        "build",
        "compile",
        "vue-tsc",
        "assemble",
        "check-types",
    ]) || phrase(&["cargo check", "go build", "mvn compile", "mvnw compile"])
    {
        return Category::Build;
    }
    if word(&[
        "prettier",
        "eslint",
        "lint-staged",
        "lint",
        "stylelint",
        "ruff",
        "black",
        "isort",
        "flake8",
        "pylint",
        "gofmt",
        "goimports",
        "golangci-lint",
        "rustfmt",
        "fmt",
        "clippy",
        "spotless",
        "ktlint",
        "checkstyle",
        "biome",
        "format",
        "markdownlint",
        "shellcheck",
        "shfmt",
        "rubocop",
        "standard",
        "xo",
        "pretty-quick",
        "dprint",
        "oxlint",
        "vet",
    ]) {
        return Category::Format;
    }
    Category::Other
}

/// The line `key:` is on, from 1, at or after `from` (a 0-based line).
fn line_of(text: &str, key: &str, from: usize) -> Option<u32> {
    text.lines()
        .enumerate()
        .skip(from)
        .find(|(_, line)| {
            let trimmed = line.trim_start().trim_start_matches("- ").trim_start();
            trimmed.starts_with(&format!("{key}:"))
                || trimmed.starts_with(&format!("\"{key}\":"))
                || trimmed.starts_with(&format!("'{key}':"))
        })
        .map(|(at, _)| at as u32 + 1)
}

/// The line `id: <id>` or `- id: <id>` is on, from 1.
fn id_line(text: &str, id: &str) -> Option<u32> {
    text.lines()
        .position(|line| {
            let trimmed = line.trim_start().trim_start_matches("- ").trim_start();
            trimmed
                .strip_prefix("id:")
                .map(|rest| rest.trim().trim_matches(['"', '\'']) == id)
                .unwrap_or(false)
        })
        .map(|at| at as u32 + 1)
}

fn lefthook(file: &str, text: &str, out: &mut Vec<Step>) {
    if file.ends_with(".toml") {
        return;
    }
    let Ok(documents) = YamlLoader::load_from_str(text) else {
        return;
    };
    let Some(document) = documents.first() else {
        return;
    };
    for hook in NAMES {
        let settings = &document[*hook];
        if settings.is_badvalue() || settings.is_null() {
            continue;
        }
        let hook_line = line_of(text, hook, 0)
            .map(|line| line as usize)
            .unwrap_or(0);
        let together = settings["parallel"].as_bool().unwrap_or(false);
        let mut found: Vec<(i64, usize, Step)> = Vec::new();
        if let Some(commands) = settings["commands"].as_hash() {
            for (index, (name, command)) in commands.iter().enumerate() {
                let Some(name) = name.as_str() else { continue };
                let run = command["run"].as_str().unwrap_or("").trim().to_string();
                if run.is_empty() || command["skip"].as_bool() == Some(true) {
                    continue;
                }
                let priority = command["priority"].as_i64().unwrap_or(i64::MAX);
                found.push((
                    priority,
                    index,
                    Step {
                        hook: hook.to_string(),
                        order: 0,
                        name: name.to_string(),
                        category: category(hook, &run, name),
                        command: run,
                        file: file.to_string(),
                        line: line_of(text, name, hook_line),
                        together,
                    },
                ));
            }
        }
        if let Some(scripts) = settings["scripts"].as_hash() {
            for (index, (name, script)) in scripts.iter().enumerate() {
                let Some(name) = name.as_str() else { continue };
                let runner = script["runner"].as_str().unwrap_or("sh");
                let priority = script["priority"].as_i64().unwrap_or(i64::MAX);
                found.push((
                    priority,
                    1000 + index,
                    Step {
                        hook: hook.to_string(),
                        order: 0,
                        name: name.to_string(),
                        command: format!("{runner} .lefthook/{hook}/{name}"),
                        category: category(hook, name, name),
                        file: file.to_string(),
                        line: line_of(text, name, hook_line),
                        together,
                    },
                ));
            }
        }
        found.sort_by_key(|(priority, index, _)| (*priority, *index));
        for (order, (_, _, mut step)) in found.into_iter().enumerate() {
            step.order = order as u32 + 1;
            out.push(step);
        }
    }
}

/// pre-commit's stage names, old and new, as git's hook names.
fn hook_of_stage(stage: &str) -> Option<&'static str> {
    Some(match stage {
        "pre-commit" | "commit" => "pre-commit",
        "pre-push" | "push" => "pre-push",
        "commit-msg" => "commit-msg",
        "prepare-commit-msg" => "prepare-commit-msg",
        "post-commit" => "post-commit",
        "post-checkout" => "post-checkout",
        "post-merge" => "post-merge",
        "pre-merge-commit" | "merge-commit" => "pre-merge-commit",
        "pre-rebase" => "pre-rebase",
        "post-rewrite" => "post-rewrite",
        _ => return None,
    })
}

fn strings(value: &Yaml) -> Vec<String> {
    value
        .as_vec()
        .map(|values| {
            values
                .iter()
                .filter_map(Yaml::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn pre_commit(file: &str, text: &str, out: &mut Vec<Step>) {
    let Ok(documents) = YamlLoader::load_from_str(text) else {
        return;
    };
    let Some(document) = documents.first() else {
        return;
    };
    let defaults = {
        let stages = strings(&document["default_stages"]);
        if stages.is_empty() {
            vec!["pre-commit".to_string()]
        } else {
            stages
        }
    };
    let mut by_hook: Vec<(String, Vec<Step>)> = Vec::new();
    for repository in document["repos"].as_vec().into_iter().flatten() {
        let source = repository["repo"].as_str().unwrap_or("");
        for hook in repository["hooks"].as_vec().into_iter().flatten() {
            let Some(id) = hook["id"].as_str() else {
                continue;
            };
            let name = hook["name"].as_str().unwrap_or(id).to_string();
            let command = match hook["entry"].as_str() {
                Some(entry) => entry.trim().to_string(),
                None => {
                    let short = source
                        .trim_end_matches(".git")
                        .rsplit('/')
                        .next()
                        .unwrap_or(source);
                    format!("{id} ({short})")
                }
            };
            let stages = {
                let own = strings(&hook["stages"]);
                if own.is_empty() {
                    defaults.clone()
                } else {
                    own
                }
            };
            for stage in stages {
                let Some(git_hook) = hook_of_stage(&stage) else {
                    continue;
                };
                let step = Step {
                    hook: git_hook.to_string(),
                    order: 0,
                    name: name.clone(),
                    category: category(git_hook, &command, &format!("{id} {name}")),
                    command: command.clone(),
                    file: file.to_string(),
                    line: id_line(text, id),
                    together: false,
                };
                match by_hook.iter_mut().find(|(hook, _)| hook == git_hook) {
                    Some((_, steps)) => steps.push(step),
                    None => by_hook.push((git_hook.to_string(), vec![step])),
                }
            }
        }
    }
    for (_, steps) in by_hook {
        for (order, mut step) in steps.into_iter().enumerate() {
            step.order = order as u32 + 1;
            out.push(step);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{Config, Hook};

    fn hooks(manager: Manager, hooks: Vec<Hook>, config: Option<Config>) -> Hooks {
        Hooks {
            dir: ".git/hooks".into(),
            manager,
            config,
            hooks,
            enabled: true,
            steps: Vec::new(),
        }
    }

    fn hook(name: &str, path: &str, script: &str) -> Hook {
        Hook {
            name: name.into(),
            path: path.into(),
            script: script.into(),
            truncated: false,
            on_commit: true,
        }
    }

    #[test]
    fn a_script_is_its_commands_with_their_lines() {
        let script = "#!/usr/bin/env sh\n. \"$(dirname -- \"$0\")/_/husky.sh\"\n\n# lint first\nset -e\necho \"checking\"\nnpx lint-staged\nif ! npm test; then\n  exit 1\nfi\n./gradlew \\\n  spotlessCheck\n";
        let steps = script_steps(script);
        assert_eq!(
            steps,
            [
                (7, "npx lint-staged".to_string()),
                (8, "npm test".to_string()),
                (11, "./gradlew spotlessCheck".to_string())
            ]
        );
    }

    #[test]
    fn a_step_is_named_for_the_program_it_runs() {
        assert_eq!(program("npx lint-staged"), "lint-staged");
        assert_eq!(program("npm run test:unit"), "test:unit");
        assert_eq!(program("pnpm typecheck"), "pnpm typecheck");
        assert_eq!(program("./mvnw -q verify"), "mvnw");
        assert_eq!(program("CI=1 cargo test --all"), "cargo test");
    }

    #[test]
    fn a_step_is_sorted_by_what_its_words_say_it_does() {
        assert_eq!(
            category("pre-commit", "npx lint-staged", ""),
            Category::Format
        );
        assert_eq!(
            category("pre-commit", "npx eslint --fix .", ""),
            Category::Format
        );
        assert_eq!(category("pre-push", "npm test", ""), Category::Tests);
        assert_eq!(
            category("pre-push", "npm run test:e2e", ""),
            Category::Tests
        );
        assert_eq!(
            category("pre-commit", "npx tsc --noEmit", ""),
            Category::Build
        );
        assert_eq!(
            category("commit-msg", "npx --no -- commitlint --edit $1", ""),
            Category::Message
        );
        assert_eq!(
            category("pre-commit", "gitleaks protect --staged", ""),
            Category::Secrets
        );
        assert_eq!(category("post-merge", "npm ci", ""), Category::Dependencies);
        assert_eq!(
            category("pre-commit", "check-added-large-files", ""),
            Category::Files
        );
        assert_eq!(
            category("pre-commit", "no-commit-to-branch", ""),
            Category::Branches
        );
        assert_eq!(
            category("pre-commit", "./scripts/do-thing.sh", ""),
            Category::Other
        );
    }

    #[test]
    fn husky_and_plain_hooks_are_read_script_by_script_and_spagittys_own_are_left_to_their_blocks()
    {
        let info = hooks(
            Manager::Husky,
            vec![
                hook(
                    "pre-commit",
                    ".husky/pre-commit",
                    "npx lint-staged\nnpx tsc --noEmit\n",
                ),
                hook(
                    "commit-msg",
                    ".husky/commit-msg",
                    "npx --no -- commitlint --edit \"$1\"\n",
                ),
                hook(
                    "pre-push",
                    ".git/hooks/pre-push",
                    &format!("#!/bin/sh\n{}\nrun\n", generate::MARK),
                ),
            ],
            None,
        );
        let steps = steps(&info);
        let seen: Vec<(&str, u32, Category)> = steps
            .iter()
            .map(|step| (step.hook.as_str(), step.order, step.category))
            .collect();
        assert_eq!(
            seen,
            [
                ("pre-commit", 1, Category::Format),
                ("pre-commit", 2, Category::Build),
                ("commit-msg", 1, Category::Message)
            ]
        );
        assert_eq!(steps[1].line, Some(2));
        assert_eq!(steps[1].file, ".husky/pre-commit");
    }

    #[test]
    fn lefthook_runs_by_priority_then_as_written_and_says_when_its_steps_run_together() {
        let config = "pre-commit:\n  parallel: true\n  commands:\n    lint:\n      run: npx eslint {staged_files}\n    types:\n      priority: 1\n      run: npx tsc --noEmit\npre-push:\n  commands:\n    test:\n      run: npm test\n";
        let info = hooks(
            Manager::Lefthook,
            vec![hook(
                "pre-commit",
                ".git/hooks/pre-commit",
                "#!/bin/sh\nlefthook run pre-commit\n",
            )],
            Some(Config {
                path: "lefthook.yml".into(),
                text: config.into(),
            }),
        );
        let steps = steps(&info);
        let seen: Vec<(&str, u32, &str, bool)> = steps
            .iter()
            .map(|step| {
                (
                    step.hook.as_str(),
                    step.order,
                    step.name.as_str(),
                    step.together,
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                ("pre-commit", 1, "types", true),
                ("pre-commit", 2, "lint", true),
                ("pre-push", 1, "test", false)
            ]
        );
        assert_eq!(steps[0].line, Some(6));
        assert_eq!(steps[2].line, Some(11));
    }

    #[test]
    fn pre_commit_reads_each_hook_into_the_stages_it_runs_in() {
        let config = "default_stages: [pre-commit]\nrepos:\n  - repo: https://github.com/pre-commit/pre-commit-hooks\n    rev: v4.6.0\n    hooks:\n      - id: trailing-whitespace\n      - id: check-added-large-files\n  - repo: local\n    hooks:\n      - id: unit\n        name: Unit tests\n        entry: pytest -q\n        language: system\n        stages: [pre-push]\n";
        let info = hooks(
            Manager::PreCommit,
            Vec::new(),
            Some(Config {
                path: ".pre-commit-config.yaml".into(),
                text: config.into(),
            }),
        );
        let steps = steps(&info);
        let seen: Vec<(&str, u32, &str, Category)> = steps
            .iter()
            .map(|step| {
                (
                    step.hook.as_str(),
                    step.order,
                    step.name.as_str(),
                    step.category,
                )
            })
            .collect();
        assert_eq!(
            seen,
            [
                ("pre-commit", 1, "trailing-whitespace", Category::Files),
                ("pre-commit", 2, "check-added-large-files", Category::Files),
                ("pre-push", 1, "Unit tests", Category::Tests)
            ]
        );
        assert_eq!(steps[2].command, "pytest -q");
        assert_eq!(steps[2].line, Some(10));
        assert_eq!(steps[0].command, "trailing-whitespace (pre-commit-hooks)");
    }
}
