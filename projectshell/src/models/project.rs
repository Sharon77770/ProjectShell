use serde::{Deserialize, Serialize};

use super::AppItem;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: String,
    pub workspace_path: String,
    pub apps: Vec<AppItem>,
}

impl Project {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        workspace_path: impl Into<String>,
        apps: Vec<AppItem>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            workspace_path: workspace_path.into(),
            apps,
        }
    }
}
