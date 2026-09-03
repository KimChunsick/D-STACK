// verbs/plan/lifecycle.rs
// dstack plan render, plan start and plan done: the status a worker's isolation depends on.

use std::path::PathBuf;

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::utc_now;
use crate::core::roots::git_out;
use crate::store::plan_graph::{counts_line, render_table};

plan_verb!(PlanRender, "plan render", render);
plan_verb!(PlanStart, "plan start", start);
plan_verb!(PlanDone, "plan done", done);

fn render(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let doc = target.load()?;
    ctx.out.raw(&render_table(&doc));
    target.regen(&doc)?;
    say!(ctx, "{}", counts_line(&doc));
    say!(
        ctx,
        "regenerated: {}/ROADMAP.md, {}/STATE.md",
        target.dir.display(),
        target.dir.display()
    );
    Ok(())
}

fn start(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let (mut p, mut worktree) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(arg, next, "worktree")? {
            worktree = value;
            i += eaten;
        } else if is_option(arg) {
            fail!("unknown option: {arg} (usage: dstack plan start P<n> [--worktree <path>])")
        } else if p.is_empty() {
            p = arg.to_string();
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if p.is_empty() {
        fail!("usage: dstack plan start P<n> [--worktree <path>]")
    }
    let mut doc = target.load()?;
    if !doc.plan_ids().contains(&p) {
        fail!("plan not found: {p} (known: {})", doc.plan_ids().join(" "))
    }
    let status = doc.field(&p, "status");
    if status != "pending" && status != "ready" {
        fail!("refused: {p} is {status} — only a pending or ready plan can start")
    }
    let done: Vec<&String> = doc
        .plans
        .iter()
        .filter(|plan| plan.status == "done")
        .map(|plan| &plan.id)
        .collect();
    let unmet: Vec<String> = doc
        .plan(&p)
        .expect("the plan was found above")
        .deps
        .iter()
        .filter(|dep| !done.contains(dep))
        .cloned()
        .collect();
    if !unmet.is_empty() {
        fail!(
            "refused: {p} waits on unfinished dependencies: {}",
            unmet.join(", ")
        )
    }

    let mut created = String::new();
    if !worktree.is_empty() {
        let path = match worktree.starts_with('/') {
            true => PathBuf::from(&worktree),
            false => std::env::current_dir()
                .map_err(|_| Error::Exit(1))?
                .join(&worktree),
        };
        if !path.exists() {
            created = make_worktree(&target, &doc, &p, &path)?;
        }
        // `cd "$wt" && pwd -P`: a path that cannot be entered ends the run the way `set -e` ends
        // it, with cd's status and nothing of the store written. A path that exists but is not a
        // directory — a regular file, a symlink to one — is exactly that case, and canonicalize
        // would happily resolve it, so the directory is asked for before the plan is touched.
        // (bash prints its own "cd: ...: Not a directory" diagnostic there; D-11 leaves it out.)
        if !path.is_dir() {
            return Err(Error::Exit(1));
        }
        worktree = std::fs::canonicalize(&path)
            .map_err(|_| Error::Exit(1))?
            .to_string_lossy()
            .into_owned();
    }

    let now = utc_now();
    let plan = doc.plan_mut(&p).expect("the plan was found above");
    plan.status = "in-progress".to_string();
    plan.worktree = worktree.clone();
    plan.started_at = now.clone();

    let doc = target.write(doc)?;
    say!(ctx, "plan {p}: {status} → in-progress at {now}");
    if !worktree.is_empty() {
        say!(ctx, "  worktree: {worktree}{created}");
    }
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}

/// R36: the worker's isolation is a git worktree that dstack makes, not Claude Code's own
/// worktree mode — that mode blocks writes back to the main checkout, where .dstack lives.
fn make_worktree(
    target: &super::Target,
    doc: &crate::store::plan::PlanDoc,
    p: &str,
    path: &std::path::Path,
) -> Result<String> {
    let src = target.worktree()?;
    let branch = format!("plan/{p}-{}", doc.field(p, "slug"));
    if git_out(
        Some(&src),
        &[
            "show-ref",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ],
    )
    .is_some()
    {
        fail!("branch already exists: {branch} (delete it or pass an existing --worktree path)")
    }
    let added = git_out(
        Some(&src),
        &[
            "worktree",
            "add",
            "-b",
            &branch,
            &path.to_string_lossy(),
            "HEAD",
        ],
    );
    if added.is_none() {
        fail!(
            "git worktree add failed for {} (branch {branch})",
            path.display()
        )
    }
    // The worker will run dstack inside this checkout; give it the local dir and the self-ignore
    // so nothing under .dstack can be staged from there (D-01).
    let local = path.join(".dstack/local");
    std::fs::create_dir_all(&local).map_err(|_| Error::Exit(1))?;
    std::fs::create_dir_all(path.join(".dstack/quick")).map_err(|_| Error::Exit(1))?;
    let _ = std::fs::set_permissions(&local, std::os::unix::fs::PermissionsExt::from_mode(0o700));
    std::fs::write(path.join(".dstack/.gitignore"), "*\n").map_err(|_| Error::Exit(1))?;
    Ok(format!(" (created on branch {branch})"))
}

fn done(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let p = rest.first().cloned().unwrap_or_default();
    if p.is_empty() {
        fail!("usage: dstack plan done P<n>")
    }
    let mut doc = target.load()?;
    if !doc.plan_ids().contains(&p) {
        fail!("plan not found: {p} (known: {})", doc.plan_ids().join(" "))
    }
    let status = doc.field(&p, "status");
    if status != "in-progress" {
        fail!("refused: {p} is {status} — only an in-progress plan can be marked done (dstack plan start {p})")
    }
    let now = utc_now();
    let plan = doc.plan_mut(&p).expect("the plan was found above");
    plan.status = "done".to_string();
    plan.done_at = now.clone();

    // The refresh inside the write promotes whatever this unblocked from pending to ready.
    let doc = target.write(doc)?;
    let ready: Vec<String> = doc
        .plans
        .iter()
        .filter(|plan| plan.status == "ready")
        .map(|plan| plan.id.clone())
        .collect();
    say!(ctx, "plan {p}: in-progress → done at {now}");
    say!(ctx, "  unblocked → ready: {}", ready.join(", "));
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}
