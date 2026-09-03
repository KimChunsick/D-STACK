// verbs/review/bundle.rs
// dstack review: the frozen request, the plan, the allowed diff and the reviewer's contract.

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::meta::meta_get;
use crate::core::target::{resolve_target, TargetKind};
use crate::store::plan::{self, PlanDoc};
use crate::store::plan_graph::{milestone_covers, plan_covers};
use crate::store::review_index::{next_seq, sealed_counts};
use crate::store::rows;

use super::emit_diff::{self, Counts};
use super::{check_bundle, lines, take, MAX_BUNDLE};

pub fn review(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut scope, mut pid) = (String::new(), String::new());
    let (mut mid, mut out) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        if let Some((value, eaten)) = take(&rest, i, "scope")? {
            scope = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "plan")? {
            pid = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "milestone")? {
            mid = value;
            i += eaten;
        } else if let Some((value, eaten)) = take(&rest, i, "out")? {
            out = value;
            i += eaten;
        } else {
            fail!("unexpected argument: {} (dstack review --scope plan --plan P1)", rest[i]);
        }
    }
    match scope.as_str() {
        "plan" if pid.is_empty() => fail!("--scope plan needs --plan P<n>"),
        "milestone" if mid.is_empty() => fail!("--scope milestone needs --milestone M<n>"),
        "plan" | "milestone" => {}
        _ => fail!("--scope must be plan or milestone (got '{scope}')"),
    }
    if target.kind == TargetKind::Quick {
        fail!("quick tasks have no plans — nothing to review as a bundle (R99: review: off is the only place a skipped review is real)");
    }

    let dir = target.dir.clone();
    let request = dir.join("request.md");
    if !request.is_file() {
        fail!("no request.md in {} — a bundle without the request is what R69 forbids", dir.display());
    }
    if !dir.join("request.approved").is_file() {
        fail!("request is not approved (dstack request approve) — the frozen section needs a frozen request");
    }
    if !plan::exists(&dir) {
        fail!("no plan.json in {} (dstack plan add …)", dir.display());
    }
    let doc = plan::load(&dir)?;

    let (id, body, counts) = match scope.as_str() {
        "plan" => {
            if doc.plan(&pid).is_none() {
                fail!("plan not found: {pid}");
            }
            let covers = plan_covers(&doc, &pid);
            let wt = worktree(&doc, &dir, &pid, &roots.wt_root)?;
            let (body, counts) = plan_bundle(&dir, &request, &doc, &pid, &covers, &wt)?;
            (pid.clone(), body, counts)
        }
        _ => {
            if !doc.milestones.iter().any(|m| m.id == mid) {
                fail!("milestone not found: {mid}");
            }
            let covers = milestone_covers(&doc, &mid);
            let body = milestone_bundle(&dir, &request, &doc, &mid, &covers)?;
            (mid.clone(), body, Counts::default())
        }
    };

    let total = body.len();
    if total > MAX_BUNDLE {
        say!(ctx, "bundle would be {total} bytes (ceiling {MAX_BUNDLE})");
        fail!("bundle exceeds 512KB: split the plan");
    }

    let review_dir = dir.join("review");
    let _ = std::fs::create_dir_all(&review_dir);
    let out = match out.is_empty() {
        true => review_dir.join(format!(
            "bundle-{scope}-{id}-{}.txt",
            next_seq(&review_dir, &format!("bundle-{scope}-{id}-"))
        )),
        false => {
            let path = absolute(&out)?;
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            path
        }
    };
    std::fs::write(&out, &body)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", out.display())))?;

    // R69: only a bundle that survives its own checker leaves this command.
    if !check_bundle::check_file(ctx, &out, &dir)? {
        let _ = std::fs::remove_file(&out);
        fail!("bundle deleted (it would have hidden a requirement): {}", out.display());
    }
    say!(ctx, "  bundle: {}", out.display());
    say!(ctx, "  bytes {total} of {MAX_BUNDLE}; diff files {}, oversize skipped {}",
         counts.files, counts.skipped);
    Ok(())
}

/// Where the diff is read: the plan's own worktree, else the run's, else this checkout.
fn worktree(doc: &PlanDoc, dir: &Path, pid: &str, wt_root: &Path) -> Result<String> {
    let declared = doc.plan(pid).map(|p| p.worktree.clone()).unwrap_or_default();
    if !declared.is_empty() {
        return Ok(declared);
    }
    Ok(match meta_get(dir, "worktree")?.filter(|w| !w.is_empty()) {
        Some(worktree) => worktree,
        None => wt_root.to_string_lossy().into_owned(),
    })
}

