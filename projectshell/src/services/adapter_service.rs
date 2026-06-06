use crate::models::AppItem;

use super::window_service::{executable_file_name, normalize_process_name, RunningWindow};

pub fn app_from_window(
    id: impl Into<String>,
    window: &RunningWindow,
    alias: Option<&str>,
) -> AppItem {
    let process_name = normalize_process_name(window.process_name.as_str());
    let executable_path = window
        .executable_path
        .clone()
        .unwrap_or_else(|| window.process_name.clone());
    let name = alias
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| display_name_for_window(window));

    AppItem::new(id, name, executable_path, "")
        .with_process_name(process_name)
        .with_window_title_match(window_title_match_for_window(window))
}

pub fn default_alias_for_window(window: &RunningWindow) -> String {
    let process = normalize_process_name(window.process_name.as_str());
    match process.as_str() {
        "code.exe" => "VSCode".to_owned(),
        "chrome.exe" => "Chrome".to_owned(),
        "windowsterminal.exe" | "wt.exe" => "Windows Terminal".to_owned(),
        "figma.exe" => "Figma".to_owned(),
        "githubdesktop.exe" => "GitHub Desktop".to_owned(),
        "discord.exe" => "Discord".to_owned(),
        "kakaotalk.exe" => "KakaoTalk".to_owned(),
        "systemsettings.exe" => "Settings".to_owned(),
        _ => window
            .process_name
            .trim_end_matches(".exe")
            .trim_end_matches(".EXE")
            .to_owned(),
    }
}

pub fn launch_target(app: &AppItem, workspace_path: &str) -> (String, Vec<String>) {
    let process = normalize_process_name(app.process_name.as_str());
    let executable_name = executable_file_name(app.executable_path.as_str())
        .map(|name| normalize_process_name(name.as_str()))
        .unwrap_or_default();
    let executable = app.executable_path.trim();
    let mut args = app
        .args
        .split_whitespace()
        .filter(|arg| !arg.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();

    if is_vscode(process.as_str(), executable_name.as_str()) {
        if !workspace_path.trim().is_empty() {
            args.clear();
            args.push(workspace_path.trim().to_owned());
        }
        let command = if executable.is_empty() {
            "code".to_owned()
        } else {
            executable.to_owned()
        };
        return (command, args);
    }

    if is_terminal(process.as_str(), executable_name.as_str()) && executable.is_empty() {
        return ("wt.exe".to_owned(), args);
    }

    (executable.to_owned(), args)
}

pub fn inferred_workspace_path(_window: &RunningWindow) -> Option<String> {
    None
}

pub fn window_title_match_for_window(window: &RunningWindow) -> String {
    let process = normalize_process_name(window.process_name.as_str());
    let title = window.title.trim();
    if title.is_empty() {
        return String::new();
    }

    match process.as_str() {
        "code.exe" | "cursor.exe" => vscode_workspace_title(title),
        "chrome.exe" | "msedge.exe" | "brave.exe" | "firefox.exe" => browser_tab_title(title),
        _ => title.to_owned(),
    }
}

fn display_name_for_window(window: &RunningWindow) -> String {
    let process = normalize_process_name(window.process_name.as_str());
    let title = window.title.trim();
    let app_name = match process.as_str() {
        "code.exe" => "VSCode",
        "chrome.exe" => "Chrome",
        "windowsterminal.exe" | "wt.exe" => "Windows Terminal",
        "figma.exe" => "Figma",
        "githubdesktop.exe" => "GitHub Desktop",
        "idea64.exe" => "IntelliJ IDEA",
        _ => window
            .process_name
            .trim_end_matches(".exe")
            .trim_end_matches(".EXE"),
    };

    if title.is_empty() {
        app_name.to_owned()
    } else {
        format!("{app_name} - {title}")
    }
}

fn vscode_workspace_title(title: &str) -> String {
    let parts = title
        .split(" - ")
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if parts.len() >= 3 {
        return parts[parts.len() - 2].to_owned();
    }

    if parts.len() == 2 {
        return parts[0].to_owned();
    }

    title.to_owned()
}

fn browser_tab_title(title: &str) -> String {
    for suffix in [
        " - Google Chrome",
        " - Microsoft Edge",
        " - Brave",
        " - Mozilla Firefox",
    ] {
        if let Some(tab_title) = title.strip_suffix(suffix) {
            return tab_title.trim().to_owned();
        }
    }

    title.to_owned()
}

fn is_vscode(process_name: &str, executable_name: &str) -> bool {
    process_name == "code.exe" || executable_name == "code.exe"
}

fn is_terminal(process_name: &str, executable_name: &str) -> bool {
    matches!(process_name, "windowsterminal.exe" | "wt.exe")
        || matches!(executable_name, "windowsterminal.exe" | "wt.exe")
}
