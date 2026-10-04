// verbs/plan/edit.rs
// dstack plan remove and plan edit: the two ways a plan changes before it starts (R67).

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::paths::valid_slug;
use crate::store::plan_graph::{counts_line, subtree_busy};
use crate::store::plan_ids::{
    assert_acyclic_plans, csv_list, path_within, validate_deps, validate_files,
};

const EDIT_USAGE: &str = "usage: dstack plan edit P<n> [--slug s] [--files a,b] [--deps P..] [--purpose <text>] [--e2e-focus <text>] | dstack plan edit P<n> --suspend --reason <text> --worker-stopped";

plan_verb!(PlanRemove, "plan remove", remove);
plan_verb!(PlanEdit, "plan edit", edit);

fn remove(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let p = rest.first().cloned().unwrap_or_default();
    if p.is_empty() {
        fail!("usage: dstack plan remove <P<n>>")
    }
    let mut doc = target.load()?;
    if !doc.plan_ids().contains(&p) {
        fail!("plan not found: {p} (known: {})", doc.plan_ids().join(" "))
    }
    let status = doc.field(&p, "status");
    if status == "in-progress" || status == "suspended" {
        fail!("refused: {p} is {status} — unfinished Plan history and worktree must be retained (R24)")
    }
    let users: Vec<String> = doc
        .plans
        .iter()
        .filter(|plan| plan.deps.contains(&p))
        .map(|plan| plan.id.clone())
        .collect();
    if !users.is_empty() {
        fail!(
            "refused: these plans depend on {p} ({}) — edit their --deps first (R67)",
            users.join(", ")
        )
    }
    doc.plans.retain(|plan| plan.id != p);
    // A confirmation records Plan ids and plan add/insert can mint this id again, so the
    // confirmation leaves with the Plan instead of passing to its successor (D-36).
    for milestone in doc.milestones.iter_mut() {
        milestone.confirmed.retain(|id| *id != p);
    }

    let doc = target.write(doc)?;
    say!(ctx, "removed plan {p} (was {status})");
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}

fn edit(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let (mut p, mut slug) = (String::new(), String::new());
    let (mut files, mut deps) = (String::new(), String::new());
    let (mut set_files, mut set_deps) = (false, false);
    let (mut suspend, mut worker_stopped) = (false, false);
    let mut reason = None;
    let (mut purpose, mut e2e_focus) = (None, None);
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(arg, next, "slug")? {
            slug = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "files")? {
            files = value;
            set_files = true;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "deps")? {
            deps = value;
            set_deps = true;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "purpose")? {
            purpose = Some(value);
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "e2e-focus")? {
            e2e_focus = Some(value);
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "reason")? {
            reason = Some(value);
            i += eaten;
        } else if arg == "--suspend" {
            suspend = true;
            i += 1;
        } else if arg == "--worker-stopped" {
            worker_stopped = true;
            i += 1;
        } else if is_option(arg) {
            fail!("unknown option: {arg} ({EDIT_USAGE})")
        } else if p.is_empty() {
            p = arg.to_string();
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if p.is_empty() {
        fail!("{EDIT_USAGE}")
    }
    if suspend {
        if !slug.is_empty() || set_files || set_deps || purpose.is_some() || e2e_focus.is_some() {
            fail!("--suspend cannot be combined with plan field edits")
        }
        return super::suspension::suspend(ctx, &target, &p, reason, worker_stopped);
    }
    if reason.is_some() || worker_stopped {
        fail!("--reason and --worker-stopped require --suspend")
    }
    let mut doc = target.load()?;
    if !doc.plan_ids().contains(&p) {
        fail!("plan not found: {p} (known: {})", doc.plan_ids().join(" "))
    }
    if slug.is_empty() && !set_files && !set_deps && purpose.is_none() && e2e_focus.is_none() {
        fail!("nothing to edit: pass --slug, --files, --deps, --purpose or --e2e-focus")
    }
    let status = doc.field(&p, "status");
    if status == "done" {
        fail!("refused: {p} is done — its files and covers are already reviewed; add a new plan instead (R67)")
    }
    if status == "suspended" {
        fail!("refused: {p} is suspended — resume it before editing its declared files or dependencies")
    }
    let busy = subtree_busy(&doc, &p);
    if !busy.is_empty() {
        fail!("refused: the affected subtree of {p} is in progress ({busy}) — a worker is holding those files (R67)")
    }
    if !slug.is_empty() && !valid_slug(&slug) {
        fail!("slug must match [a-z0-9][a-z0-9-]* (got '{slug}')")
    }
    let purpose = super::free_text("purpose", purpose)?;
    let e2e_focus = super::free_text("e2e-focus", e2e_focus)?;
    let files = match set_files {
        true => validate_files(&files)?,
        false => Vec::new(),
    };

    let plan = doc.plan_mut(&p).expect("the plan was found above");
    if !slug.is_empty() {
        plan.slug = slug;
    }
    if set_files {
        plan.files = files.clone();
    }
    if set_deps {
        plan.deps = csv_list(&deps);
    }
    if let Some(purpose) = purpose {
        plan.purpose = purpose;
    }
    if let Some(e2e_focus) = e2e_focus {
        plan.e2e_focus = e2e_focus;
    }
    if set_deps {
        let known = doc.plan_ids();
        validate_deps(&deps, &known, "plan", &known)?;
        assert_acyclic_plans(&doc)?;
    }
    // Tasks inherited a file list that the edit may have narrowed; a task outside its plan's
    // files is exactly what R64 forbids, so the edit is refused rather than left inconsistent.
    if set_files {
        let mut bad = String::new();
        let plan = doc.plan(&p).expect("the plan was found above");
        for task_file in plan.tasks.iter().flat_map(|task| task.files.iter()) {
            if !files.iter().any(|file| path_within(task_file, file)) {
                bad.push(' ');
                bad.push_str(task_file);
            }
        }
        if !bad.is_empty() {
            fail!("refused: existing tasks of {p} declare files outside the new --files:{bad}")
        }
    }

    let doc = target.write(doc)?;
    say!(ctx, "edited plan {p} ({status})");
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}
