use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn log_error(message: impl AsRef<str>) {
    let _ = append_log("ERROR", message.as_ref());
}

pub fn log_info(message: impl AsRef<str>) {
    let _ = append_log("INFO", message.as_ref());
}

fn append_log(level: &str, message: &str) -> Result<(), String> {
    let path = log_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|err| format!("Failed to create {}: {err}", parent.display()))?;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|err| format!("Failed to open {}: {err}", path.display()))?;

    writeln!(file, "[{timestamp}] {level}: {message}")
        .map_err(|err| format!("Failed to write {}: {err}", path.display()))
}

fn log_file_path() -> PathBuf {
    PathBuf::from("logs").join("projectshell.log")
}
