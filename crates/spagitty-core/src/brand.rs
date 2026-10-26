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

/// Whether the app asks for a newer release. Workbench comes through CIB's
/// internal software channel, so it asks nobody and makes no request.
pub const CHECKS_FOR_UPDATES: bool = false;

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
