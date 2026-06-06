use eframe::egui::{self, Color32, RichText};

use crate::i18n::{I18n, Language, Text};
use crate::models::{AppItem, Project};

use super::launcher::{ACCENT, BORDER, PANEL, PANEL_ELEVATED, TEXT, TEXT_SECONDARY};

#[derive(Debug)]
pub enum SettingsAction {
    Back,
    AddProject,
    DeleteSelected,
    AddApp,
    DeleteApp(String),
    SetLanguage(Language),
}

#[derive(Debug, Default)]
pub struct SettingsResponse {
    pub changed: bool,
    pub actions: Vec<SettingsAction>,
}

pub fn render_settings(
    ui: &mut egui::Ui,
    project: Option<&mut Project>,
    selected_app_id: Option<&str>,
    focus_app_id: Option<&str>,
    i18n: I18n,
) -> SettingsResponse {
    let mut response = SettingsResponse::default();

    egui::Frame::none()
        .fill(PANEL)
        .rounding(egui::Rounding::same(14.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(i18n.t(Text::Settings))
                        .strong()
                        .size(18.0)
                        .color(TEXT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(i18n.t(Text::Back)).clicked() {
                        response.actions.push(SettingsAction::Back);
                    }
                    if ui
                        .small_button(format!("+ {}", i18n.t(Text::Project)))
                        .clicked()
                    {
                        response.actions.push(SettingsAction::AddProject);
                    }
                });
            });

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(i18n.t(Text::Language))
                        .size(11.0)
                        .color(TEXT_SECONDARY),
                );
                if language_button(ui, "한국어", i18n.language() == Language::Korean).clicked() {
                    response
                        .actions
                        .push(SettingsAction::SetLanguage(Language::Korean));
                }
                if language_button(ui, "English", i18n.language() == Language::English).clicked() {
                    response
                        .actions
                        .push(SettingsAction::SetLanguage(Language::English));
                }
            });

            ui.add_space(12.0);

            let Some(project) = project else {
                ui.label(
                    RichText::new(i18n.t(Text::NoProjectSelected))
                        .size(13.0)
                        .color(TEXT_SECONDARY),
                );
                return;
            };

            egui::ScrollArea::vertical()
                .id_source("settings_scroll")
                .max_height(360.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    response.changed |= project_field(ui, i18n.t(Text::Name), &mut project.name);
                    response.changed |=
                        project_field(ui, i18n.t(Text::Description), &mut project.description);
                    response.changed |=
                        project_field(ui, i18n.t(Text::Workspace), &mut project.workspace_path);

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.small_button(i18n.t(Text::DeleteProject)).clicked() {
                            response.actions.push(SettingsAction::DeleteSelected);
                        }
                        if ui
                            .small_button(format!("+ {}", i18n.t(Text::NewApp)))
                            .clicked()
                        {
                            response.actions.push(SettingsAction::AddApp);
                        }
                    });

                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(i18n.t(Text::Apps))
                            .strong()
                            .size(13.0)
                            .color(TEXT),
                    );
                    ui.add_space(4.0);

                    for app in &mut project.apps {
                        let selected = selected_app_id == Some(app.id.as_str());
                        let request_focus = focus_app_id == Some(app.id.as_str());
                        response.changed |= app_row(
                            ui,
                            app,
                            selected,
                            request_focus,
                            &mut response.actions,
                            i18n,
                        );
                        ui.add_space(6.0);
                    }
                });
        });

    response
}

fn language_button(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let text = if selected {
        RichText::new(label).strong().color(TEXT)
    } else {
        RichText::new(label).color(TEXT_SECONDARY)
    };
    ui.add_sized([70.0, 24.0], egui::Button::new(text))
}

fn project_field(ui: &mut egui::Ui, label: &str, value: &mut String) -> bool {
    let mut changed = false;
    ui.label(RichText::new(label).size(11.0).color(TEXT_SECONDARY));
    if ui
        .add_sized(
            [ui.available_width(), 28.0],
            egui::TextEdit::singleline(value).text_color(TEXT),
        )
        .changed()
    {
        changed = true;
    }
    ui.add_space(6.0);
    changed
}

fn app_row(
    ui: &mut egui::Ui,
    app: &mut AppItem,
    selected: bool,
    request_focus: bool,
    actions: &mut Vec<SettingsAction>,
    i18n: I18n,
) -> bool {
    let mut changed = false;
    let frame_response = egui::Frame::none()
        .fill(if selected {
            Color32::from_rgb(29, 37, 47)
        } else {
            PANEL_ELEVATED
        })
        .rounding(egui::Rounding::same(8.0))
        .stroke(if selected {
            egui::Stroke::new(1.0, ACCENT)
        } else {
            egui::Stroke::new(1.0, BORDER)
        })
        .inner_margin(egui::Margin::symmetric(8.0, 6.0))
        .show(ui, |ui| {
            ui.push_id(app.id.as_str(), |ui| {
                ui.horizontal(|ui| {
                    let name_response = ui.add_sized(
                        [112.0, 24.0],
                        egui::TextEdit::singleline(&mut app.name).text_color(TEXT),
                    );
                    if request_focus {
                        name_response.request_focus();
                    }
                    if name_response.changed() {
                        changed = true;
                    }
                    if ui.small_button(i18n.t(Text::Delete)).clicked() {
                        actions.push(SettingsAction::DeleteApp(app.id.clone()));
                    }
                });
                if ui
                    .add_sized(
                        [ui.available_width(), 24.0],
                        egui::TextEdit::singleline(&mut app.executable_path)
                            .hint_text(i18n.t(Text::ExecutablePath)),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add_sized(
                        [ui.available_width(), 24.0],
                        egui::TextEdit::singleline(&mut app.process_name)
                            .hint_text(i18n.t(Text::AppProcessHint)),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add_sized(
                        [ui.available_width(), 24.0],
                        egui::TextEdit::singleline(&mut app.window_title_match)
                            .hint_text(i18n.t(Text::WindowTitleMatch)),
                    )
                    .changed()
                {
                    changed = true;
                }
                if ui
                    .add_sized(
                        [ui.available_width(), 24.0],
                        egui::TextEdit::singleline(&mut app.args).hint_text(i18n.t(Text::Args)),
                    )
                    .changed()
                {
                    changed = true;
                }
            });
        });
    if selected {
        ui.scroll_to_rect(frame_response.response.rect, Some(egui::Align::Center));
    }
    changed
}