fn plan_bundle(
    dir: &Path,
    request: &Path,
    doc: &PlanDoc,
    pid: &str,
    covers: &[String],
    wt: &str,
) -> Result<(Vec<u8>, Counts)> {
    let plan = doc.plan(pid).expect("the plan was found above");
    let base = meta_get(dir, "base_head")?.unwrap_or_default();
    let mut out = Vec::new();
    push(&mut out, "=== REQUEST (frozen) ===\n");
    emit_request_rows(&mut out, request, covers)?;
    push(&mut out, "\n=== PLAN ===\n");
    push(&mut out, &format!("plan: {pid}\n"));
    push(&mut out, &format!("slug: {}\n", plan.slug));
    push(&mut out, &format!("status: {}\n", plan.status));
    push(&mut out, &format!("files: {}\n", plan.files.join(", ")));
    push(&mut out, &format!("deps: {}\n", match plan.deps.is_empty() {
        true => "(none)".to_string(),
        false => plan.deps.join(", "),
    }));
    for task in &plan.tasks {
        push(&mut out, &format!(
            "{} {} covers: {} files: {}\n",
            task.id, task.slug, task.covers.join(", "), task.files.join(", ")
        ));
    }
    push(&mut out, "\n=== DIFF (allowed files only) ===\n");
    push(&mut out, &format!("worktree: {wt}\nbase: {}\n", match base.is_empty() {
        true => "none",
        false => &base,
    }));
    // The shell expands `$files` unquoted, so a declared path carrying a space arrives as two.
    let files: Vec<String> = plan
        .files
        .join("\n")
        .split_whitespace()
        .map(String::from)
        .collect();
    let counts = match files.is_empty() {
        true => {
            push(&mut out, "(the plan declares no files)\n");
            Counts::default()
        }
        false => emit_diff::emit(&mut out, Path::new(wt), &base, &files),
    };
    push(&mut out, "\n=== CONTRACT ===\n");
    push(&mut out, "Your first output is a per-R verdict table with one row per R id in the REQUEST section, in the form `| R | verdict (covered|partial|absent) | evidence in the diff |`; judge only against the frozen rows above, and cite the file and hunk that proves each verdict.\n");
    push(&mut out, "Then list findings by axis, each as `[axis] SEV: finding`, with the axes goal achievement / security / UI·UX&DX / performance / architecture & code quality; a finding must name the file it lives in.\n");
    push(&mut out, "Your last line is `VERDICT: approve|reject`; any absent verdict in the table means reject.\n");
    Ok((out, counts))
}

fn milestone_bundle(
    dir: &Path,
    request: &Path,
    doc: &PlanDoc,
    mid: &str,
    covers: &[String],
) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    push(&mut out, "=== REQUEST (frozen) ===\n");
    emit_request_rows(&mut out, request, covers)?;
    push(&mut out, "\n=== FINDINGS (open) ===\n");
    push(&mut out, &format!("milestone: {mid}\n"));
    let findings = dir.join("findings.md");
    if findings.is_file() {
        let text = read(&findings)?;
        let mut open = 0;
        for line in lines(&text).iter().filter(|line| is_open(line)) {
            push(&mut out, &format!("{line}\n"));
            open += 1;
        }
        if open == 0 {
            push(&mut out, "(findings.md has no open items)\n");
        }
    } else {
        push(&mut out, "(no findings.md — nothing carried over)\n");
    }
    push(&mut out, "\n=== INTEGRATION ===\n");
    for plan in doc.plans.iter().filter(|p| p.milestone == mid) {
        let (rounds, absent, partial, covered) = sealed_counts(dir, &plan.id)?;
        push(&mut out, &format!(
            "{} {} status: {} sealed rounds: {rounds} (covered {covered}, partial {partial}, absent {absent}) covers: {}\n",
            plan.id, plan.slug, plan.status, plan_covers(doc, &plan.id).join(" ")
        ));
    }
    push(&mut out, "\n=== CONTRACT ===\n");
    push(&mut out, &format!("This is a ledger pass, not a fresh review: re-check only the open findings listed above and the integration behaviour between the plans of {mid}.\n"));
    push(&mut out, "Opening new scope-wide findings is forbidden — if you see one, say so in one line under `out of scope` and do not raise it as a finding.\n");
    push(&mut out, "Output the same per-R verdict table (`| R | verdict (covered|partial|absent) | evidence |`) for the R ids listed in the REQUEST section, then `VERDICT: approve|reject` as the last line.\n");
    Ok(out)
}

/// A markdown list item is open until the line itself says "resolved": the ledger records the
/// resolution on the item, so nothing else has to be re-read to know what is still owed (R70).
fn is_open(line: &str) -> bool {
    let item = line.trim_start_matches([' ', '\t']);
    (item.starts_with("- ") || item.starts_with("* ")) && !line.contains("resolved")
}

/// Verbatim R rows, in request order, for the ids the plan covers. Verbatim matters: the
/// reviewer must judge against the approved text, not against a re-worded copy (R69 "frozen").
fn emit_request_rows(out: &mut Vec<u8>, request: &Path, ids: &[String]) -> Result<()> {
    let mut wanted = String::from(" ");
    for id in ids {
        wanted.push_str(id);
        wanted.push(' ');
    }
    wanted.push(' ');
    let text = read(request)?;
    for line in lines(&text) {
        if let Some(row) = rows::parse_line(1, line) {
            if wanted.contains(&format!(" {} ", row.id)) {
                push(out, &format!("{line}\n"));
            }
        }
    }
    Ok(())
}

/// A relative --out is resolved against the physical working directory, as `$(pwd -P)/` does.
fn absolute(out: &str) -> Result<PathBuf> {
    let path = PathBuf::from(out);
    if path.is_absolute() {
        return Ok(path);
    }
    let cwd = std::env::current_dir()
        .map_err(|e| Error::cannot_decide(format!("cannot read the working directory: {e}")))?;
    Ok(cwd.join(out))
}

/// A file the awk readers walked line by line. Everything they match is ASCII, so replacing
/// bytes that are not UTF-8 changes no decision this module makes. A file that is not there is
/// empty (the caller asked whether it exists); one that cannot be read is a cannot-decide (D-12),
/// because a bundle built from a request nobody could read is a review of nothing.
fn read(path: &Path) -> Result<String> {
    match std::fs::read(path) {
        Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).into_owned()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(Error::cannot_decide(format!(
            "cannot read {}: {e}",
            path.display()
        ))),
    }
}

fn push(out: &mut Vec<u8>, text: &str) {
    out.extend_from_slice(text.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_an_item_is_open_until_the_line_says_resolved() {
        assert!(is_open("- the cap is silent about the plan"));
        assert!(is_open("  * an indented item"));
        assert!(!is_open("- the cap was fixed — resolved in P1"));
        assert!(!is_open("a paragraph that is not a list item"));
        assert!(!is_open("-no space after the dash"));
    }
}
