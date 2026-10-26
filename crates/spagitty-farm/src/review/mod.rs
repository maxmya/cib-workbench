// SPDX-License-Identifier: MIT

//! A second agent's opinion, and the rule that it must be a second agent.

pub mod decision;
pub mod reviewer;

pub use decision::{Decision, Issue, Review, Severity};
pub use reviewer::pick;
