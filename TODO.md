# ProjectShell TODO

## This Work

- [x] Analyze current project structure.
- [x] Create `PLAN.md`.
- [x] Create `TODO.md`.
- [x] Add root Cargo workspace support for `cargo run`.
- [x] Implement project and app models.
- [x] Implement JSON storage in `config/projects.json`.
- [x] Implement individual app launch.
- [x] Implement Launch All / Resume Workspace.
- [x] Implement first-pass app status display.
- [x] Separate Settings mode from the main launcher.
- [x] Remove project CRUD controls from the main launcher.
- [x] Remove app CRUD controls from the main launcher.
- [x] Remove card-based project and app UI from the main launcher.
- [x] Remove floating button and sidebar entry layouts.
- [x] Rebuild main UI as a fixed `620x520` Workspace Navigator.
- [x] Add project parent rows and app child rows.
- [x] Add `LauncherRow`-based visible row calculation.
- [x] Add per-project expand/collapse state.
- [x] Add project/app search with auto-expanded search results.
- [x] Add keyboard navigation for rows, projects, collapse/expand, and activation.
- [x] Add compact selected detail panel for project and app rows.
- [x] Keep the main launcher focused on project switching.
- [x] Update README.

## Known Incomplete Items

- [ ] System tray icon, hide, restore, and quit menu.
- [ ] Robust process tracking by PID after launch.
- [ ] Native Windows always-on-top behavior validation across all window managers.
- [ ] App icon discovery and display.
- [ ] Project-specific tab/session restore.
- [ ] Separate polished configuration editor outside the launcher.
- [ ] Global hotkey for show/hide.
- [ ] Auto-hide after Resume Workspace.
- [ ] Save and restore launcher window position.
- [ ] Project-specific window/tab snapshots.
- [ ] Improve settings screen visual design.
- [ ] Improve search ranking.

## Next Features

- [ ] Support quoted argument parsing.
- [ ] Add recent launch history.
- [ ] Add per-project terminal command presets.
