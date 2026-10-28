// SPDX-License-Identifier: MIT

//! Workbench installs its own updates (FEAT-187).
//!
//! FEAT-054 only said that a newer release existed; replacing the binary was
//! somebody else's job. This finishes the sentence: the update is checked,
//! downloaded in the background, and installed when the person presses
//! *Restart* — never mid-work, and never without asking.
//!
//! # Where updates come from, and why they can be trusted
//!
//! `latest.json` on this repository's latest GitHub release, which gate 6
//! writes and attaches. The endpoint and the public key are in
//! `tauri.conf.json` (`plugins.updater`), compiled into the binary: neither is
//! a setting, for the reason `spagitty_core::update` gives — an update source
//! that can be pointed elsewhere is a way to hand somebody a different
//! program. Every download is verified against that key before a byte of it
//! is installed; a release whose signature does not match is refused.
//!
//! # What cannot update itself
//!
//! [`blocker`] decides, before any request: a development build has no release
//! to be behind; a build with no key cannot verify what it downloads; and a
//! binary that was not installed by one of Workbench's bundles (the bare
//! `.exe`, `cargo run`) has nothing an installer can replace. Those fall back
//! to FEAT-054's notice and its link.
//!
//! Called from Rust only, like the notification plugin: the webview has no
//! updater permission, only these two commands.

use serde::Serialize;
use spagitty_core::{Error, Result};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};
use tauri::{ipc::Channel, utils::config::BundleType, AppHandle, Manager, Runtime, State};
use tauri_plugin_updater::UpdaterExt;

/// Where the downloaded update waits for the restart.
#[derive(Default)]
pub struct UpdaterState {
    ready: Mutex<Option<Ready>>,
    /// One check and download at a time: startup and *Check now* can race.
    busy: AtomicBool,
}

struct Ready {
    update: tauri_plugin_updater::Update,
    bytes: Vec<u8>,
}

/// What preparing found.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum Prepared {
    /// This build cannot install updates; the reason says why.
    Unsupported { reason: String },
    /// Nothing newer than the version running.
    Current { version: String },
    /// Downloaded, verified, and waiting for a restart.
    Ready { version: String, current: String },
}

/// How much of the download has arrived.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub received: u64,
    pub total: Option<u64>,
}

/// Why this build cannot install an update, or `None` when it can.
///
/// Pure, so each refusal has a test: the inputs are the release tag stamped in
/// by the build, the public key from the configuration, and the bundle the
/// running binary was installed from.
pub fn blocker(
    release: Option<&str>,
    key: &str,
    bundle: Option<BundleType>,
) -> Option<&'static str> {
    if release.is_none() {
        return Some("A development build is not updated in place.");
    }
    if key.trim().is_empty() {
        return Some("This build has no update key, so it cannot verify a download.");
    }
    match bundle {
        Some(BundleType::Nsis | BundleType::AppImage | BundleType::App) => None,
        // Releases sign the NSIS installer, the AppImage and the macOS app;
        // a package manager owns the other two.
        Some(BundleType::Msi) => Some("Installed from the .msi. Install the new release with its setup .exe once, and updates arrive from then on."),
        Some(BundleType::Deb | BundleType::Rpm) => Some("Installed by your package manager. Install the new release's package to update."),
        _ => Some("Not installed by Workbench's installer. Download the new release instead."),
    }
}

/// The public key `tauri.conf.json` carries for the updater.
fn key<R: Runtime>(app: &AppHandle<R>) -> String {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|updater| updater.get("pubkey"))
        .and_then(|key| key.as_str())
        .unwrap_or_default()
        .to_string()
}

fn failed(detail: impl std::fmt::Display) -> Error {
    Error::Forge {
        host: "github.com".into(),
        detail: detail.to_string(),
    }
}

/// Look for a newer release and, if there is one, download it.
///
/// The download is held in memory until [`update_restart`]; nothing on disk
/// changes here. A second call while one is running is refused rather than
/// downloading twice, and a call after a download has finished answers from
/// what is already held.
#[tauri::command]
pub async fn update_prepare<R: Runtime>(
    app: AppHandle<R>,
    state: State<'_, UpdaterState>,
    progress: Channel<Progress>,
) -> Result<Prepared> {
    let current = app.package_info().version.to_string();
    if let Some(reason) = blocker(
        spagitty_core::update::current_release(),
        &key(&app),
        tauri::utils::platform::bundle_type(),
    ) {
        return Ok(Prepared::Unsupported {
            reason: reason.into(),
        });
    }
    if let Some(ready) = state
        .ready
        .lock()
        .map_err(|e| Error::Config(e.to_string()))?
        .as_ref()
    {
        return Ok(Prepared::Ready {
            version: ready.update.version.clone(),
            current,
        });
    }
    if state.busy.swap(true, Ordering::AcqRel) {
        return Err(Error::Config("An update check is already running.".into()));
    }
    let result = prepare(&app, &state, &progress, current).await;
    state.busy.store(false, Ordering::Release);
    result
}

