// verbs/review/findings.rs
// Open ledger items are selected using the plan graph without changing their quoted text.

use std::path::Path;

use regex::Regex;

use crate::core::error::{Error, Result};
use crate::store::plan::PlanDoc;
use crate::store::plan_graph::{milestone_covers, plan_covers};

/// Findings use `[P1, ...]`, `[P1 / round ..., ...]` and prose R/Plan/Milestone references.
/// Any known selected owner retains the whole item. Explicit requirements narrow inherited
/// coverage; without them, a prior Plan's coverage can carry its finding to a follow-up.
/// Unknown or absent ownership is an error, never permission to drop an open finding.
pub(super) fn selected<'a>(
    text: &'a str,
    path: &Path,
    doc: &PlanDoc,
    mid: &str,
    covers: &[String],
) -> Result<Vec<&'a str>> {
    let markers = Regex::new(r"\b[PMR][0-9]+(?:\.[0-9]+)*\b").expect("scope markers");
    let resolved =
        Regex::new(r"\bresolved\s*:|(?:^|[;.—]\s*)resolved\s+in\b").expect("resolution annotation");
    let known: Vec<&String> = doc
        .plans
        .iter()
        .flat_map(|p| &p.tasks)
        .flat_map(|t| &t.covers)
        .collect();
    let mut selected = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if !is_open(line, &resolved) {
            continue;
        }
        let ids: Vec<&str> = markers.find_iter(line).map(|m| m.as_str()).collect();
        let cannot_scope = |why: &str| {
            Error::failed(format!(
                "{}:{}: cannot scope open finding for {mid}: {why}",
                path.display(),
                index + 1
            ))
        };
        if ids.is_empty() {
            return Err(cannot_scope(
                "no known Plan, Milestone or requirement reference",
            ));
        }
        let explicit_requirements = ids.iter().any(|id| id.starts_with('R'));
        let mut relevant = false;
        for id in ids {
            relevant |= match id.as_bytes()[0] {
                b'P' => {
                    let plan = doc
                        .plan(id)
                        .ok_or_else(|| cannot_scope(&format!("unknown Plan {id}")))?;
                    plan.milestone == mid
                        || (!explicit_requirements
                            && plan_covers(doc, id).iter().any(|r| covers.contains(r)))
                }
                b'M' => {
                    if !doc.milestones.iter().any(|m| m.id == id) {
                        return Err(cannot_scope(&format!("unknown Milestone {id}")));
                    }
                    id == mid
                        || (!explicit_requirements
                            && milestone_covers(doc, id).iter().any(|r| covers.contains(r)))
                }
                _ => {
                    if !known.iter().any(|r| r.as_str() == id) {
                        return Err(cannot_scope(&format!("no Plan covers requirement {id}")));
                    }
                    covers.iter().any(|r| r == id)
                }
            };
        }
        if relevant {
            // Cross-scope R references stay verbatim: the existing bundle checker refuses
            // unrepresentable mixed coverage instead of accepting edited IDs or quotes.
            selected.push(line);
        }
    }
    Ok(selected)
}

/// Only affirmative resolution annotations close an item. "unresolved" and "to be resolved
/// by P3" describe pending work and must remain visible to the ledger pass.
fn is_open(line: &str, resolved: &Regex) -> bool {
    let item = line.trim_start_matches([' ', '\t']);
    (item.starts_with("- ") || item.starts_with("* ")) && !resolved.is_match(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_resolution_annotations_do_not_hide_pending_work() {
        let resolved = Regex::new(r"\bresolved\s*:|(?:^|[;.—]\s*)resolved\s+in\b").unwrap();
        for line in [
            "- [P1] remains unresolved",
            "  * [P1] to be resolved by P3",
            "- [P1] is not resolved",
            "- [P1] resolved input parsing but not the issue",
        ] {
            assert!(is_open(line, &resolved), "{line}");
        }
        for line in [
            "- [P1] was fixed — resolved in P1",
            "- [P1] fixed; resolved: P1",
            "a paragraph",
            "-no space after the dash",
        ] {
            assert!(!is_open(line, &resolved), "{line}");
        }
    }
}
