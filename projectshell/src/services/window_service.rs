use std::ffi::OsString;
use std::path::Path;

#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStringExt;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, BOOL, HWND, LPARAM};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{
    GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetShellWindow, GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId,
    IsWindowVisible,
};

use crate::models::AppItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningWindow {
    pub hwnd: isize,
    pub title: String,
    pub process_name: String,
    pub executable_path: Option<String>,
}

pub fn list_running_windows() -> Result<Vec<RunningWindow>, String> {
    #[cfg(target_os = "windows")]
    {
        list_running_windows_windows()
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(Vec::new())
    }
}

pub fn find_matching_window(app: &AppItem) -> Option<RunningWindow> {
    let windows = list_running_windows().ok()?;
    windows
        .into_iter()
        .find(|window| window_matches_app(app, window))
}

pub fn window_matches_app(app: &AppItem, window: &RunningWindow) -> bool {
    process_matches_app(app, window) && title_matches_app(app, window)
}

pub fn process_matches_app(app: &AppItem, window: &RunningWindow) -> bool {
    let running_process = normalize_process_name(window.process_name.as_str());
    if running_process.is_empty() {
        return false;
    }

    let mut candidates = Vec::new();
    let app_process = normalize_process_name(app.process_name.as_str());
    if !app_process.is_empty() {
        candidates.push(app_process);
    }

    if let Some(executable_name) = executable_file_name(app.executable_path.as_str()) {
        let executable_name = normalize_process_name(executable_name.as_str());
        if !executable_name.is_empty() {
            candidates.push(executable_name.clone());
            if !executable_name.ends_with(".exe") {
                candidates.push(format!("{executable_name}.exe"));
            }
        }
    }

    candidates
        .iter()
        .any(|candidate| running_process == *candidate)
}

fn title_matches_app(app: &AppItem, window: &RunningWindow) -> bool {
    let expected = normalize_window_title(app.window_title_match.as_str());
    if expected.is_empty() {
        return !is_title_sensitive_app(app, window);
    }

    let actual = normalize_window_title(window.title.as_str());
    if actual.is_empty() {
        return false;
    }

    actual == expected || actual.contains(expected.as_str())
}

fn is_title_sensitive_app(app: &AppItem, window: &RunningWindow) -> bool {
    let app_process = normalize_process_name(app.process_name.as_str());
    let running_process = normalize_process_name(window.process_name.as_str());
    let executable = executable_file_name(app.executable_path.as_str())
        .map(|name| normalize_process_name(name.as_str()))
        .unwrap_or_default();

    [
        app_process.as_str(),
        running_process.as_str(),
        executable.as_str(),
    ]
    .iter()
    .any(|process| {
        matches!(
            *process,
            "code.exe"
                | "cursor.exe"
                | "chrome.exe"
                | "msedge.exe"
                | "brave.exe"
                | "firefox.exe"
                | "idea64.exe"
                | "webstorm64.exe"
                | "pycharm64.exe"
                | "phpstorm64.exe"
                | "rider64.exe"
                | "clion64.exe"
                | "goland64.exe"
        )
    })
}

pub fn normalize_window_title(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches('"')
        .to_ascii_lowercase()
}

pub fn normalize_process_name(value: &str) -> String {
    value.trim().trim_matches('"').to_ascii_lowercase()
}

pub fn executable_file_name(value: &str) -> Option<String> {
    let cleaned = value.trim().trim_matches('"');
    if cleaned.is_empty() {
        return None;
    }
    Path::new(cleaned)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .or_else(|| Some(cleaned.to_owned()))
}

#[cfg(target_os = "windows")]
fn list_running_windows_windows() -> Result<Vec<RunningWindow>, String> {
    let mut windows = Vec::<RunningWindow>::new();
    let windows_ptr = &mut windows as *mut Vec<RunningWindow>;

    unsafe {
        EnumWindows(Some(enum_window_proc), windows_ptr as LPARAM);
    }

    windows.sort_by(|a, b| {
        a.process_name
            .to_ascii_lowercase()
            .cmp(&b.process_name.to_ascii_lowercase())
            .then_with(|| {
                a.title
                    .to_ascii_lowercase()
                    .cmp(&b.title.to_ascii_lowercase())
            })
    });
    windows.dedup_by(|a, b| a.hwnd == b.hwnd);
    Ok(windows)
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    if !is_candidate_window(hwnd) {
        return 1;
    }

    let title = window_title(hwnd);
    if title.trim().is_empty() {
        return 1;
    }

    let mut process_id = 0u32;
    GetWindowThreadProcessId(hwnd, &mut process_id);
    if process_id == 0 || process_id == GetCurrentProcessId() {
        return 1;
    }

    let executable_path = executable_path_for_process(process_id);
    let process_name = executable_path
        .as_deref()
        .and_then(executable_file_name)
        .unwrap_or_else(|| format!("pid-{process_id}"));

    if should_exclude_window(&title, &process_name) {
        return 1;
    }

    let windows = &mut *(lparam as *mut Vec<RunningWindow>);
    windows.push(RunningWindow {
        hwnd,
        title,
        process_name,
        executable_path,
    });

    1
}

#[cfg(target_os = "windows")]
unsafe fn is_candidate_window(hwnd: HWND) -> bool {
    if hwnd == 0 || hwnd == GetShellWindow() {
        return false;
    }
    IsWindowVisible(hwnd) != 0 && GetWindowTextLengthW(hwnd) > 0
}

#[cfg(target_os = "windows")]
unsafe fn window_title(hwnd: HWND) -> String {
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return String::new();
    }

    let mut buffer = vec![0u16; len as usize + 1];
    let copied = GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
    if copied <= 0 {
        return String::new();
    }

    OsString::from_wide(&buffer[..copied as usize])
        .to_string_lossy()
        .trim()
        .to_owned()
}

#[cfg(target_os = "windows")]
unsafe fn executable_path_for_process(process_id: u32) -> Option<String> {
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
    if handle == 0 {
        return None;
    }

    let mut buffer = vec![0u16; 32768];
    let mut size = buffer.len() as u32;
    let result = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
    CloseHandle(handle);

    if result == 0 || size == 0 {
        return None;
    }

    Some(
        OsString::from_wide(&buffer[..size as usize])
            .to_string_lossy()
            .into_owned(),
    )
}

fn should_exclude_window(title: &str, process_name: &str) -> bool {
    let title = title.trim().to_ascii_lowercase();
    let process = normalize_process_name(process_name);

    if title.is_empty()
        || title == "program manager"
        || title == "default ime"
        || title == "msctfime ui"
    {
        return true;
    }

    matches!(
        process.as_str(),
        "projectshell.exe"
            | "applicationframehost.exe"
            | "shellexperiencehost.exe"
            | "searchhost.exe"
            | "startmenuexperiencehost.exe"
            | "textinputhost.exe"
            | "dwm.exe"
            | "lockapp.exe"
            | "ctfmon.exe"
    )
}
