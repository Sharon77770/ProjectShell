use std::collections::HashMap;

use eframe::egui::{self, Color32, RichText};

use crate::i18n::{I18n, Text};
use crate::models::{AppItem, AppStatus, Project};
use crate::services::{adapter_service, window_service::RunningWindow};
use crate::ui::project_list::LauncherRow;

use super::launcher::{truncate, ACCENT, BORDER, PANEL, TEXT, TEXT_SECONDARY};

pub fn render_selected_detail(
    ui: &mut egui::Ui,
    projects: &[Project],
    running_windows: &[RunningWindow],
    running_window_aliases: &mut HashMap<isize, String>,
    selected_row: Option<&LauncherRow>,
    app_statuses: &HashMap<String, AppStatus>,
    status_message: &str,
    status_is_error: bool,
    i18n: I18n,
) {
    egui::Frame::none()
        .fill(PANEL)
        .rounding(egui::Rounding::same(12.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(12.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.set_min_height(86.0);
            ui.set_max_height(86.0);

            match selected_row {
                Some(LauncherRow::Section { .. }) => {
                    ui.label(
                        RichText::new(i18n.t(Text::SelectRunningOrWorkspace))
                            .size(13.0)
                            .color(TEXT_SECONDARY),
                    );
                }
                Some(LauncherRow::RunningWindow { hwnd }) => {
                    let Some(window) = running_windows.iter().find(|window| window.hwnd == *hwnd)
                    else {
                        return;
                    };
                    let alias = running_window_aliases
                        .entry(*hwnd)
                        .or_insert_with(|| adapter_service::default_alias_for_window(window));
                    render_running_window_detail(ui, window, alias, i18n);
                }
                Some(LauncherRow::Project { project_id }) => {
                    let Some(project) = find_project(projects, project_id) else {
                        return;
                    };
                    render_project_detail(ui, project, app_statuses, i18n);
                }
                Some(LauncherRow::App { project_id, app_id }) => {
                    let Some(project) = find_project(projects, project_id) else {
                        return;
                    };
                    let Some(app) = project.apps.iter().find(|app| app.id == *app_id) else {
                        return;
                    };
                    render_app_detail(ui, project, app, app_statuses, i18n);
                }
                None => {
                    ui.label(
                        RichText::new(i18n.t(Text::NoRowSelected))
                            .size(13.0)
                            .color(TEXT_SECONDARY),
                    );
                }
            }

            if !status_message.trim().is_empty() {
                let color = if status_is_error {
                    Color32::from_rgb(239, 113, 113)
                } else {
                    Color32::from_rgb(94, 234, 154)
                };
                ui.add_sized(
                    [ui.available_width(), 14.0],
                    egui::Label::new(
                        RichText::new(truncate(status_message, 68))
                            .size(10.5)
                            .color(color),
                    ),
                );
            }
        });
}

fn render_project_detail(
    ui: &mut egui::Ui,
    project: &Project,
    app_statuses: &HashMap<String, AppStatus>,
    i18n: I18n,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_sized(
                [(ui.available_width() - 140.0).max(1.0), 18.0],
                egui::Label::new(
                    RichText::new(format!(
                        "{}: {}",
                        i18n.t(Text::SelectedProject),
                        truncate(project.name.as_str(), 25)
                    ))
                    .size(12.5)
                    .color(TEXT),
                ),
            );
            ui.add_sized(
                [ui.available_width(), 15.0],
                egui::Label::new(
                    RichText::new(truncate(project.description.trim(), 58))
                        .size(10.5)
                        .color(TEXT_SECONDARY),
                ),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            action_hint(ui, i18n.t(Text::EnterResume), 126.0);
        });
    });

    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{}:", i18n.t(Text::Apps)))
                .size(10.5)
                .color(TEXT_SECONDARY),
        );
        for app in project.apps.iter().take(4) {
            status_dot(ui, app_statuses.get(app.id.as_str()).copied());
            ui.add_sized(
                [76.0, 16.0],
                egui::Label::new(
                    RichText::new(truncate(app.name.as_str(), 9))
                        .size(10.5)
                        .color(TEXT_SECONDARY),
                ),
            );
        }
    });
}

