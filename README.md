# ProjectShell

ProjectShell is a native Windows-first Workspace Navigator for developers. It is
not a CRUD dashboard, persistent sidebar, or general desktop app. The main view
is a compact overlay for switching projects and launching the tools attached to
the selected workspace.

## Tech Stack

- Rust
- eframe / egui
- serde
- serde_json
- JSON file storage

ProjectShell does not use Tauri, Electron, or WebView.

## Running

From the repository root:

```powershell
cargo run
```

## Workspace Navigator

The launcher opens as a fixed `620x520` always-on-top overlay.

- Header: title and drag region.
- Search: `Search workspace or app...`.
- Tree view: project rows are parents, app rows are children.
- Detail panel: compact context for the selected project or app.
- Footer: keyboard hints and a Settings entry.

The main view does not show app cards, executable paths, args, or project CRUD
controls.

## Keyboard

- `Up` / `Down`: move one visible row.
- `Shift+Up` / `Shift+Down`: move to previous or next project row.
- `Left`: move from app row to its parent project.
- `Right`: move from project row to its first app row.
- `Shift+Left` / `Shift+Right`: collapse or expand the current project.
- `Space`: toggle the current project row.
- `Enter`: resume a selected project or launch a selected app.
- `Esc`: close the launcher, or leave Settings mode.
- `Ctrl+,`: open Settings mode.

Mouse clicks only move selection. Double-clicking rows or clicking the detail
panel does not launch anything; activation is intentionally gated on `Enter`.

## Search

Search matches project name, description, workspace path, and app name. If an app
matches, its parent project is also shown. Search results are auto-expanded and
selection resets to the first visible row when the query changes.

## Resume Workspace

Resume Workspace launches every app registered on the selected project using
`std::process::Command`. Apps with an empty executable path are skipped. Launch
successes, skips, and the first failure are shown as a compact status message.

## Settings Mode

Settings Mode is intentionally simple in this MVP. It supports basic project and
app editing so the main navigator can stay focused on switching workspaces.

## Config File

ProjectShell stores data in:

```text
config/projects.json
```

The file is pretty JSON and contains the selected project id plus project data.
If it does not exist, ProjectShell creates sample projects.

## Current Limits

- No global hotkey registration yet.
- Resume does not auto-hide the launcher yet.
- Settings Mode is basic and should become a separate configuration experience.
- App status detection is filename-based using Windows `tasklist`.
- Argument parsing is whitespace-based.
