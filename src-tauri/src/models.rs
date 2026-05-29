use serde::{Deserialize, Serialize};

/// A bookable leaf timecode (Replicon Client -> Project -> Task).
/// `label` is the precomputed flat string the frontend fuzzy-searches,
/// e.g. "William Smith / Chevron Kits / QA".
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timecode {
    pub id: i64,
    pub client: String,
    pub project: String,
    pub task: String,
    pub active: bool,
    pub label: String,
}

impl Timecode {
    pub fn label(client: &str, project: &str, task: &str) -> String {
        format!("{client} / {project} / {task}")
    }
}

/// A single captured piece of work. `timecode_id` is nullable to support
/// capture-now-resolve-later (blank timecode entries).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: i64,
    pub timecode_id: Option<i64>,
    pub date: String, // ISO yyyy-mm-dd
    pub duration_minutes: i64,
    pub description: String,
    pub created_at: String,
}
