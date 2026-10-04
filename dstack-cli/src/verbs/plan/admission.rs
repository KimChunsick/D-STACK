// verbs/plan/admission.rs
// Shared admission check for starting and resuming a Plan. Callers hold the store lock.

use crate::core::error::{Error, Result};
use crate::core::paths::{paths_overlap, shell_int};
use crate::core::tools::policy_get;
use crate::store::plan::PlanDoc;

use super::confirm::gate_reason;
use super::Target;

pub(super) fn check(target: &Target, doc: &PlanDoc, id: &str, e2e: &str) -> Result<()> {
    let plan = doc.plan(id).expect("the caller checked the plan id");
    let done: Vec<&str> = doc
        .plans
        .iter()
        .filter(|p| p.status == "done")
        .map(|p| p.id.as_str())
        .collect();
    let unmet: Vec<&str> = plan
        .deps
        .iter()
        .map(String::as_str)
        .filter(|dep| !done.contains(dep))
        .collect();
    if !unmet.is_empty() {
        return Err(Error::failed(format!(
            "refused: {id} waits on unfinished dependencies: {}",
            unmet.join(", ")
        )));
    }
    if let Some(reason) = gate_reason(doc, e2e, id) {
        return Err(Error::failed(format!("refused: {reason}")));
    }
    let cap = policy_get(&target.roots.store, "max_concurrent")
        .filter(|v| !v.is_empty() && v.chars().all(|c| c.is_ascii_digit()))
        .map(|v| shell_int(&v))
        .unwrap_or(5);
    let active: Vec<_> = doc
        .plans
        .iter()
        .filter(|p| p.status == "in-progress")
        .collect();
    if active.len() as i64 >= cap {
        return Err(Error::failed(format!(
            "refused: {id} has no free worker slot (cap {cap})"
        )));
    }
    for other in active {
        if let Some((a, b)) = plan.files.iter().find_map(|a| {
            other
                .files
                .iter()
                .find(|b| paths_overlap(a, b))
                .map(|b| (a, b))
        }) {
            return Err(Error::failed(format!(
                "refused: {id} overlaps in-progress {} on {a} and {b}",
                other.id
            )));
        }
    }
    Ok(())
}