async fn prepare<R: Runtime>(
    app: &AppHandle<R>,
    state: &UpdaterState,
    progress: &Channel<Progress>,
    current: String,
) -> Result<Prepared> {
    let updater = app.updater().map_err(failed)?;
    let Some(update) = updater.check().await.map_err(failed)? else {
        return Ok(Prepared::Current { version: current });
    };

    // A progress message per whole percent, not per chunk: ninety megabytes
    // arrive in thousands of chunks, and each message crosses into the webview.
    let mut received = 0u64;
    let mut reported = 0u64;
    let bytes = update
        .download(
            |chunk, total| {
                received += chunk as u64;
                let step = total.map_or(1 << 20, |total| (total / 100).max(1));
                if received - reported >= step || Some(received) == total {
                    reported = received;
                    let _ = progress.send(Progress { received, total });
                }
            },
            || {},
        )
        .await
        .map_err(failed)?;

    let version = update.version.clone();
    *state
        .ready
        .lock()
        .map_err(|e| Error::Config(e.to_string()))? = Some(Ready { update, bytes });
    Ok(Prepared::Ready { version, current })
}

/// Install the downloaded update and start the new version.
///
/// Windows hands over to the installer, which closes this process, updates in
/// place for the current user without asking for administrator rights, and
/// starts Workbench again. macOS and Linux replace the bundle here, and the app
/// restarts through the event loop so agents and extensions are shut down the
/// way an ordinary quit shuts them down.
#[tauri::command(async)]
pub fn update_restart<R: Runtime>(app: AppHandle<R>, state: State<'_, UpdaterState>) -> Result<()> {
    let ready = state
        .ready
        .lock()
        .map_err(|e| Error::Config(e.to_string()))?
        .take()
        .ok_or_else(|| Error::Config("No update has been downloaded.".into()))?;

    // On Windows the installer takes over and this process exits inside
    // `install`, before the event loop could say goodbye to anything.
    #[cfg(windows)]
    {
        crate::agents::shutdown(&app);
        crate::extensions::shutdown(&app);
    }

    ready.update.install(&ready.bytes).map_err(failed)?;
    app.request_restart();
    Ok(())
}

/// Register the plugin's state slot beside the app's own.
pub fn manage<R: Runtime>(app: &AppHandle<R>) {
    app.manage(UpdaterState::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk=";

    #[test]
    fn an_installed_release_with_a_key_can_update() {
        for bundle in [BundleType::Nsis, BundleType::AppImage, BundleType::App] {
            let name = format!("{bundle:?}");
            assert_eq!(blocker(Some("v1.4.0"), KEY, Some(bundle)), None, "{name}");
        }
    }

    #[test]
    fn a_development_build_is_never_updated_in_place() {
        // It has no release to be behind, the same rule the notice follows.
        let reason = blocker(None, KEY, Some(BundleType::Nsis)).unwrap();
        assert!(reason.contains("development build"), "{reason}");
    }

    #[test]
    fn a_build_without_a_key_refuses_rather_than_trusting_an_unsigned_download() {
        for key in ["", "  "] {
            let reason = blocker(Some("v1.4.0"), key, Some(BundleType::Nsis)).unwrap();
            assert!(reason.contains("update key"), "{reason}");
        }
    }

    #[test]
    fn a_binary_no_installer_put_there_says_to_download() {
        let reason = blocker(Some("v1.4.0"), KEY, None).unwrap();
        assert!(reason.contains("Download the new release"), "{reason}");
    }

    #[test]
    fn an_msi_install_is_pointed_at_the_setup_exe() {
        // Releases publish the NSIS installer's update; an .msi install would
        // be updated by a different installer and end up installed twice.
        let reason = blocker(Some("v1.4.0"), KEY, Some(BundleType::Msi)).unwrap();
        assert!(reason.contains("setup .exe"), "{reason}");
    }

    #[test]
    fn a_package_install_is_left_to_its_package_manager() {
        for bundle in [BundleType::Deb, BundleType::Rpm] {
            let reason = blocker(Some("v1.4.0"), KEY, Some(bundle)).unwrap();
            assert!(reason.contains("package manager"), "{reason}");
        }
    }

    #[test]
    fn the_answer_is_tagged_for_the_webview() {
        let ready = serde_json::to_value(Prepared::Ready {
            version: "1.4.0".into(),
            current: "1.3.1".into(),
        })
        .unwrap();
        assert_eq!(ready["state"], "ready");
        assert_eq!(ready["version"], "1.4.0");

        let current = serde_json::to_value(Prepared::Current {
            version: "1.3.1".into(),
        })
        .unwrap();
        assert_eq!(current["state"], "current");
    }

    #[test]
    fn restarting_with_nothing_downloaded_is_an_error() {
        let app = crate::testing::app();
        app.manage(UpdaterState::default());
        let error = update_restart(app.handle().clone(), app.state()).unwrap_err();
        assert!(error.to_string().contains("No update"), "{error}");
    }
}
