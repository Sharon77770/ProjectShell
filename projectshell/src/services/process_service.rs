use std::collections::HashMap;

use crate::models::{AppItem, AppStatus};

use super::window_service::{window_matches_app, RunningWindow};

pub fn query_statuses_from_windows(
    apps: &[AppItem],
    running: &[RunningWindow],
) -> HashMap<String, AppStatus> {
    let mut statuses = HashMap::with_capacity(apps.len());
    for app in apps {
        statuses.insert(app.id.clone(), status_for_app(app, running));
    }

    statuses
}

fn status_for_app(app: &AppItem, running: &[RunningWindow]) -> AppStatus {
    if running.iter().any(|window| window_matches_app(app, window)) {
        AppStatus::Running
    } else if app.process_name.trim().is_empty() && app.executable_path.trim().is_empty() {
        AppStatus::Unknown
    } else {
        AppStatus::Stopped
    }
}
