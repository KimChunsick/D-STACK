use crate::core::mode::{Mode, Provider};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryRecord {
    pub reference: String,
    pub kind: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct History {
    pub provider: Provider,
    pub session: String,
    pub cwd: String,
    pub path: String,
    pub sha256: String,
    pub records: Vec<HistoryRecord>,
    pub warnings: Vec<String>,
    pub omitted: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    pub reference: String,
    pub path: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkItem {
    pub id: String,
    pub plan: String,
    pub state: String,
    pub title: String,
    pub covers: Vec<String>,
    pub files: Vec<String>,
    pub deps: Vec<String>,
    pub commit: String,
    pub refs: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub run_id: String,
    pub worktree: String,
    pub mode: Mode,
    pub owner_session: String,
    pub fingerprint: String,
    pub documents: Vec<Document>,
    pub items: Vec<WorkItem>,
}
