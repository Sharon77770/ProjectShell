use std::collections::{HashMap, HashSet};
use std::fs;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use crate::i18n::{I18n, Text};
use crate::models::{AppItem, AppStatus, Project};
use crate::services::window_service::RunningWindow;
use crate::services::{
    adapter_service, focus_service, launcher_service, log_service, process_service,
    resident_service, storage_service, window_service,
};
use crate::ui::help::{self, HelpAction};
use crate::ui::launcher::{self, LauncherAction, BG, BORDER, PANEL, TEXT, TEXT_SECONDARY};
use crate::ui::project_list::{self, LauncherRow, TreeAction};
use crate::ui::project_summary;
use crate::ui::settings::{self, SettingsAction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViewMode {
    Launcher,
    Settings,
    Help,
}

#[derive(Debug)]
struct StatusSnapshot {
    app_statuses: HashMap<String, AppStatus>,
    running_windows: Vec<RunningWindow>,
}

pub struct ProjectShellApp {
    projects: Vec<Project>,
    selected_project_id: Option<String>,
    selected_row_index: usize,
    expanded_project_ids: HashSet<String>,
    search_query: String,
    view_mode: ViewMode,
    settings_focus_app_id: Option<String>,
    i18n: I18n,
    status_message: String,
    status_is_error: bool,
    app_statuses: HashMap<String, AppStatus>,
    running_windows: Vec<RunningWindow>,
    running_window_aliases: HashMap<isize, String>,
    resident: Option<resident_service::ResidentController>,
    gui_visible: bool,
    initial_hide_done: bool,
    resident_started_at: Instant,
    last_status_refresh: Option<Instant>,
    status_rx: Option<mpsc::Receiver<Result<StatusSnapshot, String>>>,
    status_refresh_in_flight: bool,
    id_counter: u64,
    viewport_initialized: bool,
}

impl ProjectShellApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_egui(&cc.egui_ctx);

        let (projects, selected_project_id, language, status_message, status_is_error) =
            match storage_service::load_or_create() {
                Ok(state) => (
                    state.projects,
                    state.selected_project_id,
                    state.language,
                    String::new(),
                    false,
                ),
                Err(err) => {
                    let state = storage_service::default_state();
                    (
                        state.projects,
                        state.selected_project_id,
                        state.language,
                        format!("Sample data loaded. {err}"),
                        true,
                    )
                }
            };

        let expanded_project_ids = projects
            .iter()
            .map(|project| project.id.clone())
            .collect::<HashSet<_>>();

        let mut app = Self {
            projects,
            selected_project_id,
            selected_row_index: 0,
            expanded_project_ids,
            search_query: String::new(),
            view_mode: ViewMode::Launcher,
            settings_focus_app_id: None,
            i18n: I18n::new(language),
            status_message,
            status_is_error,
            app_statuses: HashMap::new(),
            running_windows: Vec::new(),
            running_window_aliases: HashMap::new(),
            resident: None,
            gui_visible: false,
            initial_hide_done: false,
            resident_started_at: Instant::now(),
            last_status_refresh: None,
            status_rx: None,
            status_refresh_in_flight: false,
            id_counter: 0,
            viewport_initialized: false,
        };

        app.ensure_valid_selection();
        app.sync_row_to_selected_project();
        app.install_resident_mode(cc);
        app.refresh_statuses();
        app
    }

    fn install_resident_mode(&mut self, cc: &eframe::CreationContext<'_>) {
        let Some(hwnd) = hwnd_from_creation_context(cc) else {
            self.set_status(
                format!(
                    "{}: window handle not found.",
                    self.i18n.t(Text::ResidentUnavailable)
                ),
                true,
            );
            return;
        };

        match resident_service::ResidentController::install(hwnd) {
            Ok(controller) => {
                self.resident = Some(controller);
                self.gui_visible = false;
                self.initial_hide_done = false;
                self.resident_started_at = Instant::now();
                self.set_status(self.i18n.t(Text::ResidentActive), false);
            }
            Err(err) => {
                log_service::log_error(format!("Resident mode failed: {err}"));
                self.gui_visible = true;
                cc.egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::Visible(true));
                self.set_status(err, true);
            }
        }
    }

    fn set_status(&mut self, message: impl Into<String>, is_error: bool) {
        self.status_message = launcher::truncate(message.into().as_str(), 96);
        self.status_is_error = is_error;
    }

    fn save(&mut self, message: impl Into<String>) {
        match storage_service::save_state(
            &self.projects,
            self.selected_project_id.as_deref(),
            self.i18n.language(),
        ) {
            Ok(()) => self.set_status(message, false),
            Err(err) => {
                log_service::log_error(format!("Save failed: {err}"));
                self.set_status(err, true);
            }
        }
    }

    fn persist_selection(&self) {
        if let Err(err) = storage_service::save_state(
            &self.projects,
            self.selected_project_id.as_deref(),
            self.i18n.language(),
        ) {
            log_service::log_error(format!("Selection save failed: {err}"));
        }
    }

    fn ensure_valid_selection(&mut self) {
        let selected_is_valid = self
            .selected_project_id
            .as_deref()
            .is_some_and(|id| self.projects.iter().any(|project| project.id == id));

        if !selected_is_valid {
            self.selected_project_id = self.projects.first().map(|project| project.id.clone());
        }
    }

    fn selected_project_index(&self) -> Option<usize> {
        let selected_id = self.selected_project_id.as_deref()?;
        self.projects
            .iter()
            .position(|project| project.id == selected_id)
    }

    fn selected_project_mut(&mut self) -> Option<&mut Project> {
        let index = self.selected_project_index()?;
        self.projects.get_mut(index)
    }

    fn project_by_id(&self, project_id: &str) -> Option<&Project> {
        self.projects
            .iter()
            .find(|project| project.id == project_id)
    }

    fn next_id(&mut self, prefix: &str) -> String {
        self.id_counter += 1;
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or_default();
        format!("{prefix}-{millis}-{}", self.id_counter)
    }

    fn visible_rows(&self) -> Vec<LauncherRow> {
        let query = self.search_query.trim().to_ascii_lowercase();
        let searching = !query.is_empty();
        let mut rows = Vec::new();

        let matching_windows = self
            .running_windows
            .iter()
            .filter(|window| {
                let alias = self
                    .running_window_aliases
                    .get(&window.hwnd)
                    .map(String::as_str)
                    .unwrap_or_default();
                !searching
                    || format!("{} {} {}", alias, window.process_name, window.title)
                        .to_ascii_lowercase()
                        .contains(query.as_str())
            })
            .collect::<Vec<_>>();
        if !matching_windows.is_empty() {
            rows.push(LauncherRow::Section {
                label: self.i18n.t(Text::RunningApps).to_owned(),
            });
            for window in matching_windows {
                rows.push(LauncherRow::RunningWindow { hwnd: window.hwnd });
            }
        }

        for project in &self.projects {
            let project_match = searching
                && format!(
                    "{} {} {}",
                    project.name, project.description, project.workspace_path
                )
                .to_ascii_lowercase()
                .contains(query.as_str());

            let matching_apps = project
                .apps
                .iter()
                .filter(|app| {
                    !searching
                        || format!(
                            "{} {} {} {}",
                            app.name, app.process_name, app.executable_path, app.window_title_match
                        )
                        .to_ascii_lowercase()
                        .contains(query.as_str())
                })
                .collect::<Vec<_>>();

            if searching {
                if !project_match && matching_apps.is_empty() {
                    continue;
                }
                rows.push(LauncherRow::Project {
                    project_id: project.id.clone(),
                });
                let apps_to_show = if project_match {
                    project.apps.iter().collect::<Vec<_>>()
                } else {
                    matching_apps
                };
                for app in apps_to_show {
                    rows.push(LauncherRow::App {
                        project_id: project.id.clone(),
                        app_id: app.id.clone(),
                    });
                }
            } else {
                rows.push(LauncherRow::Project {
                    project_id: project.id.clone(),
                });
                if self.expanded_project_ids.contains(project.id.as_str()) {
                    for app in &project.apps {
                        rows.push(LauncherRow::App {
                            project_id: project.id.clone(),
                            app_id: app.id.clone(),
                        });
                    }
                }
            }
        }

        rows
    }

    fn selected_row(&self) -> Option<LauncherRow> {
        self.visible_rows().get(self.selected_row_index).cloned()
    }

    fn clamp_selected_row(&mut self) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            self.selected_row_index = 0;
            return;
        }
        if self.selected_row_index >= rows.len() {
            self.selected_row_index = rows.len() - 1;
        }
        if !Self::is_selectable_row(&rows[self.selected_row_index]) {
            if let Some(index) = rows
                .iter()
                .enumerate()
                .skip(self.selected_row_index + 1)
                .find_map(|(index, row)| Self::is_selectable_row(row).then_some(index))
                .or_else(|| {
                    rows.iter()
                        .enumerate()
                        .take(self.selected_row_index)
                        .rev()
                        .find_map(|(index, row)| Self::is_selectable_row(row).then_some(index))
                })
            {
                self.selected_row_index = index;
            }
        }
        self.sync_project_from_selected_row();
    }

    fn is_selectable_row(row: &LauncherRow) -> bool {
        !matches!(row, LauncherRow::Section { .. })
    }

    fn sync_project_from_selected_row(&mut self) {
        let Some(row) = self.selected_row() else {
            return;
        };
        let project_id = match row {
            LauncherRow::Section { .. } | LauncherRow::RunningWindow { .. } => return,
            LauncherRow::Project { project_id } => project_id,
            LauncherRow::App { project_id, .. } => project_id,
        };
        self.selected_project_id = Some(project_id);
    }

    fn sync_row_to_selected_project(&mut self) {
        let Some(selected_project_id) = self.selected_project_id.as_deref() else {
            self.selected_row_index = 0;
            return;
        };
        let rows = self.visible_rows();
        self.selected_row_index = rows
            .iter()
            .position(|row| {
                matches!(
                    row,
                    LauncherRow::Project { project_id } if project_id == selected_project_id
                )
            })
            .unwrap_or(0);
    }

    fn select_row_index(&mut self, index: usize) {
        self.selected_row_index = index;
        self.clamp_selected_row();
        self.persist_selection();
    }

    fn move_selection_delta(&mut self, delta: isize) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            self.selected_row_index = 0;
            return;
        }
        let len = rows.len() as isize;
        for step in 1..=rows.len() {
            let next =
                (self.selected_row_index as isize + delta * step as isize).rem_euclid(len) as usize;
            if Self::is_selectable_row(&rows[next]) {
                self.selected_row_index = next;
                break;
            }
        }
        self.sync_project_from_selected_row();
        self.persist_selection();
    }

    fn move_to_prev_project(&mut self) {
        self.move_to_project_direction(-1);
    }

    fn move_to_next_project(&mut self) {
        self.move_to_project_direction(1);
    }

    fn move_to_project_direction(&mut self, direction: isize) {
        let rows = self.visible_rows();
        if rows.is_empty() {
            return;
        }

        let mut project_indexes = rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| matches!(row, LauncherRow::Project { .. }).then_some(index))
            .collect::<Vec<_>>();
        if project_indexes.is_empty() {
            return;
        }

        project_indexes.sort_unstable();
        let current_project_position = project_indexes
            .iter()
            .rposition(|index| *index <= self.selected_row_index)
            .unwrap_or(0);
        let len = project_indexes.len() as isize;
        let next_project_position =
            (current_project_position as isize + direction).rem_euclid(len) as usize;
        self.select_row_index(project_indexes[next_project_position]);
    }

    fn collapse_current_project(&mut self) {
        let Some(row) = self.selected_row() else {
            return;
        };
        let project_id = match row {
            LauncherRow::Section { .. } | LauncherRow::RunningWindow { .. } => return,
            LauncherRow::Project { project_id } => project_id,
            LauncherRow::App { project_id, .. } => project_id,
        };
        self.expanded_project_ids.remove(project_id.as_str());
        self.select_project_row(&project_id);
    }

    fn expand_current_project(&mut self) {
        let Some(row) = self.selected_row() else {
            return;
        };
        let project_id = match row {
            LauncherRow::Section { .. } | LauncherRow::RunningWindow { .. } => return,
            LauncherRow::Project { project_id } => project_id,
            LauncherRow::App { project_id, .. } => project_id,
        };
        self.expanded_project_ids.insert(project_id);
        self.clamp_selected_row();
    }

    fn move_app_to_parent_project(&mut self) {
        let Some(LauncherRow::App { project_id, .. }) = self.selected_row() else {
            return;
        };
        self.select_project_row(&project_id);
    }

    fn move_project_to_first_app(&mut self) {
        let Some(LauncherRow::Project { project_id }) = self.selected_row() else {
            return;
        };
        if self.search_query.trim().is_empty() {
            self.expanded_project_ids.insert(project_id.clone());
        }
        let rows = self.visible_rows();
        if let Some(index) = rows.iter().position(|row| {
            matches!(
                row,
                LauncherRow::App {
                    project_id: row_project_id,
                    ..
                } if row_project_id == &project_id
            )
        }) {
            self.select_row_index(index);
        }
    }

    fn toggle_current_project(&mut self) {
        let Some(LauncherRow::Project { project_id }) = self.selected_row() else {
            return;
        };
        if self.expanded_project_ids.contains(project_id.as_str()) {
            self.expanded_project_ids.remove(project_id.as_str());
        } else {
            self.expanded_project_ids.insert(project_id);
        }
        self.clamp_selected_row();
    }

    fn select_project_row(&mut self, project_id: &str) {
        let rows = self.visible_rows();
        if let Some(index) = rows.iter().position(|row| {
            matches!(row, LauncherRow::Project { project_id: row_project_id } if row_project_id == project_id)
        }) {
            self.select_row_index(index);
        }
    }

    fn handle_launcher_keyboard(&mut self, ctx: &egui::Context) {
        let keyboard = ctx.input(|input| {
            (
                input.key_pressed(egui::Key::ArrowUp),
                input.key_pressed(egui::Key::ArrowDown),
                input.key_pressed(egui::Key::ArrowLeft),
                input.key_pressed(egui::Key::ArrowRight),
                input.key_pressed(egui::Key::Enter),
                input.key_pressed(egui::Key::Space),
                input.modifiers.ctrl && input.key_pressed(egui::Key::A),
                input.modifiers.ctrl && input.key_pressed(egui::Key::O),
                input.modifiers.shift,
            )
        });
        let (up, down, left, right, enter, space, assign, open_workspace, shift) = keyboard;

        if assign {
            self.assign_selected_window_to_project();
        } else if open_workspace {
            self.open_selected_workspace();
        } else if shift && up {
            self.move_to_prev_project();
        } else if shift && down {
            self.move_to_next_project();
        } else if shift && left {
            self.collapse_current_project();
        } else if shift && right {
            self.expand_current_project();
        } else if up {
            self.move_selection_delta(-1);
        } else if down {
            self.move_selection_delta(1);
        } else if left {
            self.move_app_to_parent_project();
        } else if right {
            self.move_project_to_first_app();
        } else if enter {
            self.activate_selected_row();
        } else if space {
            self.toggle_current_project();
        }
    }

    fn handle_keyboard(&mut self, ctx: &egui::Context) {
        let (escape, settings) = ctx.input(|input| {
            (
                input.key_pressed(egui::Key::Escape),
                input.modifiers.ctrl && input.key_pressed(egui::Key::Comma),
            )
        });

        if settings {
            match self.view_mode {
                ViewMode::Launcher | ViewMode::Help => self.open_settings_from_selection(),
                ViewMode::Settings => self.view_mode = ViewMode::Launcher,
            }
        }

        if escape {
            match self.view_mode {
                ViewMode::Launcher => self.hide_gui(ctx),
                ViewMode::Settings | ViewMode::Help => self.view_mode = ViewMode::Launcher,
            }
        }

        if self.view_mode == ViewMode::Launcher {
            self.handle_launcher_keyboard(ctx);
        }
    }

    fn activate_selected_row(&mut self) {
        match self.selected_row() {
            Some(LauncherRow::Project { project_id }) => self.resume_project_by_id(&project_id),
            Some(LauncherRow::App { project_id, app_id }) => {
                self.launch_app_by_id(&project_id, &app_id)
            }
            Some(LauncherRow::RunningWindow { hwnd }) => self.focus_running_window(hwnd),
            Some(LauncherRow::Section { .. }) => {
                self.set_status(self.i18n.t(Text::NoAppSelected), true)
            }
            None => self.set_status(self.i18n.t(Text::NoRowSelected), true),
        }
    }

    fn open_settings_from_selection(&mut self) {
        self.settings_focus_app_id = self.selected_app_id_for_settings();
        self.view_mode = ViewMode::Settings;
    }

    fn selected_app_id_for_settings(&self) -> Option<String> {
        match self.selected_row() {
            Some(LauncherRow::App { app_id, .. }) => Some(app_id),
            _ => None,
        }
    }

    fn resume_project_by_id(&mut self, project_id: &str) {
        self.selected_project_id = Some(project_id.to_owned());
        self.persist_selection();
        self.resume_selected_workspace();
    }

    fn resume_selected_workspace(&mut self) {
        let Some(project_id) = self.selected_project_id.clone() else {
            self.set_status(self.i18n.t(Text::NoWorkspaceSelected), true);
            return;
        };
        let Some(project) = self.project_by_id(&project_id).cloned() else {
            self.set_status(self.i18n.t(Text::WorkspaceNotFound), true);
            return;
        };

        if project.apps.is_empty() {
            self.set_status(
                format!("{}: {}", project.name, self.i18n.t(Text::LaunchSetEmpty)),
                true,
            );
            return;
        }

        let summary = launcher_service::resume_project(&project);
        for app in &project.apps {
            self.app_statuses.insert(app.id.clone(), AppStatus::Running);
        }

        if summary.failed > 0 {
            self.set_status(
                format!(
                    "{}: {} {} / {} {} / {} {}",
                    summary.project_name,
                    self.i18n.t(Text::Focused),
                    summary.focused,
                    self.i18n.t(Text::Launched),
                    summary.launched,
                    self.i18n.t(Text::Failed),
                    summary.failed
                ),
                true,
            );
        } else {
            self.set_status(
                format!(
                    "{}: {} {} / {} {} / {} 0",
                    summary.project_name,
                    self.i18n.t(Text::Focused),
                    summary.focused,
                    self.i18n.t(Text::Launched),
                    summary.launched,
                    self.i18n.t(Text::Failed)
                ),
                false,
            );
        }
    }

    fn launch_app_by_id(&mut self, project_id: &str, app_id: &str) {
        let Some(project) = self.project_by_id(project_id).cloned() else {
            self.set_status(self.i18n.t(Text::WorkspaceNotFound), true);
            return;
        };
        let Some(app) = project.apps.iter().find(|app| app.id == app_id).cloned() else {
            self.set_status(self.i18n.t(Text::AppNotFound), true);
            return;
        };

        if app.executable_path.trim().is_empty() {
            self.set_status(
                format!("{}: {}", app.name, self.i18n.t(Text::AppPathMissing)),
                true,
            );
            return;
        }

        match launcher_service::activate_app(&app, &project.workspace_path) {
            Ok(launcher_service::AppActivation::Focused(window)) => {
                self.app_statuses.insert(app.id.clone(), AppStatus::Running);
                self.set_status(
                    format!("{} {}.", self.i18n.t(Text::Focused), window.process_name),
                    false,
                );
            }
            Ok(launcher_service::AppActivation::Launched) => {
                self.app_statuses.insert(app.id.clone(), AppStatus::Running);
                self.set_status(
                    format!("{} {}.", self.i18n.t(Text::Launched), app.name),
                    false,
                );
            }
            Err(err) => {
                log_service::log_error(format!("App activation failed: {err}"));
                self.app_statuses.insert(app.id.clone(), AppStatus::Stopped);
                self.set_status(err, true);
            }
        }
    }

    fn focus_running_window(&mut self, hwnd: isize) {
        let Some(window) = self
            .running_windows
            .iter()
            .find(|window| window.hwnd == hwnd)
            .cloned()
        else {
            self.set_status(self.i18n.t(Text::RunningAppNotFound), true);
            return;
        };

        match focus_service::focus_window(hwnd) {
            Ok(()) => self.set_status(
                format!("{} {}.", self.i18n.t(Text::Focused), window.process_name),
                false,
            ),
            Err(err) => {
                log_service::log_error(format!("Focus failed for {}: {err}", window.title));
                self.set_status(format!("{}: {err}", self.i18n.t(Text::FocusFailed)), true);
            }
        }
    }

    fn assign_selected_window_to_project(&mut self) {
        let Some(LauncherRow::RunningWindow { hwnd }) = self.selected_row() else {
            self.set_status(self.i18n.t(Text::AssignSelectedRunningApp), true);
            return;
        };
        let Some(window) = self
            .running_windows
            .iter()
            .find(|window| window.hwnd == hwnd)
            .cloned()
        else {
            self.set_status(self.i18n.t(Text::RunningAppNotFound), true);
            return;
        };
        let Some(project_id) = self.selected_project_id.clone() else {
            self.set_status(self.i18n.t(Text::NoWorkspaceSelected), true);
            return;
        };

        let app_id = self.next_id("app");
        let alias = self
            .running_window_aliases
            .get(&hwnd)
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let app = adapter_service::app_from_window(app_id, &window, alias);
        let Some(project) = self
            .projects
            .iter_mut()
            .find(|project| project.id == project_id)
        else {
            self.set_status(self.i18n.t(Text::WorkspaceNotFound), true);
            return;
        };

        if project.workspace_path.trim().is_empty() {
            if let Some(path) = adapter_service::inferred_workspace_path(&window) {
                project.workspace_path = path;
            }
        }

        if let Some(existing) = project.apps.iter_mut().find(|existing| {
            !existing.process_name.trim().is_empty()
                && existing
                    .process_name
                    .eq_ignore_ascii_case(app.process_name.as_str())
                && same_window_title_match(existing, &app)
        }) {
            *existing = app.clone();
        } else {
            project.apps.push(app.clone());
        }

        self.expanded_project_ids.insert(project_id);
        self.app_statuses.insert(app.id.clone(), AppStatus::Running);
        self.save(format!(
            "{}: {}.",
            window.process_name,
            self.i18n.t(Text::Assigned)
        ));
    }

    fn open_selected_workspace(&mut self) {
        let Some(project_id) = self.selected_project_id.clone() else {
            self.set_status(self.i18n.t(Text::NoWorkspaceSelected), true);
            return;
        };
        let Some(project) = self.project_by_id(&project_id).cloned() else {
            self.set_status(self.i18n.t(Text::WorkspaceNotFound), true);
            return;
        };

        match launcher_service::open_workspace(project.workspace_path.as_str()) {
            Ok(()) => self.set_status(
                format!("{}: {}", project.name, self.i18n.t(Text::OpenedWorkspace)),
                false,
            ),
            Err(err) => {
                log_service::log_error(format!("Open workspace failed: {err}"));
                self.set_status(err, true);
            }
        }
    }

    fn refresh_statuses(&mut self) {
        if let Some(rx) = self.status_rx.take() {
            match rx.try_recv() {
                Ok(Ok(snapshot)) => {
                    self.app_statuses = snapshot.app_statuses;
                    self.running_windows = snapshot.running_windows;
                    self.last_status_refresh = Some(Instant::now());
                    self.status_refresh_in_flight = false;
                }
                Ok(Err(err)) => {
                    self.last_status_refresh = Some(Instant::now());
                    self.status_refresh_in_flight = false;
                    self.set_status(
                        format!("{}: {err}", self.i18n.t(Text::ProcessStatusUnavailable)),
                        true,
                    );
                }
                Err(mpsc::TryRecvError::Empty) => {
                    self.status_rx = Some(rx);
                    return;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.last_status_refresh = Some(Instant::now());
                    self.status_refresh_in_flight = false;
                    self.set_status(
                        format!(
                            "{}: worker stopped.",
                            self.i18n.t(Text::ProcessStatusUnavailable)
                        ),
                        true,
                    );
                }
            }
        }

        if self.status_refresh_in_flight {
            return;
        }

        let should_refresh = self
            .last_status_refresh
            .map(|last| last.elapsed() >= Duration::from_secs(3))
            .unwrap_or(true);

        if !should_refresh {
            return;
        }

        let apps = self
            .projects
            .iter()
            .flat_map(|project| project.apps.iter().cloned())
            .collect::<Vec<AppItem>>();

        let (tx, rx) = mpsc::channel();
        self.status_rx = Some(rx);
        self.status_refresh_in_flight = true;
        self.last_status_refresh = Some(Instant::now());

        thread::spawn(move || {
            let result = window_service::list_running_windows().map(|running_windows| {
                let app_statuses =
                    process_service::query_statuses_from_windows(&apps, &running_windows);
                StatusSnapshot {
                    app_statuses,
                    running_windows,
                }
            });
            let _ = tx.send(result);
        });
    }

    fn add_project(&mut self) {
        let id = self.next_id("project");
        self.projects.push(Project::new(
            id.clone(),
            self.i18n.t(Text::NewWorkspace),
            "",
            "",
            Vec::new(),
        ));
        self.expanded_project_ids.insert(id.clone());
        self.selected_project_id = Some(id);
        self.sync_row_to_selected_project();
        self.save(self.i18n.t(Text::WorkspaceAdded));
    }

    fn delete_selected_project(&mut self) {
        let Some(selected_id) = self.selected_project_id.clone() else {
            return;
        };
        self.projects.retain(|project| project.id != selected_id);
        self.expanded_project_ids.remove(selected_id.as_str());
        self.selected_project_id = self.projects.first().map(|project| project.id.clone());
        self.clamp_selected_row();
        self.save(self.i18n.t(Text::WorkspaceDeleted));
    }

    fn add_app_to_selected_project(&mut self) {
        let app_id = self.next_id("app");
        let app_name = self.i18n.t(Text::NewApp);
        let Some(project_id) = self.selected_project_id.clone() else {
            return;
        };
        if let Some(project) = self.selected_project_mut() {
            project.apps.push(AppItem::new(app_id, app_name, "", ""));
            self.expanded_project_ids.insert(project_id);
            self.save(self.i18n.t(Text::AppAdded));
        }
    }

    fn delete_app_by_id(&mut self, app_id: &str) {
        if let Some(project) = self.selected_project_mut() {
            project.apps.retain(|app| app.id != app_id);
            self.app_statuses.remove(app_id);
            self.clamp_selected_row();
            self.save(self.i18n.t(Text::AppDeleted));
        }
    }

    fn apply_viewport(&mut self, ctx: &egui::Context) {
        if self.viewport_initialized {
            return;
        }

        self.viewport_initialized = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(620.0, 520.0)));
        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
            egui::WindowLevel::AlwaysOnTop,
        ));
    }

    fn handle_resident_requests(&mut self, ctx: &egui::Context) {
        if resident_service::consume_show_request() {
            self.show_gui(ctx);
        }

        if resident_service::consume_exit_request() {
            self.resident.take();
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if self.resident.is_some()
            && !self.initial_hide_done
            && self.resident_started_at.elapsed() >= Duration::from_millis(1200)
        {
            self.initial_hide_done = true;
            self.hide_gui(ctx);
        }
    }

    fn hide_gui(&mut self, ctx: &egui::Context) {
        self.gui_visible = false;
        let _ = ctx;
        if let Some(resident) = &self.resident {
            resident.hide_window();
        }
    }

    fn show_gui(&mut self, ctx: &egui::Context) {
        self.gui_visible = true;
        self.initial_hide_done = true;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
        if let Some(resident) = &self.resident {
            resident.show_window();
        }
    }
}

