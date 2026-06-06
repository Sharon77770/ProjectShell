# ProjectShell MVP Plan

## Product Direction

ProjectShell is a transient Workspace Navigator. It should feel closer to a
keyboard-first explorer in VSCode, Raycast, PowerToys Run, and the VSCode
Command Palette than a persistent sidebar or CRUD dashboard.

## Current Structure

- The repository root is a Cargo workspace.
- The executable crate is `projectshell/`.
- Models and services remain separated from UI.
- Main navigator UI and settings UI are separate.

## Module Structure

- `src/main.rs`: eframe bootstrap and fixed `620x520` overlay options.
- `src/app.rs`: row state, search, selection, keyboard handling, resume actions.
- `src/models/`: serializable `Project`, `AppItem`, and `AppStatus`.
- `src/services/`: JSON storage, process launch, and process status checks.
- `src/ui/launcher.rs`: header/search/footer and shared launcher styling.
- `src/ui/project_list.rs`: project/app tree rows and row rendering.
- `src/ui/project_summary.rs`: compact selected project/app detail panel.
- `src/ui/settings.rs`: simple project/app editing mode.

## Implemented Flow

1. Load projects from `config/projects.json`.
2. Show a fixed `620x520` always-on-top navigator.
3. Build visible rows from project parents and app children.
4. Filter by project name, description, workspace path, and app name.
5. Navigate rows and projects with keyboard shortcuts.
6. Resume selected project or launch selected app with `Enter`.
7. Enter settings with `Ctrl+,` or the footer button.

## Deferred Scope

- Global launcher hotkey.
- Auto-hide after Resume Workspace.
- Tray resident mode.
- Window position persistence.
- Better search ranking.
- Separate polished configuration editor.
