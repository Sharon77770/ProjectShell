use std::path::Path;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::models::{AppItem, Project};

use super::adapter_service;
use super::focus_service;
use super::log_service;
use super::window_service::{find_matching_window, RunningWindow};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Clone, Default)]
pub struct ResumeSummary {
    pub project_name: String,
    pub focused: usize,
    pub launched: usize,
    pub failed: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum AppActivation {
    Focused(RunningWindow),
    Launched,
}

pub fn resume_project(project: &Project) -> ResumeSummary {
    let mut summary = ResumeSummary {
        project_name: project.name.clone(),
        ..ResumeSummary::default()
    };

    for app in &project.apps {
        match activate_app(app, project.workspace_path.as_str()) {
            Ok(AppActivation::Focused(_)) => summary.focused += 1,
            Ok(AppActivation::Launched) => summary.launched += 1,
            Err(err) => {
                summary.failed += 1;
                log_service::log_error(format!("Resume failed for {}: {err}", app.name));
                summary.failures.push(format!("{}: {err}", app.name));
            }
        }
    }

    summary
}

pub fn activate_app(app: &AppItem, workspace_path: &str) -> Result<AppActivation, String> {
    if let Some(window) = find_matching_window(app) {
        focus_service::focus_window(window.hwnd)?;
        return Ok(AppActivation::Focused(window));
    }

    launch_app(app, workspace_path)?;
    Ok(AppActivation::Launched)
}

pub fn launch_app(app: &AppItem, workspace_path: &str) -> Result<(), String> {
    let (executable, args) = adapter_service::launch_target(app, workspace_path);
    let executable = executable.trim();
    if executable.is_empty() {
        return Err(format!("{} has no executable path.", app.name));
    }

    let mut command = Command::new(executable);
    for arg in args {
        command.arg(arg);
    }

    let workspace_path = workspace_path.trim();
    if !workspace_path.is_empty() {
        let path = Path::new(workspace_path);
        if path.exists() {
            command.current_dir(path);
        }
    }

    #[cfg(target_os = "windows")]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }

    command
        .spawn()
        .map(|_| {
            log_service::log_info(format!("Launched {}", app.name));
        })
        .map_err(|err| format!("Failed to launch {}: {err}", app.name))
}

pub fn open_workspace(workspace_path: &str) -> Result<(), String> {
    let workspace_path = workspace_path.trim();
    if workspace_path.is_empty() {
        return Err("Workspace path is empty.".to_owned());
    }

    let path = Path::new(workspace_path);
    if !path.exists() {
        return Err(format!("Workspace path does not exist: {}", path.display()));
    }

    spawn_path_opener(path)
}

#[cfg(target_os = "windows")]
fn spawn_path_opener(path: &Path) -> Result<(), String> {
    let mut command = Command::new("explorer.exe");
    command.arg(path).creation_flags(CREATE_NO_WINDOW);
    command
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("Failed to open workspace in Explorer: {err}"))
}

#[cfg(target_os = "macos")]
fn spawn_path_opener(path: &Path) -> Result<(), String> {
    Command::new("open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("Failed to open workspace: {err}"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn spawn_path_opener(path: &Path) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|err| format!("Failed to open workspace: {err}"))
}