impl eframe::App for ProjectShellApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [11.0 / 255.0, 13.0 / 255.0, 18.0 / 255.0, 1.0]
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_viewport(ctx);
        self.handle_resident_requests(ctx);
        self.refresh_statuses();
        self.handle_keyboard(ctx);
        self.clamp_selected_row();

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG))
            .show(ctx, |ui| match self.view_mode {
                ViewMode::Launcher => self.render_launcher(ui),
                ViewMode::Settings => self.render_settings(ui),
                ViewMode::Help => self.render_help(ui),
            });

        ctx.request_repaint_after(Duration::from_millis(800));
    }
}

fn hwnd_from_creation_context(cc: &eframe::CreationContext<'_>) -> Option<isize> {
    let raw = cc.window_handle().ok()?.as_raw();
    match raw {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
        _ => None,
    }
}

impl ProjectShellApp {
    fn render_launcher(&mut self, ui: &mut egui::Ui) {
        egui::Frame::none()
            .fill(PANEL)
            .rounding(egui::Rounding::same(18.0))
            .stroke(egui::Stroke::new(1.0, BORDER))
            .inner_margin(egui::Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_min_size(egui::vec2(588.0, 488.0));
                let header_response = self.render_header(ui);
                if header_response.drag_started() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.add_space(8.0);
                let search_changed =
                    launcher::render_search_box(ui, &mut self.search_query, self.i18n);
                if search_changed {
                    self.selected_row_index = 0;
                    self.clamp_selected_row();
                }

                ui.add_space(8.0);
                let visible_rows = self.visible_rows();
                for action in project_list::render_tree_view(
                    ui,
                    &self.projects,
                    &self.running_windows,
                    &self.running_window_aliases,
                    &visible_rows,
                    self.i18n,
                    self.selected_row_index,
                    &self.expanded_project_ids,
                    !self.search_query.trim().is_empty(),
                    &self.app_statuses,
                ) {
                    match action {
                        TreeAction::SelectIndex(index) => self.select_row_index(index),
                    }
                }

                ui.add_space(8.0);
                let selected_row = self.selected_row();
                project_summary::render_selected_detail(
                    ui,
                    &self.projects,
                    &self.running_windows,
                    &mut self.running_window_aliases,
                    selected_row.as_ref(),
                    &self.app_statuses,
                    &self.status_message,
                    self.status_is_error,
                    self.i18n,
                );

                ui.add_space(8.0);
                for action in launcher::render_footer_hints(ui, self.i18n) {
                    match action {
                        LauncherAction::OpenSettings => self.open_settings_from_selection(),
                        LauncherAction::OpenHelp => self.view_mode = ViewMode::Help,
                    }
                }
            });
    }

    fn render_header(&self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), 22.0), egui::Sense::drag());
        let painter = ui.painter_at(rect);
        painter.text(
            rect.left_center(),
            egui::Align2::LEFT_CENTER,
            self.i18n.t(Text::ProjectShellTitle),
            egui::FontId::proportional(17.0),
            TEXT,
        );
        painter.text(
            rect.right_center(),
            egui::Align2::RIGHT_CENTER,
            self.i18n.t(Text::WorkspaceNav),
            egui::FontId::proportional(11.0),
            TEXT_SECONDARY,
        );
        response
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        let selected_index = self.selected_project_index();
        let selected_app_id = self.selected_app_id_for_settings();
        let focus_app_id = self.settings_focus_app_id.clone();
        let mut response = settings::render_settings(
            ui,
            selected_index.and_then(|index| self.projects.get_mut(index)),
            selected_app_id.as_deref(),
            focus_app_id.as_deref(),
            self.i18n,
        );
        self.settings_focus_app_id = None;

        if response.changed {
            self.save(self.i18n.t(Text::SavedSettings));
        }

        for action in response.actions.drain(..) {
            match action {
                SettingsAction::Back => self.view_mode = ViewMode::Launcher,
                SettingsAction::AddProject => self.add_project(),
                SettingsAction::DeleteSelected => self.delete_selected_project(),
                SettingsAction::AddApp => self.add_app_to_selected_project(),
                SettingsAction::DeleteApp(app_id) => self.delete_app_by_id(&app_id),
                SettingsAction::SetLanguage(language) => {
                    self.i18n.set_language(language);
                    self.save(self.i18n.t(Text::SavedSettings));
                }
            }
        }
    }

    fn render_help(&mut self, ui: &mut egui::Ui) {
        for action in help::render_help(ui, self.i18n) {
            match action {
                HelpAction::Back => self.view_mode = ViewMode::Launcher,
            }
        }
    }
}

