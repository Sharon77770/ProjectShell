use std::collections::{HashMap, HashSet};

use eframe::egui::{self, Color32, RichText};

use crate::i18n::{I18n, Text};
use crate::models::{AppStatus, Project};
use crate::services::window_service::RunningWindow;

use super::launcher::{truncate, ACCENT, BORDER, PANEL, PANEL_ELEVATED, TEXT, TEXT_SECONDARY};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LauncherRow {
    Section { label: String },
    RunningWindow { hwnd: isize },
    Project { project_id: String },
    App { project_id: String, app_id: String },
}

#[derive(Debug)]
pub enum TreeAction {
    SelectIndex(usize),
}

pub fn render_tree_view(
    ui: &mut egui::Ui,
    projects: &[Project],
    running_windows: &[RunningWindow],
    running_window_aliases: &HashMap<isize, String>,
    rows: &[LauncherRow],
    i18n: I18n,
    selected_row_index: usize,
    expanded_project_ids: &HashSet<String>,
    search_active: bool,
    app_statuses: &HashMap<String, AppStatus>,
) -> Vec<TreeAction> {
    let mut actions = Vec::new();

    egui::Frame::none()
        .fill(PANEL)
        .rounding(egui::Rounding::same(12.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(8.0, 8.0))
        .show(ui, |ui| {
            ui.set_min_height(224.0);
            ui.set_max_height(224.0);

            if rows.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.label(
                        RichText::new(i18n.t(Text::NoWorkspaceFound))
                            .size(14.0)
                            .color(TEXT_SECONDARY),
                    );
                });
                return;
            }

            egui::ScrollArea::vertical()
                .id_source("workspace_tree")
                .max_height(224.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    for (row_index, row) in rows.iter().enumerate() {
                        let selected = row_index == selected_row_index;
                        let response = match row {
                            LauncherRow::Section { label } => render_section_row(ui, label),
                            LauncherRow::RunningWindow { hwnd } => {
                                let Some(window) =
                                    running_windows.iter().find(|window| window.hwnd == *hwnd)
                                else {
                                    continue;
                                };
                                render_running_window_row(
                                    ui,
                                    window,
                                    running_window_aliases.get(hwnd).map(String::as_str),
                                    i18n,
                                    selected,
                                )
                            }
                            LauncherRow::Project { project_id } => {
                                let Some(project) = find_project(projects, project_id) else {
                                    continue;
                                };
                                let expanded =
                                    search_active || expanded_project_ids.contains(project_id);
                                render_project_row(
                                    ui,
                                    project,
                                    selected,
                                    expanded,
                                    app_statuses,
                                    i18n,
                                )
                            }
                            LauncherRow::App { project_id, app_id } => {
                                let Some(project) = find_project(projects, project_id) else {
                                    continue;
                                };
                                let Some(app) = project.apps.iter().find(|app| app.id == *app_id)
                                else {
                                    continue;
                                };
                                render_app_row(
                                    ui,
                                    app.name.as_str(),
                                    app_id,
                                    selected,
                                    app_statuses,
                                    i18n,
                                    !app.executable_path.trim().is_empty(),
                                )
                            }
                        };

                        if response.clicked() {
                            actions.push(TreeAction::SelectIndex(row_index));
                        }

                        if selected {
                            ui.scroll_to_rect(response.rect, None);
                        }
                    }
                });
        });

    actions
}

fn render_section_row(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 26.0), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.text(
        rect.left_center() + egui::vec2(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(10.5),
        TEXT_SECONDARY,
    );
    response
}

fn render_running_window_row(
    ui: &mut egui::Ui,
    window: &RunningWindow,
    alias: Option<&str>,
    i18n: I18n,
    selected: bool,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 32.0), egui::Sense::click());
    let painter = ui.painter_at(rect);

    let fill = row_fill(selected, response.hovered());
    painter.rect_filled(rect, egui::Rounding::same(7.0), fill);
    if selected {
        painter.rect_stroke(
            rect,
            egui::Rounding::same(7.0),
            egui::Stroke::new(1.0, Color32::from_rgb(64, 92, 124)),
        );
        painter.rect_filled(
            egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height())),
            egui::Rounding::same(2.0),
            ACCENT,
        );
    }

    painter.circle_filled(
        rect.left_center() + egui::vec2(30.0, 0.0),
        3.0,
        Color32::from_rgb(83, 206, 135),
    );
    let label = alias
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!("{value} - {}", window.process_name))
        .unwrap_or_else(|| format!("{} - {}", window.process_name, window.title));
    painter.text(
        rect.left_center() + egui::vec2(43.0, 0.0),
        egui::Align2::LEFT_CENTER,
        truncate(label.as_str(), 48),
        egui::FontId::proportional(12.5),
        TEXT,
    );
    if selected || response.hovered() {
        painter.text(
            rect.right_center() - egui::vec2(12.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            i18n.t(Text::CtrlAssign),
            egui::FontId::proportional(10.5),
            TEXT_SECONDARY,
        );
    }

    response.on_hover_text(window.title.as_str())
}

