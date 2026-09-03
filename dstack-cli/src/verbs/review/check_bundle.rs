// verbs/review/check_bundle.rs
// The R69 check: the REQUEST rows, the plan's covers and the R ids the body cites must agree.

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::target::resolve_target;
use crate::store::plan;
use crate::store::plan_graph::{milestone_covers, plan_covers};
use crate::store::rows;

use super::lines;

pub fn check(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let path = match rest.first() {
        Some(path) if !path.is_empty() => path.clone(),
        _ => fail!("usage: dstack check review-bundle <path> [--run <id>|--quick <slug>]"),
    };
    let bundle = Path::new(&path);
    if !bundle.is_file() {
        fail!("bundle not found: {path}");
    }
    if !check_file(ctx, bundle, &target.dir)? {
        fail!("review bundle rejected: {path}");
    }
    Ok(())
}

/// The check itself. Prints the three counts and answers false on any disagreement, so `review`
/// can reuse it before publishing a bundle and `check review-bundle` can expose it as a verb.
pub fn check_file(ctx: &mut Context, bundle: &Path, dir: &Path) -> Result<bool> {
    let text = read(bundle);
    let (scope, id) = scope_id(&text);
    say!(ctx, "review-bundle: {}", bundle.display());
    if scope.is_empty() {
        say!(ctx, "  no 'plan: P<n>' or 'milestone: M<n>' line — cannot tell what this bundle reviews");
        return Ok(false);
    }
    if !plan::exists(dir) {
        say!(ctx, "  target has no plan.json ({}) — cannot count covers", dir.display());
        return Ok(false);
    }
    let doc = plan::load(dir)?;
    let rows = request_ids(&text);
    let covers = match scope.as_str() {
        "plan" => plan_covers(&doc, &id),
        _ => milestone_covers(&doc, &id),
    };
    let cited = cited_ids(&text);
    say!(ctx, "  scope: {scope} {id}");
    say!(ctx, "  (a) R rows in REQUEST: {}", rows.len());
    say!(ctx, "  (b) covers of {id}:     {}", covers.len());
    say!(ctx, "  (c) R ids cited in body: {}", cited.len());
    let mut bad = 0;
    if rows.is_empty() {
        say!(ctx, "  REJECT: the REQUEST section is empty — a review without requirements is what v1 did");
        bad = 1;
    }
    if rows.len() != covers.len() || rows.len() != cited.len() {
        say!(ctx, "  REJECT: the three counts disagree (a={} b={} c={})",
             rows.len(), covers.len(), cited.len());
        // Name the actual difference: "they disagree" is not actionable at 2 a.m.
        let mut sorted = rows.clone();
        sorted.sort();
        sorted.dedup();
        let only_req = trailing_space(&only_in(&sorted, &cited));
        let only_body = trailing_space(&only_in(&cited, &sorted));
        if !only_req.is_empty() {
            say!(ctx, "    in REQUEST but never cited: {only_req}");
        }
        if !only_body.is_empty() {
            say!(ctx, "    cited but not in REQUEST:   {only_body}");
        }
        bad = 1;
    }
    say!(ctx, "  checked 3 counts, mismatched {bad}");
    Ok(bad == 0)
}

/// The bundle as the awk readers see it. Every marker they match is ASCII, so replacing bytes
/// that are not UTF-8 can neither invent nor hide an R id.
fn read(path: &Path) -> String {
    String::from_utf8_lossy(&std::fs::read(path).unwrap_or_default()).into_owned()
}

/// (a) The R rows inside the frozen REQUEST section. The section ends at the next "=== " banner,
/// so a row that drifted into the DIFF section is not counted as a requirement.
fn request_ids(text: &str) -> Vec<String> {
    let mut inside = false;
    let mut found = Vec::new();
    for line in lines(text) {
        if line == "=== REQUEST (frozen) ===" {
            inside = true;
            continue;
        }
        if inside && line.starts_with("=== ") {
            inside = false;
        }
        if inside {
            if let Some(row) = rows::parse_line(1, line) {
                found.push(row.id);
            }
        }
    }
    found
}

