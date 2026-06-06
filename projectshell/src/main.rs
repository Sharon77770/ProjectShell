#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod i18n;
mod models;
mod services;
mod ui;

use app::ProjectShellApp;
use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ProjectShell")
            .with_inner_size([620.0, 520.0])
            .with_min_inner_size([620.0, 520.0])
            .with_max_inner_size([620.0, 520.0])
            .with_decorations(false)
            .with_transparent(false)
            .with_visible(false)
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "ProjectShell",
        native_options,
        Box::new(|cc| Box::new(ProjectShellApp::new(cc))),
    )
}
