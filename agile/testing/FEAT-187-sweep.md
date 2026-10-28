<!-- SPDX-License-Identifier: MIT -->

# FEAT-187 — Manual sweep

**Item:** [`agile/items/FEAT-187-workbench-installs-its-own-updates.md`](../items/FEAT-187-workbench-installs-its-own-updates.md)

Needs the key setup in `docs/self-update.md` and **two** releases built with
it: one to install, one to update onto.

| Ticket | Preconditions | Steps | Expected result | Priority | Pass/Fail |
| --- | --- | --- | --- | --- | --- |
| SWEEP-001 | Key set, a release from gate 6 | Read the release's assets | `latest.json` is attached and names the setup `.exe`, the `.AppImage` and `Workbench-macos-arm64.app.tar.gz`, each with a signature. | High | |
| SWEEP-002 | Windows, no administrator rights | Run the `*-setup.exe` | The wizard shows CIB's header and sidebar, asks for no administrator, installs under `%LOCALAPPDATA%`, and starts Workbench. | High | |
| SWEEP-003 | SWEEP-002's install, a newer release published | Start Workbench | *Restart to update* appears in the status strip; Settings shows the version as downloaded. Nothing installs on its own. | High | |
| SWEEP-004 | SWEEP-003 | Press *Restart to update* | A progress bar, no administrator prompt, and Workbench starts again on the new version. Settings, accounts and recent repositories are kept. | High | |
| SWEEP-005 | macOS, the `.dmg` | Open it | A white window with the app, an arrow and Applications, laid out. | Medium | |
| SWEEP-006 | macOS install, a newer release | SWEEP-003 and SWEEP-004 | The app is replaced and restarts. | High | |
| SWEEP-007 | Linux, the `.AppImage` run from a writable folder | SWEEP-003 and SWEEP-004 | The AppImage is replaced in place and restarts. | High | |
| SWEEP-008 | The bare `cib-workbench-windows-x86_64.exe` | Settings → Check now | It says it was not installed by the installer and links to the release. | Medium | |
| SWEEP-009 | A `latest.json` whose signature is edited by one character, served to a test build | Check now | The download is refused; nothing is installed. | High | |
| SWEEP-010 | Settings → Behaviour → Updates turned off | Start Workbench | No request is made, and nothing appears in the status strip. | Medium | |