fn render_app_detail(
    ui: &mut egui::Ui,
    project: &Project,
    app: &AppItem,
    app_statuses: &HashMap<String, AppStatus>,
    i18n: I18n,
) {
    let status = app_statuses
        .get(app.id.as_str())
        .copied()
        .unwrap_or(AppStatus::Unknown);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_sized(
                [(ui.available_width() - 92.0).max(1.0), 18.0],
                egui::Label::new(
                    RichText::new(format!(
                        "{}: {}",
                        i18n.t(Text::SelectedApp),
                        truncate(app.name.as_str(), 28)
                    ))
                    .size(12.5)
                    .color(TEXT),
                ),
            );
            ui.add_sized(
                [ui.available_width(), 15.0],
                egui::Label::new(
                    RichText::new(format!(
                        "{}: {} / {}: {}",
                        i18n.t(Text::Project),
                        truncate(project.name.as_str(), 22),
                        i18n.t(Text::Status),
                        status_label(status, i18n)
                    ))
                    .size(10.5)
                    .color(TEXT_SECONDARY),
                ),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            action_hint(ui, i18n.t(Text::EnterLaunch), 108.0);
        });
    });
}

fn render_running_window_detail(
    ui: &mut egui::Ui,
    window: &RunningWindow,
    alias: &mut String,
    i18n: I18n,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.add_sized(
                [(ui.available_width() - 230.0).max(1.0), 18.0],
                egui::Label::new(
                    RichText::new(format!(
                        "{}: {}",
                        i18n.t(Text::RunningApp),
                        truncate(window.process_name.as_str(), 25)
                    ))
                    .size(12.5)
                    .color(TEXT),
                ),
            );
            ui.add_sized(
                [(ui.available_width() - 230.0).max(1.0), 22.0],
                egui::TextEdit::singleline(alias)
                    .hint_text(i18n.t(Text::Alias))
                    .text_color(TEXT),
            );
            ui.add_sized(
                [(ui.available_width() - 230.0).max(1.0), 15.0],
                egui::Label::new(
                    RichText::new(truncate(window.title.as_str(), 58))
                        .size(10.5)
                        .color(TEXT_SECONDARY),
                ),
            );
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            action_hint(ui, i18n.t(Text::CtrlAssign), 116.0);
            ui.add_space(6.0);
            action_hint(ui, i18n.t(Text::EnterFocus), 96.0);
        });
    });
}

fn action_hint(ui: &mut egui::Ui, text: &str, width: f32) {
    egui::Frame::none()
        .fill(ACCENT)
        .rounding(egui::Rounding::same(5.0))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(91, 206, 190)))
        .inner_margin(egui::Margin::symmetric(8.0, 5.0))
        .show(ui, |ui| {
            ui.add_sized(
                [width - 16.0, 18.0],
                egui::Label::new(
                    RichText::new(text)
                        .size(11.5)
                        .strong()
                        .color(Color32::from_rgb(8, 26, 21)),
                ),
            );
        });
}

fn status_dot(ui: &mut egui::Ui, status: Option<AppStatus>) {
    let color = match status.unwrap_or(AppStatus::Unknown) {
        AppStatus::Running => Color32::from_rgb(83, 206, 135),
        AppStatus::Stopped => Color32::from_rgb(101, 111, 126),
        AppStatus::Unknown => Color32::from_rgb(210, 172, 83),
    };

    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 16.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 3.0, color);
}

fn status_label(status: AppStatus, i18n: I18n) -> &'static str {
    match status {
        AppStatus::Running => i18n.t(Text::Active),
        AppStatus::Stopped => i18n.t(Text::Stopped),
        AppStatus::Unknown => i18n.t(Text::Unknown),
    }
}

fn find_project<'a>(projects: &'a [Project], project_id: &str) -> Option<&'a Project> {
    projects.iter().find(|project| project.id == project_id)
}
