<!-- SPDX-License-Identifier: MIT -->

# FEAT-180 — Automated tests

**Item:** [`agile/items/FEAT-180-first-launch-onboarding.md`](../items/FEAT-180-first-launch-onboarding.md)

| Where | Asserts |
| --- | --- |
| `src/lib/workbench/onboarding.test.ts` | All six pages with a checked token, then finish; Next held on Token until the check passes; Back page by page and from GitLab to the last Welcome page; GitLab and Token passed over when connected, both ways, and marked done. |
| `src/lib/workbench/onboarding-screen.test.ts` | Back hidden on the first page; *Set up later* reaches Home and leaves `onboarded` unset; Next held until GitLab accepts the token; "Connected as @login" and the passed-over steps; followed and `HomeLayout` written only on *Open Workbench*, then `onboarded`. |
| `src/lib/session.test.ts` | Onboarding before anything is read; a command-line path loaded after it; the shell going away during it stops the launch. |
| `src-tauri/src/workbench.rs` | `onboarded` reads back with its version and time. |