fn render_project_row(
    ui: &mut egui::Ui,
    project: &Project,
    selected: bool,
    expanded: bool,
    app_statuses: &HashMap<String, AppStatus>,
    i18n: I18n,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 32.0), egui::Sense::click());
    let painter = ui.painter_at(rect);

    let fill = row_fill(selected, response.hovered());
    painter.rect_filled(rect, egui::Rounding::same(7.0), fill);
    if selected {
        painter.rect_stroke(
            rect,
            egui::Rounding::same(7.0),
            egui::Stroke::new(1.0, Color32::from_rgb(74, 110, 152)),
        );
        painter.rect_filled(
            egui::Rect::from_min_size(rect.left_top(), egui::vec2(3.0, rect.height())),
            egui::Rounding::same(2.0),
            ACCENT,
        );
    }

    painter.text(
        rect.left_center() + egui::vec2(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        if expanded { "v" } else { ">" },
        egui::FontId::proportional(12.5),
        TEXT_SECONDARY,
    );
    painter.text(
        rect.left_center() + egui::vec2(30.0, 0.0),
        egui::Align2::LEFT_CENTER,
        truncate(project.name.as_str(), 31),
        egui::FontId::proportional(13.0),
        TEXT,
    );
    painter.text(
        rect.right_center() - egui::vec2(12.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        project_meta(project, app_statuses, i18n),
        egui::FontId::proportional(11.0),
        TEXT_SECONDARY,
    );

    response.on_hover_text(project.name.as_str())
}

fn render_app_row(
    ui: &mut egui::Ui,
    app_name: &str,
    app_id: &str,
    selected: bool,
    app_statuses: &HashMap<String, AppStatus>,
    i18n: I18n,
    has_path: bool,
) -> egui::Response {
    let width = ui.available_width();
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, 32.0), egui::Sense::click());
    let painter = ui.painter_at(rect);

    let fill = row_fill(selected, response.hovered());
    painter.rect_filled(rect, egui::Rounding::same(7.0), fill);
    if selected {
        painter.rect_stroke(
            rect,
            egui::Rounding::same(7.0),
            egui::Stroke::new(1.0, Color32::from_rgb(64, 92, 124)),
        );
    }

    let status = app_statuses
        .get(app_id)
        .copied()
        .unwrap_or(AppStatus::Unknown);
    painter.circle_filled(
        rect.left_center() + egui::vec2(37.0, 0.0),
        3.0,
        status_color(status),
    );
    painter.text(
        rect.left_center() + egui::vec2(49.0, 0.0),
        egui::Align2::LEFT_CENTER,
        truncate(app_name, 34),
        egui::FontId::proportional(12.5),
        TEXT,
    );

    let hint = if selected || response.hovered() {
        if has_path {
            i18n.t(Text::EnterLaunch)
        } else {
            i18n.t(Text::MissingPath)
        }
    } else {
        ""
    };
    painter.text(
        rect.right_center() - egui::vec2(12.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        hint,
        egui::FontId::proportional(10.5),
        TEXT_SECONDARY,
    );

    response.on_hover_text(app_name)
}

fn row_fill(selected: bool, hovered: bool) -> Color32 {
    if selected {
        Color32::from_rgb(29, 37, 47)
    } else if hovered {
        PANEL_ELEVATED
    } else {
        PANEL
    }
}

fn find_project<'a>(projects: &'a [Project], project_id: &str) -> Option<&'a Project> {
    projects.iter().find(|project| project.id == project_id)
}

fn project_meta(
    project: &Project,
    app_statuses: &HashMap<String, AppStatus>,
    i18n: I18n,
) -> String {
    let running = project
        .apps
        .iter()
        .filter(|app| app_statuses.get(app.id.as_str()) == Some(&AppStatus::Running))
        .count();
    format!(
        "{} {} / {} {}",
        project.apps.len(),
        i18n.t(Text::Apps),
        running,
        i18n.t(Text::Active)
    )
}

fn status_color(status: AppStatus) -> Color32 {
    match status {
        AppStatus::Running => Color32::from_rgb(83, 206, 135),
        AppStatus::Stopped => Color32::from_rgb(101, 111, 126),
        AppStatus::Unknown => Color32::from_rgb(210, 172, 83),
    }
}
