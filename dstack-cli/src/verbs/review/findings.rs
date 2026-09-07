// verbs/review/findings.rs
// Open ledger items are selected using the plan graph without changing their quoted text.

use std::path::Path;

use regex::Regex;

use crate::core::error::{Error, Result};
use crate::store::plan::PlanDoc;
use crate::store::plan_graph::{milestone_covers, plan_covers};

// Recognize the ledger's affirmative forms only. Unknown explanations remain open rather
// than interpreting arbitrary prose, booleans, or conditional promises as completed work.
const RESOLUTION: &str = concat!(
    r"^(?:resolved:\s*(?:commit [0-9a-fA-F]{7,40}\b.*|",
    r"(?:fixed|verified)(?: in P[0-9]+(?:\.[0-9]+)*)?\.?|",
    r"P[0-9]+(?:\.[0-9]+)*\.?)|resolved in P[0-9]+(?:\.[0-9]+)*\.?)$"
);

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
    let resolved = Regex::new(RESOLUTION).expect("resolution annotation");
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

/// An annotation starts after a sentence/clause separator outside quoted diagnostic text.
/// Requiring the entire remaining annotation to match keeps negation, false values and
/// conditional/unknown explanations open. Apostrophes within words are not quote boundaries.
fn is_open(line: &str, resolved: &Regex) -> bool {
    let item = line.trim_start_matches([' ', '\t']);
    if !(item.starts_with("- ") || item.starts_with("* ")) {
        return false;
    }
    let mut quote = None;
    let mut escaped = false;
    let mut code_width = 0;
    let mut skip_until = 0;
    for (at, ch) in line.char_indices() {
        if at < skip_until {
            continue;
        }
        if ch == '`' && (quote.is_none() || quote == Some('`')) {
            let width = line[at..].bytes().take_while(|b| *b == b'`').count();
            skip_until = at + width;
            if quote.is_none() {
                quote = Some('`');
                code_width = width;
            } else if code_width == width {
                quote = None;
            }
            continue;
        }
        if quote == Some('`') {
            continue;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = quote.is_some();
            continue;
        }
        let after = &line[at + ch.len_utf8()..];
        if matches!(ch, '\'' | '’')
            && line[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_alphanumeric)
            && after.chars().next().is_some_and(char::is_alphanumeric)
        {
            continue;
        }
        if let Some(end) = quote {
            if ch == end {
                quote = None;
            }
            continue;
        }
        quote = match ch {
            '"' | '\'' => Some(ch),
            '“' => Some('”'),
            '‘' => Some('’'),
            _ => None,
        };
        if matches!(ch, ';' | '.' | '—')
            && after.starts_with(char::is_whitespace)
            && resolved.is_match(after.trim())
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_resolution_annotations_do_not_hide_pending_work() {
        let resolved = Regex::new(RESOLUTION).unwrap();
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
