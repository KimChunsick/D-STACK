// store/plan_ids.rs
// Plan and task ids, decimal inserts and the validation every plan verb runs before a write.

use crate::core::error::{Error, Result};
use crate::core::paths::valid_rel_path;
use crate::store::plan::PlanDoc;
use crate::store::plan_graph::{acyclic_plans, acyclic_tasks};

/// _next_int_id: one past the largest whole number an id of this prefix carries. The decimal part
/// is ignored, so P1.3 does not reserve P3.
pub fn next_int_id(doc: &PlanDoc, prefix: &str) -> String {
    let ids: Vec<&String> = if prefix == "M" {
        doc.milestones.iter().map(|m| &m.id).collect()
    } else {
        doc.plans.iter().map(|p| &p.id).collect()
    };
    let max = ids
        .iter()
        .filter_map(|id| {
            id.trim_start_matches(['M', 'P'])
                .split('.')
                .next()
                .and_then(|n| n.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    format!("{prefix}{}", max + 1)
}

/// _next_decimal_id: inserting in the middle takes a decimal, so the ids after it never shift.
pub fn next_decimal_id(parent: &str, taken: &[String]) -> Result<String> {
    for k in 1..=99 {
        let id = format!("{parent}.{k}");
        if !taken.iter().any(|t| *t == id) {
            return Ok(id);
        }
    }
    Err(Error::cannot_decide(format!(
        "no free decimal id under {parent} (99 taken)"
    )))
}

/// sed's `[[:space:]]` in the C locale — the set _csv_list trims off both ends of an item.
const SPACE: [char; 6] = [' ', '\t', '\n', '\u{b}', '\u{c}', '\r'];

/// _csv_list: `tr ',' '\n'` first, then one item per line — so a comma and a newline are the
/// same separator, an item carrying either would be two — each item trimmed, empty ones dropped.
pub fn csv_list(csv: &str) -> Vec<String> {
    csv.split([',', '\n'])
        .map(|item| item.trim_matches(SPACE))
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect()
}

/// _path_within: `file` is covered by `plan_file` when they are equal or `plan_file` is a
/// directory prefix of it. Directional, unlike paths_overlap: task files ⊆ plan files.
pub fn path_within(file: &str, plan_file: &str) -> bool {
    file == plan_file || format!("{file}/").starts_with(&format!("{plan_file}/"))
}

/// _validate_files: fails with the offending path and the rule it broke.
pub fn validate_files(csv: &str) -> Result<Vec<String>> {
    let files = csv_list(csv);
    for file in &files {
        if !valid_rel_path(file) {
            return Err(Error::failed(format!(
                "invalid file path: '{file}' (must be repo-relative: no leading /, no .. segment, no * ? [ , not empty)"
            )));
        }
    }
    if files.is_empty() {
        return Err(Error::failed(
            "--files must list at least one repo-relative path (comma separated)",
        ));
    }
    Ok(files)
}

/// _validate_deps: `known` carries the element being added too, so that a self-dependency reaches
/// the cycle check; `shown` is what the message offers the user, and an empty one reads "none".
pub fn validate_deps(
    csv: &str,
    known: &[String],
    noun: &str,
    shown: &[String],
) -> Result<Vec<String>> {
    let deps = csv_list(csv);
    for dep in &deps {
        if !known.iter().any(|k| k == dep) {
            let list = shown.join(" ");
            let list = if list.is_empty() { "none" } else { &list };
            return Err(Error::failed(format!(
                "{noun} dependency does not exist: {dep} (known: {list})"
            )));
        }
    }
    Ok(deps)
}

pub fn assert_acyclic_plans(doc: &PlanDoc) -> Result<()> {
    if acyclic_plans(doc) {
        return Ok(());
    }
    Err(cycle("plans"))
}

pub fn assert_acyclic_tasks(doc: &PlanDoc) -> Result<()> {
    if acyclic_tasks(doc) {
        return Ok(());
    }
    Err(cycle("tasks"))
}

fn cycle(nouns: &str) -> Error {
    Error::failed(format!(
        "the resulting {nouns} dependency graph has a cycle (a plan cannot depend on itself or on anything that waits for it)"
    ))
}
