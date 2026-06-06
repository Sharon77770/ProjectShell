use eframe::egui::{self, Color32, RichText};

use crate::i18n::{I18n, Text};

pub const BG: Color32 = Color32::from_rgb(11, 13, 18);
pub const PANEL: Color32 = Color32::from_rgb(17, 21, 28);
pub const PANEL_ELEVATED: Color32 = Color32::from_rgb(23, 28, 38);
pub const BORDER: Color32 = Color32::from_rgb(42, 49, 64);
pub const ACCENT: Color32 = Color32::from_rgb(94, 234, 212);
pub const TEXT: Color32 = Color32::from_rgb(229, 231, 235);
pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(156, 163, 175);

#[derive(Debug)]
pub enum LauncherAction {
    OpenSettings,
    OpenHelp,
}

pub fn render_search_box(ui: &mut egui::Ui, search_query: &mut String, i18n: I18n) -> bool {
    let response = egui::Frame::none()
        .fill(PANEL_ELEVATED)
        .rounding(egui::Rounding::same(10.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(12.0, 6.0))
        .show(ui, |ui| {
            ui.add_sized(
                [ui.available_width(), 26.0],
                egui::TextEdit::singleline(search_query)
                    .hint_text(i18n.t(Text::SearchHint))
                    .font(egui::TextStyle::Button)
                    .text_color(TEXT),
            )
        })
        .inner;

    response.changed()
}

pub fn render_footer_hints(ui: &mut egui::Ui, i18n: I18n) -> Vec<LauncherAction> {
    let mut actions = Vec::new();

    egui::Frame::none()
        .fill(PANEL)
        .rounding(egui::Rounding::same(9.0))
        .stroke(egui::Stroke::new(1.0, BORDER))
        .inner_margin(egui::Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            ui.set_min_height(22.0);
            ui.horizontal(|ui| {
                let hint_width = (ui.available_width() - 108.0).max(96.0);
                ui.add_sized(
                    [hint_width, 22.0],
                    egui::Label::new(
                        RichText::new(i18n.t(Text::HintFooter))
                            .size(11.0)
                            .color(TEXT_SECONDARY),
                    ),
                );

                if ui
                    .add_sized(
                        [28.0, 22.0],
                        egui::Button::new(
                            RichText::new("?").size(12.0).strong().color(TEXT_SECONDARY),
                        )
                        .fill(PANEL_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, BORDER)),
                    )
                    .on_hover_text(i18n.t(Text::Help))
                    .clicked()
                {
                    actions.push(LauncherAction::OpenHelp);
                }

                if ui
                    .add_sized(
                        [66.0, 22.0],
                        egui::Button::new(
                            RichText::new(i18n.t(Text::Settings))
                                .size(11.0)
                                .color(TEXT_SECONDARY),
                        )
                        .fill(PANEL_ELEVATED)
                        .stroke(egui::Stroke::new(1.0, BORDER)),
                    )
                    .clicked()
                {
                    actions.push(LauncherAction::OpenSettings);
                }
            });
        });

    actions
}

pub fn truncate(value: &str, max_chars: usize) -> String {
    let display_value = sanitize_display_text(value);
    let mut chars = display_value.chars();
    let truncated = chars.by_ref().take(max_chars).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn sanitize_display_text(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut last_was_space = false;

    for ch in value.chars().filter(is_safe_display_char) {
        if ch.is_whitespace() {
            if !last_was_space {
                output.push(' ');
                last_was_space = true;
            }
        } else {
            output.push(ch);
            last_was_space = false;
        }
    }

    output.trim().to_owned()
}

fn is_safe_display_char(ch: &char) -> bool {
    if ch.is_control() {
        return false;
    }

    let value = *ch as u32;
    !matches!(
        value,
        0xE000..=0xF8FF
            | 0xF0000..=0xFFFFD
            | 0x100000..=0x10FFFD
            | 0xFE00..=0xFE0F
            | 0x25A0..=0x25FF
            | 0x1D400..=0x1D7FF
            | 0x1F000..=0x1FAFF
            | 0x2600..=0x27BF
            | 0xFFFD
    )
}