/// (c) The distinct R ids the body actually cites. "Body" is everything after the REQUEST
/// section: citing an id only in the frozen block proves nothing about the plan or the diff. The
/// DIFF section is excluded, because code comments in this repository cite the design's rule
/// numbers (R31, R52 …) and counting them would reject every bundle whose diff mentions a rule.
fn cited_ids(text: &str) -> Vec<String> {
    let (mut body, mut in_request, mut in_diff) = (false, false, false);
    let mut found = Vec::new();
    for line in lines(text) {
        if line == "=== REQUEST (frozen) ===" {
            in_request = true;
            continue;
        }
        if in_request && line.starts_with("=== ") {
            in_request = false;
            body = true;
        }
        if line == "=== DIFF (allowed files only) ===" {
            in_diff = true;
            continue;
        }
        if in_diff && line.starts_with("=== ") {
            in_diff = false;
        }
        if body && !in_diff {
            found.extend(r_ids(line));
        }
    }
    found.sort();
    found.dedup();
    found
}

/// `grep -o 'R[0-9][0-9]*'`: every non-overlapping R followed by digits, word boundaries and all.
fn r_ids(line: &str) -> Vec<String> {
    let bytes = line.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at] != b'R' {
            at += 1;
            continue;
        }
        let mut end = at + 1;
        while end < bytes.len() && bytes[end].is_ascii_digit() {
            end += 1;
        }
        if end == at + 1 {
            at += 1;
            continue;
        }
        found.push(line[at..end].to_string());
        at = end;
    }
    found
}

/// "plan P1" or "milestone M1" from the bundle's own header lines.
fn scope_id(text: &str) -> (String, String) {
    for line in lines(text) {
        let scope = if line.starts_with("plan: P") {
            "plan"
        } else if line.starts_with("milestone: M") {
            "milestone"
        } else {
            continue;
        };
        let id = line.split_whitespace().nth(1).unwrap_or_default();
        return (scope.to_string(), id.to_string());
    }
    (String::new(), String::new())
}

/// comm over two sorted lists: what the first holds and the second does not.
fn only_in(left: &[String], right: &[String]) -> Vec<String> {
    left.iter()
        .filter(|id| !right.contains(id))
        .cloned()
        .collect()
}

/// `tr '\n' ' '`: every id keeps the space that replaced its newline, the last one included.
fn trailing_space(ids: &[String]) -> String {
    ids.iter().map(|id| format!("{id} ")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUNDLE: &str = "=== REQUEST (frozen) ===\n\
        - [ ] **R01** one — accept: a\n\
        - [ ] **R02** two — accept: b\n\
        \n=== PLAN ===\n\
        plan: P1\n\
        T1 slug covers: R01, R02 files: a\n\
        \n=== DIFF (allowed files only) ===\n\
        worktree: /tmp/x\n\
        --- file: a\n\
        +# R31 — the rule this code cites\n\
        \n=== CONTRACT ===\n\
        Your last line is `VERDICT: approve|reject`.\n";

    #[test]
    fn r13_the_three_readers_agree_on_a_good_bundle() {
        assert_eq!(request_ids(BUNDLE), vec!["R01", "R02"]);
        assert_eq!(cited_ids(BUNDLE), vec!["R01", "R02"]);
        assert_eq!(
            scope_id(BUNDLE),
            ("plan".to_string(), "P1".to_string())
        );
    }

    /// D-08's fixture: rule numbers inside the diff are the reviewed source, not the reviewed
    /// requirements, so they never reach the (c) count.
    #[test]
    fn r13_ids_inside_the_diff_section_are_not_citations() {
        assert!(!cited_ids(BUNDLE).contains(&"R31".to_string()));
    }

    #[test]
    fn r13_r_ids_are_read_the_way_grep_o_reads_them() {
        assert_eq!(r_ids("R01 and RR02, R and R3"), vec!["R01", "R02", "R3"]);
        assert!(r_ids("| R | verdict |").is_empty());
    }

    #[test]
    fn r13_a_bundle_without_a_header_line_names_no_scope() {
        assert_eq!(scope_id("nothing here\n"), (String::new(), String::new()));
    }

    #[test]
    fn r13_the_difference_lines_keep_the_trailing_space() {
        let req = vec!["R01".to_string(), "R09".to_string()];
        let cited = vec!["R02".to_string(), "R09".to_string()];
        assert_eq!(trailing_space(&only_in(&req, &cited)), "R01 ");
        assert_eq!(trailing_space(&only_in(&cited, &req)), "R02 ");
        assert_eq!(trailing_space(&[]), "");
    }
}
