// Validate evidence references and unequal detail budgets before a summary can be resumed.
use super::types::{History, Snapshot};
use crate::core::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Brief {
    pub id: String,
    pub summary: String,
    pub refs: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Active {
    pub id: String,
    pub changes: String,
    pub attempts: String,
    pub blockers: String,
    pub next_steps: Vec<String>,
    pub refs: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Uncertainty {
    pub summary: String,
    pub refs: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    pub completed: Vec<Brief>,
    pub active: Vec<Active>,
    pub pending: Vec<Brief>,
    pub uncertainties: Vec<Uncertainty>,
    pub next_actions: Vec<String>,
}

pub fn validate(text: &str, state: &Snapshot, history: &History) -> Result<Summary> {
    if text.len() > 96 * 1024 {
        return Err(invalid("exceeds 96 KiB"));
    }
    let result: Summary =
        serde_json::from_str(text).map_err(|e| invalid(format!("invalid JSON: {e}")))?;
    let valid_refs: BTreeSet<&str> = state
        .documents
        .iter()
        .map(|d| d.reference.as_str())
        .chain(history.records.iter().map(|r| r.reference.as_str()))
        .collect();
    let mut seen = BTreeSet::new();
    for (items, category, limit) in
        [(&result.completed, "completed", 400), (&result.pending, "pending", 240)]
    {
        for item in items {
            task(&item.id, category, state, &mut seen)?;
            bounded(&item.summary, limit)?;
            references(&item.refs, &valid_refs, Some(&item.id))?;
        }
    }
    for item in &result.active {
        task(&item.id, "active", state, &mut seen)?;
        for detail in [&item.changes, &item.attempts, &item.blockers] {
            bounded(detail, 4000)?;
        }
        steps(&item.next_steps)?;
        references(&item.refs, &valid_refs, Some(&item.id))?;
    }
    if seen.len() != state.items.len() {
        return Err(invalid("does not cover every task exactly once"));
    }
    if result.uncertainties.len() > 50 {
        return Err(invalid("too many uncertainties"));
    }
    for item in &result.uncertainties {
        bounded(&item.summary, 2000)?;
        references(&item.refs, &valid_refs, None)?;
    }
    steps(&result.next_actions)?;
    Ok(result)
}

fn task<'a>(
    id: &'a str,
    category: &str,
    state: &Snapshot,
    seen: &mut BTreeSet<&'a str>,
) -> Result<()> {
    if !seen.insert(id) {
        return Err(invalid(format!("duplicate task {id}")));
    }
    if !state.items.iter().any(|i| i.id == id && i.state == category) {
        return Err(invalid(format!("unknown task or wrong state: {id}/{category}")));
    }
    Ok(())
}
fn bounded(value: &str, max: usize) -> Result<()> {
    if value.trim().is_empty() || value.chars().count() > max {
        return Err(invalid("empty or oversized detail"));
    }
    Ok(())
}
fn steps(values: &[String]) -> Result<()> {
    if values.is_empty() || values.len() > 20 {
        return Err(invalid("requires 1–20 next steps"));
    }
    for value in values {
        bounded(value, 1000)?;
    }
    Ok(())
}
fn references(values: &[String], valid: &BTreeSet<&str>, task: Option<&str>) -> Result<()> {
    if values.is_empty()
        || values.len() > 30
        || values.iter().collect::<BTreeSet<_>>().len() != values.len()
        || values.iter().any(|v| !valid.contains(v.as_str()))
    {
        return Err(invalid("missing or unknown evidence reference"));
    }
    if let Some(id) = task {
        if !values.contains(&format!("task:{id}")) {
            return Err(invalid(format!("missing task:{id} reference")));
        }
    }
    Ok(())
}
fn invalid(why: impl AsRef<str>) -> Error {
    Error::failed(format!("handoff summary: {}", why.as_ref()))
}

pub struct Checker;
impl crate::selftest::Selftest for Checker {
    fn checker(&self) -> &'static str {
        "handoff"
    }
    fn run(
        &self,
        _ctx: &mut crate::core::context::Context,
        file: &std::path::Path,
    ) -> Result<crate::selftest::Verdict> {
        use super::types::{Document, WorkItem};
        use crate::core::mode::{Mode, Provider};
        use crate::selftest::Verdict;
        let text = std::fs::read_to_string(file).map_err(super::packet::io)?;
        let state = Snapshot {
            run_id: "fixture".into(),
            worktree: "/fixture".into(),
            mode: Mode::default(),
            owner_session: "source".into(),
            fingerprint: "fixture".into(),
            documents: vec![Document {
                reference: "task:T1".into(),
                path: "plan.json:1".into(),
                text: "active task".into(),
            }],
            items: vec![WorkItem {
                id: "T1".into(),
                plan: "P1".into(),
                state: "active".into(),
                title: "work".into(),
                covers: vec!["R08".into()],
                files: vec![],
                deps: vec![],
                commit: String::new(),
                refs: vec!["task:T1".into()],
            }],
        };
        let history = History {
            provider: Provider::Claude,
            session: "source".into(),
            cwd: "/fixture".into(),
            path: "history.jsonl".into(),
            sha256: String::new(),
            records: vec![],
            warnings: vec![],
            omitted: 0,
        };
        Ok(if validate(&text, &state, &history).is_ok() { Verdict::Pass } else { Verdict::Reject })
    }
}
