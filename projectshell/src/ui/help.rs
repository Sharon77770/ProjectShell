use eframe::egui::{self, RichText};

use crate::i18n::{I18n, Language, Text};

use super::launcher::{BORDER, PANEL, PANEL_ELEVATED, TEXT, TEXT_SECONDARY};

#[derive(Debug)]
pub enum HelpAction {
    Back,
}

pub fn render_help(ui: &mut egui::Ui, i18n: I18n) -> Vec<HelpAction> {
    let mut actions = Vec::new();

    egui::Frame::none()
        .fill(PANEL)
        .rounding(egui::Rounding::same(14.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::same(16.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(i18n.t(Text::Help))
                        .strong()
                        .size(18.0)
                        .color(TEXT),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(i18n.t(Text::Back)).clicked() {
                        actions.push(HelpAction::Back);
                    }
                });
            });

            ui.add_space(12.0);
            egui::ScrollArea::vertical()
                .id_source("help_scroll")
                .max_height(420.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    help_section(ui, i18n.t(Text::HelpUsage), usage_items(i18n.language()));
                    ui.add_space(12.0);
                    help_section(
                        ui,
                        i18n.t(Text::HelpShortcuts),
                        shortcut_items(i18n.language()),
                    );
                });
        });

    actions
}

fn help_section(ui: &mut egui::Ui, title: &str, items: &[(&str, &str)]) {
    ui.label(RichText::new(title).strong().size(13.5).color(TEXT));
    ui.add_space(5.0);

    egui::Frame::none()
        .fill(PANEL_ELEVATED)
        .rounding(egui::Rounding::same(10.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 4.0;
            for (key, description) in items {
                help_row(ui, key, description);
            }
        });
}

fn help_row(ui: &mut egui::Ui, key: &str, description: &str) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [108.0, 24.0],
            egui::Label::new(
                RichText::new(key)
                    .monospace()
                    .strong()
                    .size(11.5)
                    .color(TEXT),
            ),
        );
        ui.add_sized(
            [(ui.available_width()).max(120.0), 24.0],
            egui::Label::new(RichText::new(description).size(11.5).color(TEXT_SECONDARY)),
        );
    });
}

fn usage_items(language: Language) -> &'static [(&'static str, &'static str)] {
    match language {
        Language::Korean => &[
            (
                "1",
                "실행중 앱을 선택하고 Ctrl+A로 현재 프로젝트에 귀속합니다.",
            ),
            ("2", "프로젝트를 선택하고 Enter로 작업공간을 재개합니다."),
            (
                "3",
                "앱을 선택하고 Enter로 실행중 창 포커스 또는 실행을 수행합니다.",
            ),
            (
                "4",
                "설정에서 프로젝트 이름, 작업공간 경로, 앱 실행 경로를 수정합니다.",
            ),
        ],
        Language::English => &[
            (
                "1",
                "Select a running app and press Ctrl+A to assign it to the current project.",
            ),
            (
                "2",
                "Select a project and press Enter to resume the workspace.",
            ),
            (
                "3",
                "Select an app and press Enter to focus an existing window or launch it.",
            ),
            (
                "4",
                "Use Settings to edit project names, workspace paths, and app paths.",
            ),
        ],
    }
}

fn shortcut_items(language: Language) -> &'static [(&'static str, &'static str)] {
    match language {
        Language::Korean => &[
            ("Win+`", "ProjectShell GUI 열기"),
            ("Esc", "GUI 숨기기 또는 이전 화면"),
            ("Enter", "선택 항목 포커스, 실행, 작업공간 재개"),
            ("Ctrl+A", "실행중 앱을 현재 프로젝트에 귀속"),
            ("Ctrl+O", "선택 프로젝트의 작업공간 폴더 열기"),
            ("Ctrl+,", "설정 열기 또는 닫기"),
            ("↑ / ↓", "목록 선택 이동"),
            ("← / →", "프로젝트와 앱 사이 이동"),
            ("Shift+↑/↓", "프로젝트 단위 이동"),
            ("Shift+←/→", "프로젝트 접기 또는 펼치기"),
            ("Space", "프로젝트 접기 또는 펼치기"),
        ],
        Language::English => &[
            ("Win+`", "Show the ProjectShell GUI"),
            ("Esc", "Hide the GUI or return to the previous screen"),
            ("Enter", "Focus, launch, or resume the selected item"),
            (
                "Ctrl+A",
                "Assign the selected running app to the current project",
            ),
            ("Ctrl+O", "Open the selected project's workspace folder"),
            ("Ctrl+,", "Open or close Settings"),
            ("↑ / ↓", "Move through the list"),
            ("← / →", "Move between a project and its apps"),
            ("Shift+↑/↓", "Move by project"),
            ("Shift+←/→", "Collapse or expand a project"),
            ("Space", "Collapse or expand a project"),
        ],
    }
}
