use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::i18n::Language;
use crate::models::{AppItem, Project};

use super::window_service::{executable_file_name, normalize_process_name};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredProjects {
    pub selected_project_id: Option<String>,
    #[serde(default)]
    pub language: Language,
    pub projects: Vec<Project>,
}

pub fn load_or_create() -> Result<StoredProjects, String> {
    let path = projects_file_path();

    if !path.exists() {
        let state = default_state();
        save_state(
            &state.projects,
            state.selected_project_id.as_deref(),
            state.language,
        )?;
        return Ok(state);
    }

    let content = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read {}: {err}", path.display()))?;

    if content.trim().is_empty() {
        let state = default_state();
        save_state(
            &state.projects,
            state.selected_project_id.as_deref(),
            state.language,
        )?;
        return Ok(state);
    }

    let mut state = match serde_json::from_str::<StoredProjects>(&content) {
        Ok(state) => state,
        Err(object_error) => match serde_json::from_str::<Vec<Project>>(&content) {
            Ok(projects) => StoredProjects {
                selected_project_id: projects.first().map(|project| project.id.clone()),
                language: Language::default(),
                projects,
            },
            Err(list_error) => {
                return Err(format!(
                    "Failed to parse {} as ProjectShell data: {object_error}; list fallback: {list_error}",
                    path.display()
                ));
            }
        },
    };

    ensure_selected_project(&mut state);
    normalize_state(&mut state);
    Ok(state)
}

pub fn save_state(
    projects: &[Project],
    selected_project_id: Option<&str>,
    language: Language,
) -> Result<(), String> {
    let path = projects_file_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create {}: {err}", parent.display()))?;
    }

    let state = StoredProjects {
        selected_project_id: selected_project_id.map(str::to_owned),
        language,
        projects: projects.to_vec(),
    };

    let json = serde_json::to_string_pretty(&state)
        .map_err(|err| format!("Failed to serialize project data: {err}"))?;

    fs::write(&path, json).map_err(|err| format!("Failed to write {}: {err}", path.display()))
}

pub fn projects_file_path() -> PathBuf {
    PathBuf::from("config").join("projects.json")
}

pub fn default_state() -> StoredProjects {
    let projects = vec![
        Project::new(
            "project-projectshell",
            "ProjectShell",
            "Native Rust launcher for restoring this development workspace.",
            "",
            vec![
                AppItem::new("app-projectshell-vscode", "VSCode", "code", ".")
                    .with_process_name("code.exe"),
                AppItem::new(
                    "app-projectshell-terminal",
                    "Windows Terminal",
                    "wt.exe",
                    "",
                )
                .with_process_name("windowsterminal.exe"),
                AppItem::new("app-projectshell-figma", "Figma", "", "")
                    .with_process_name("figma.exe"),
                AppItem::new(
                    "app-projectshell-github-desktop",
                    "GitHub Desktop",
                    "GitHubDesktop.exe",
                    "",
                )
                .with_process_name("githubdesktop.exe"),
            ],
        ),
        Project::new(
            "project-jarvis",
            "Jarvis",
            "Automation and assistant project environment.",
            "",
            vec![
                AppItem::new("app-jarvis-vscode", "VSCode", "code", ".")
                    .with_process_name("code.exe"),
                AppItem::new(
                    "app-jarvis-docker",
                    "Docker Desktop",
                    r"C:\Program Files\Docker\Docker\Docker Desktop.exe",
                    "",
                )
                .with_process_name("docker desktop.exe"),
                AppItem::new(
                    "app-jarvis-docs",
                    "Chrome Docs",
                    "chrome.exe",
                    "https://docs.docker.com/",
                )
                .with_process_name("chrome.exe"),
            ],
        ),
        Project::new(
            "project-trendscope",
            "TrendScope",
            "Data and browser preview workspace.",
            "",
            vec![
                AppItem::new("app-trendscope-idea", "IntelliJ IDEA", "idea64.exe", "")
                    .with_process_name("idea64.exe"),
                AppItem::new(
                    "app-trendscope-mysql",
                    "MySQL Workbench",
                    r"C:\Program Files\MySQL\MySQL Workbench 8.0\MySQLWorkbench.exe",
                    "",
                )
                .with_process_name("mysqlworkbench.exe"),
                AppItem::new(
                    "app-trendscope-browser",
                    "Browser Preview",
                    "chrome.exe",
                    "http://localhost:3000",
                )
                .with_process_name("chrome.exe"),
            ],
        ),
    ];

    StoredProjects {
        selected_project_id: projects.first().map(|project| project.id.clone()),
        language: Language::default(),
        projects,
    }
}

fn normalize_state(state: &mut StoredProjects) {
    for project in &mut state.projects {
        for app in &mut project.apps {
            if app.process_name.trim().is_empty() {
                if let Some(name) = executable_file_name(app.executable_path.as_str()) {
                    app.process_name = normalize_process_name(name.as_str());
                }
            } else {
                app.process_name = normalize_process_name(app.process_name.as_str());
            }
            app.window_title_match = app.window_title_match.trim().to_owned();
        }
    }
}

fn ensure_selected_project(state: &mut StoredProjects) {
    let selected_is_valid = state
        .selected_project_id
        .as_deref()
        .is_some_and(|selected_id| {
            state
                .projects
                .iter()
                .any(|project| project.id == selected_id)
        });

    if !selected_is_valid {
        state.selected_project_id = state.projects.first().map(|project| project.id.clone());
    }
}
