// SPDX-License-Identifier: MIT

//! What a user reads that differs between Spagitty and Workbench.
//!
//! Workbench is CIB's build of Spagitty. Internal names stay Spagitty's so
//! that upstream keeps merging in; messages a person reads take the product
//! name from here. `src/lib/brand.ts` holds the same values for the frontend.

/// The name in every message.
pub const PRODUCT_NAME: &str = "Workbench";

/// The bundle identifier, as in `tauri.conf.json`. It names the app-data
/// folder and the keychain group, so Workbench sits beside Spagitty.
pub const APP_ID: &str = "com.cibeg.workbench";

/// The SPDX identifier of Workbench's own code.
pub const LICENCE: &str = "MIT";

/// The Spagitty release this build is based on. Extensions state the
/// Spagitty version they need, so their compatibility is checked against
/// this, not against Workbench's own version.
pub const BASED_ON_SPAGITTY: &str = "4.0.1";

/// What the forge and model clients send as `User-Agent`: a name and a
/// version, nothing about the machine.
pub const USER_AGENT: &str = concat!("workbench/", env!("CARGO_PKG_VERSION"));

/// Whether the app asks for a newer release. Workbench reads its own
/// repository's GitHub releases, and installs from them (FEAT-187).
pub const CHECKS_FOR_UPDATES: bool = true;

/// Where a person goes for the release the notice names, when the host's
/// answer carries no page of its own.
pub const RELEASES_PAGE: &str = "https://github.com/maxmya/cib-workbench/releases";

/// Where Workbench behaves differently from Spagitty. `FEATURES` in
/// `src/lib/brand.ts` holds the same switches for the frontend.
pub mod features {
    /// Onboarding, Home across followed CIB GitLab projects, and Load
    /// repository in place of tabs (2.0.0, FEAT-180 to FEAT-184).
    pub const WORKBENCH_WIDE: bool = true;
}

/// CIB's GitLab, built in (FEAT-181). `GITLAB` in `src/lib/brand.ts` holds
/// the same values; `tools/workbench-brand.test.ts` checks they agree.
pub mod gitlab {
    /// Lower case, no scheme. GitLab Enterprise Edition 16.6.2.
    pub const HOST: &str = "gitlab.apps.ocp-nonprod-01.hodomain.local";
    /// Confirmed signed in on 2026-10-28.
    pub const TOKEN_PATH: &str = "/-/profile/personal_access_tokens";
    pub const TOKEN_NAME: &str = "Workbench";
    /// `api` for merge requests, pipelines, issues and releases;
    /// `read_repository` to clone and fetch; `write_repository` to push.
    pub const SCOPES: [&str; 3] = ["api", "read_repository", "write_repository"];
}

/// The `User-Agent` product token Calls sends: `Workbench/<version>`.
pub fn user_agent(version: &str) -> String {
    format!("{PRODUCT_NAME}/{version}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_user_agent_names_the_product_and_its_version() {
        assert_eq!(user_agent("1.0.0"), "Workbench/1.0.0");
    }
}