impl Drop for ProjectShellApp {
    fn drop(&mut self) {
        let _ = storage_service::save_state(
            &self.projects,
            self.selected_project_id.as_deref(),
            self.i18n.language(),
        );
    }
}

fn configure_egui(ctx: &egui::Context) {
    install_windows_fonts(ctx);

    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.window_rounding = egui::Rounding::same(18.0);
    style.visuals.panel_fill = BG;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(9.0, 6.0);
    ctx.set_style(style);
}

fn install_windows_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        ("malgun_gothic", r"C:\Windows\Fonts\malgun.ttf"),
        ("malgun_gothic_bold", r"C:\Windows\Fonts\malgunbd.ttf"),
        ("segoe_ui_emoji", r"C:\Windows\Fonts\seguiemj.ttf"),
    ];

    for (name, path) in candidates {
        let Ok(bytes) = fs::read(path) else {
            continue;
        };
        fonts
            .font_data
            .insert(name.to_owned(), egui::FontData::from_owned(bytes));
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, name.to_owned());
        }
    }

    ctx.set_fonts(fonts);
}

fn same_window_title_match(left: &AppItem, right: &AppItem) -> bool {
    let left_title = window_title_key(left.window_title_match.as_str());
    let right_title = window_title_key(right.window_title_match.as_str());
    !left_title.is_empty() && left_title == right_title
}

fn window_title_key(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches('"')
        .to_ascii_lowercase()
}
