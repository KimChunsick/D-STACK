// verbs/plan/suspension.rs
// A suspended Plan keeps its worktree and history. The sidecar records why and which checkout
// the main session acknowledged as stopped; the Plan status remains the scheduling authority.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, utc_now};
use crate::core::roots::git_out;
use crate::store::plan_graph::counts_line;

use super::confirm::gate_e2e;
use super::Target;

#[derive(Serialize, Deserialize)]
struct Suspension {
    reason: String,
    at: String,
    worktree: String,
    branch: String,
    common_dir: String,
}

fn sidecar(target: &Target, id: &str) -> Result<PathBuf> {
    if !id.starts_with('P') || !id[1..].chars().all(|c| c.is_ascii_digit() || c == '.') {
        return Err(Error::cannot_decide(format!(
            "invalid Plan id for suspension: {id}"
        )));
    }
    Ok(target.dir.join("suspensions").join(format!("{id}.json")))
}

fn read_record(target: &Target, id: &str) -> Result<Suspension> {
    let file = sidecar(target, id)?;
    let record: Suspension = serde_json::from_str(&std::fs::read_to_string(&file)
        .map_err(|e| Error::cannot_decide(format!("cannot read suspension record {}: {e}", file.display())))?)
        .map_err(|e| Error::cannot_decide(format!("invalid suspension record {}: {e}", file.display())))?;
    if record.reason.trim().is_empty() {
        return Err(Error::cannot_decide(format!("suspension record has no reason: {}", file.display())));
    }
    Ok(record)
}

pub(super) fn reason(target: &Target, id: &str) -> Result<String> {
    Ok(read_record(target, id)?.reason)
}

fn identity(target: &Target, path: &Path) -> Result<(String, String, String)> {
    let wt = std::fs::canonicalize(path).map_err(|e| {
        Error::cannot_decide(format!(
            "cannot inspect Plan worktree {}: {e}",
            path.display()
        ))
    })?;
    if !wt.is_dir() {
        return Err(Error::failed(format!(
            "Plan worktree is not a directory: {}",
            wt.display()
        )));
    }
    let top = git_out(Some(&wt), &["rev-parse", "--show-toplevel"]).ok_or_else(|| {
        Error::failed(format!(
            "Plan worktree is not a Git checkout: {}",
            wt.display()
        ))
    })?;
    if Path::new(&top) != wt {
        return Err(Error::failed(format!(
            "Plan worktree is not the checkout root: {}",
            wt.display()
        )));
    }
    let branch = git_out(Some(&wt), &["symbolic-ref", "--quiet", "--short", "HEAD"])
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::failed(format!("Plan worktree has no branch: {}", wt.display())))?;
    let common = git_out(Some(&wt), &["rev-parse", "--git-common-dir"]).ok_or_else(|| {
        Error::failed(format!(
            "Plan worktree has no Git common directory: {}",
            wt.display()
        ))
    })?;
    let common = PathBuf::from(common);
    let common = if common.is_absolute() {
        common
    } else {
        wt.join(common)
    };
    let common = std::fs::canonicalize(&common).map_err(|e| {
        Error::cannot_decide(format!("cannot inspect Plan Git common directory: {e}"))
    })?;
    let expected = std::fs::canonicalize(target.roots.main_root.join(".git")).map_err(|e| {
        Error::cannot_decide(format!("cannot inspect project Git common directory: {e}"))
    })?;
    if common != expected {
        return Err(Error::failed(format!(
            "Plan worktree belongs to another repository: {}",
            wt.display()
        )));
    }
    Ok((
        wt.to_string_lossy().into_owned(),
        branch,
        common.to_string_lossy().into_owned(),
    ))
}

fn plan_worktree(target: &Target, recorded: &str) -> Result<PathBuf> {
    if recorded.is_empty() {
        target.worktree()
    } else {
        Ok(PathBuf::from(recorded))
    }
}

pub(super) fn suspend(
    ctx: &mut Context,
    target: &Target,
    id: &str,
    reason: Option<String>,
    worker_stopped: bool,
) -> Result<()> {
    let reason = super::free_text("reason", reason)?
        .ok_or_else(|| Error::failed("--reason <text> is required to suspend a Plan"))?;
    if !worker_stopped {
        return Err(Error::failed(
            "--worker-stopped is required: confirm the native worker has stopped",
        ));
    }
    let locked = target.lock()?;
    let mut doc = locked.load()?;
    let plan = doc.plan(id).ok_or_else(|| {
        Error::failed(format!(
            "plan not found: {id} (known: {})",
            doc.plan_ids().join(" ")
        ))
    })?;
    if plan.status != "in-progress" {
        return Err(Error::failed(format!(
            "refused: {id} is {} — only an in-progress Plan can be suspended",
            plan.status
        )));
    }
    let (worktree, branch, common_dir) = identity(target, &plan_worktree(target, &plan.worktree)?)?;
    let at = utc_now();
    let record = Suspension {
        reason: reason.clone(),
        at: at.clone(),
        worktree,
        branch,
        common_dir,
    };
    let file = sidecar(target, id)?;
    std::fs::create_dir_all(file.parent().expect("suspension directory"))
        .map_err(|e| Error::cannot_decide(format!("cannot create suspension directory: {e}")))?;
    let mut bytes = serde_json::to_vec_pretty(&record).expect("suspension serialises");
    bytes.push(b'\n');
    atomic_write(&file, &bytes).map_err(|e| {
        Error::cannot_decide(format!(
            "cannot write suspension record {}: {e}",
            file.display()
        ))
    })?;
    doc.plan_mut(id).expect("checked above").status = "suspended".into();
    let doc = locked.write(doc)?;
    say!(ctx, "plan {id}: in-progress → suspended at {at}");
    say!(ctx, "  reason: {reason}");
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}

pub(super) fn resume(ctx: &mut Context, target: &Target, id: &str, confirm: bool) -> Result<()> {
    if !confirm {
        return Err(Error::failed("--confirm is required to resume a suspended Plan after checking its worker remains stopped"));
    }
    let locked = target.lock()?;
    let mut doc = locked.load()?;
    let plan = doc.plan(id).ok_or_else(|| {
        Error::failed(format!(
            "plan not found: {id} (known: {})",
            doc.plan_ids().join(" ")
        ))
    })?;
    if plan.status != "suspended" {
        return Err(Error::failed(format!(
            "refused: {id} is {} — only a suspended Plan can resume",
            plan.status
        )));
    }
    let record = read_record(target, id)?;
    let (worktree, branch, common_dir) = identity(target, &plan_worktree(target, &plan.worktree)?)?;
    if worktree != record.worktree || branch != record.branch || common_dir != record.common_dir {
        return Err(Error::failed(format!(
            "refused: {id} worktree identity changed since suspension"
        )));
    }
    let e2e = gate_e2e(&target.dir)?;
    super::admission::check(target, &doc, id, &e2e)?;
    let at = utc_now();
    doc.plan_mut(id).expect("checked above").status = "in-progress".into();
    let doc = locked.write(doc)?;
    say!(ctx, "plan {id}: suspended → in-progress at {at}");
    say!(ctx, "  worktree: {}", record.worktree);
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}
