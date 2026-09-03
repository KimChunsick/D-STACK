// store/plan.rs
// plan.json: milestones, plans and tasks as typed records (ported in P3).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, utc_now};
use crate::core::roots::git_out;
use crate::store::plan_graph::{refresh, render_roadmap, render_state};

/// What the first milestone of a run writes — compact, exactly as _plan_ensure prints it.
pub const SEED: &str = "{\"v\":2,\"milestones\":[],\"plans\":[]}\n";

/// The field order of every struct here is the order jq wrote them in, because to_json has to
/// reproduce the file byte for byte (design D-02).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub slug: String,
    pub order: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub milestone: String,
    pub slug: String,
    pub files: Vec<String>,
    pub deps: Vec<String>,
    pub status: String,
    pub worktree: String,
    pub started_at: String,
    pub done_at: String,
    pub tasks: Vec<Task>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub slug: String,
    pub covers: Vec<String>,
    pub files: Vec<String>,
    pub deps: Vec<String>,
    pub commit: String,
    pub done_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanDoc {
    pub v: u32,
    pub milestones: Vec<Milestone>,
    pub plans: Vec<Plan>,
}

pub fn path(dir: &Path) -> PathBuf {
    dir.join("plan.json")
}

pub fn exists(dir: &Path) -> bool {
    path(dir).is_file()
}

/// _plan_ensure(): the first milestone or plan of a run creates the file, later calls do nothing.
pub fn ensure(dir: &Path) -> Result<()> {
    if exists(dir) {
        return Ok(());
    }
    write_file(&path(dir), SEED)
}

pub fn load(dir: &Path) -> Result<PlanDoc> {
    let file = path(dir);
    let text = std::fs::read_to_string(&file)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", file.display())))?;
    serde_json::from_str(&text)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", file.display())))
}

fn write_file(file: &Path, text: &str) -> Result<()> {
    atomic_write(file, text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))
}

impl PlanDoc {
    /// jq's pretty print: two spaces, `"key": value`, `[]` for an empty array, trailing newline.
    pub fn to_json(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).expect("plan.json serialises");
        text.push('\n');
        text
    }

    /// _plan_commit(): refresh the derived statuses, write plan.json, regenerate both documents.
    /// One function so no mutation path can forget half of it; the caller holds the lock.
    pub fn commit(&self, dir: &Path, run_id: &str, worktree_for_last_commit: &Path) -> Result<()> {
        let mut doc = self.clone();
        refresh(&mut doc);
        write_file(&path(dir), &doc.to_json())?;
        write_file(&dir.join("ROADMAP.md"), &render_roadmap(&doc, run_id))?;
        let last = git_out(
            Some(worktree_for_last_commit),
            &["rev-parse", "--short", "HEAD"],
        )
        .unwrap_or_else(|| "none".to_string());
        write_file(
            &dir.join("STATE.md"),
            &render_state(&doc, run_id, &last, &utc_now()),
        )
    }

    pub fn plan(&self, id: &str) -> Option<&Plan> {
        self.plans.iter().find(|p| p.id == id)
    }

    pub fn plan_mut(&mut self, id: &str) -> Option<&mut Plan> {
        self.plans.iter_mut().find(|p| p.id == id)
    }

    pub fn plan_ids(&self) -> Vec<String> {
        self.plans.iter().map(|p| p.id.clone()).collect()
    }

    pub fn task_ids(&self) -> Vec<String> {
        self.plans
            .iter()
            .flat_map(|p| p.tasks.iter().map(|t| t.id.clone()))
            .collect()
    }

    /// _plan_field(): the field of a plan as jq -r prints it, "" when the plan or the field is
    /// missing. An array field prints as jq's pretty JSON, which is what `jq -r` does.
    pub fn field(&self, plan_id: &str, field: &str) -> String {
        let plan = match self.plan(plan_id) {
            Some(plan) => plan,
            None => return String::new(),
        };
        let value = serde_json::to_value(plan).expect("a plan serialises");
        match value.get(field) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(serde_json::Value::Null) | None => String::new(),
            Some(other) => serde_json::to_string_pretty(other).expect("a field serialises"),
        }
    }
}
