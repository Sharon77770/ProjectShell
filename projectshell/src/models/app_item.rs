use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppItem {
    pub id: String,
    pub name: String,
    pub executable_path: String,
    pub args: String,
    #[serde(default)]
    pub process_name: String,
    #[serde(default)]
    pub window_title_match: String,
}

impl AppItem {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        executable_path: impl Into<String>,
        args: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            executable_path: executable_path.into(),
            args: args.into(),
            process_name: String::new(),
            window_title_match: String::new(),
        }
    }

    pub fn with_process_name(mut self, process_name: impl Into<String>) -> Self {
        self.process_name = process_name.into();
        self
    }

    pub fn with_window_title_match(mut self, window_title_match: impl Into<String>) -> Self {
        self.window_title_match = window_title_match.into();
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppStatus {
    Running,
    Stopped,
    Unknown,
}
