// verbs/plan/task.rs
// dstack task add and task done: a task covers live R ids, owns files inside its plan (R60, R64).

use std::path::{Path, PathBuf};

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::fsx::utc_now;
use crate::core::paths::valid_slug;
use crate::core::roots::git_out;
use crate::store::plan::Task;
use crate::store::plan_graph::counts_line;
use crate::store::plan_ids::{
    assert_acyclic_tasks, csv_list, path_within, validate_deps, validate_files,
};
use crate::store::request::RequestDoc;

plan_verb!(TaskAdd, "task add", add);
plan_verb!(TaskDone, "task done", done);

fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let (mut slug, mut p) = (String::new(), String::new());
    let (mut covers, mut files, mut deps) = (String::new(), String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(arg, next, "plan")? {
            p = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "covers")? {
            covers = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "files")? {
            files = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "deps")? {
            deps = value;
            i += eaten;
        } else if is_option(arg) {
            fail!("unknown option: {arg} (usage: dstack task add <slug> --plan P<n> --covers R.. --files a,b [--deps T..])")
        } else if slug.is_empty() {
            slug = arg.to_string();
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if slug.is_empty() {
        fail!("usage: dstack task add <slug> --plan P<n> --covers R.. --files a,b [--deps T..]")
    }
    if !valid_slug(&slug) {
        fail!("slug must match [a-z0-9][a-z0-9-]* (got '{slug}')")
    }
    if p.is_empty() {
        fail!("--plan P<n> is required (a task lives inside exactly one plan, R60)")
    }
    let mut doc = target.load()?;
    if !doc.plan_ids().contains(&p) {
        fail!("plan not found: {p} (known: {})", doc.plan_ids().join(" "))
    }
    if doc.field(&p, "status") == "done" {
        fail!("refused: {p} is done — reopening a reviewed plan is not how a late requirement lands; use dstack plan insert --after {p} (R67)")
    }
    let files = validate_files(&files)?;
    if covers.is_empty() {
        fail!(
            "--covers R.. is required: a task that covers no requirement cannot be verified (R64)"
        )
    }

    // Task files ⊆ plan files: the plan's declared files are what `dstack next` schedules on and
    // what the review bundle diffs, so a task outside them is invisible to both.
    let plan_files = doc
        .plan(&p)
        .expect("the plan was found above")
        .files
        .clone();
    for file in &files {
        if !plan_files.iter().any(|owned| path_within(file, owned)) {
            fail!(
                "file outside the plan: '{file}' is not covered by {p} --files ({})",
                plan_files.join(", ")
            )
        }
    }

    // covers must be LIVE requirement ids: a withdrawn/deferred/superseded/pending row is not
    // work anybody may claim, and check coverage would never look at it (R65).
    let request = target.dir.join("request.md");
    if !request.is_file() {
        fail!(
            "no request.md in {} — --covers must name R rows that exist (dstack request new --type <work_type>)",
            target.dir.display()
        )
    }
    let req = RequestDoc::load(&request)?;
    let live = req.live_ids();
    let covers_list = csv_list(&covers);
    for r in &covers_list {
        let digit = r.strip_prefix('R').and_then(|rest| rest.chars().next());
        if !digit.map(|c| c.is_ascii_digit()).unwrap_or(false) {
            fail!("not a requirement id: '{r}' (expected R<NN>)")
        }
        if !live.contains(r) {
            fail!(
                "{r} is not a live requirement: {}",
                cover_reason(&req, &request, r)
            )
        }
    }

    let tids = doc.task_ids();
    let id = format!("T{}", tids.iter().map(shell_number).max().unwrap_or(0) + 1);
    let task = Task {
        id: id.clone(),
        slug: slug.clone(),
        covers: covers_list.clone(),
        files: files.clone(),
        deps: csv_list(&deps),
        commit: String::new(),
        done_at: String::new(),
    };
    doc.plan_mut(&p)
        .expect("the plan was found above")
        .tasks
        .push(task);
    validate_deps(&deps, &doc.task_ids(), "task", &tids)?;
    assert_acyclic_tasks(&doc)?;

    let doc = target.write(doc)?;
    say!(ctx, "task {id}: {slug} (plan {p})");
    say!(ctx, "  covers: {}", covers_list.join(" "));
    say!(ctx, "  files:  {}", files.join(" "));
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}

/// awk's `substr($0, 2) + 0`: the digits after the leading letter, and 0 when there are none.
fn shell_number(id: &String) -> u32 {
    let digits: String = id
        .chars()
        .skip(1)
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().unwrap_or(0)
}

/// _cover_reason(): why the row is not live, in the words of its own markers.
fn cover_reason(req: &RequestDoc, path: &Path, r: &str) -> String {
    let row = match req.row(r) {
        Some(row) => row,
        None => return format!("no such row in {}", path.display()),
    };
    let markers = row.markers_string();
    let joined = format!(";{markers};");
    let of = |key: &str| row.marker(key).unwrap_or_default();
    if joined.contains(";status=pending-approval;") {
        "the row is still pending approval (dstack request approve)".to_string()
    } else if joined.contains(";withdrawn=") {
        format!("the row is withdrawn: {}", of("withdrawn"))
    } else if joined.contains(";deferred=") {
        format!("the row is deferred: {}", of("deferred"))
    } else if joined.contains(";superseded-by=") {
        format!(
            "the row was split into {} — cover those instead",
            of("superseded-by")
        )
    } else {
        let markers = match markers.is_empty() {
            true => "none".to_string(),
            false => markers,
        };
        format!("the row is not countable (markers: {markers})")
    }
}

fn done(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let (mut t, mut sha) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(arg, next, "commit")? {
            sha = value;
            i += eaten;
        } else if is_option(arg) {
            fail!("unknown option: {arg} (usage: dstack task done T<n> --commit <sha>)")
        } else if t.is_empty() {
            t = arg.to_string();
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if t.is_empty() {
        fail!("usage: dstack task done T<n> --commit <sha>")
    }
    let mut doc = target.load()?;
    if !doc.task_ids().contains(&t) {
        fail!("task not found: {t} (known: {})", doc.task_ids().join(" "))
    }
    if sha.is_empty() {
        fail!("--commit <sha> is required: a task is exactly one commit (R60)")
    }
    let owner = doc
        .plans
        .iter()
        .find(|plan| plan.tasks.iter().any(|task| task.id == t))
        .map(|plan| plan.id.clone())
        .expect("the task was found above");
    let plan_worktree = doc.field(&owner, "worktree");
    let worktree = match !plan_worktree.is_empty() && Path::new(&plan_worktree).is_dir() {
        true => PathBuf::from(&plan_worktree),
        false => target.worktree()?,
    };
    // An unverified sha would let a task read as done while pointing at nothing reviewable.
    if git_out(
        Some(&worktree),
        &["cat-file", "-e", &format!("{sha}^{{commit}}")],
    )
    .is_none()
    {
        fail!(
            "commit not found in {}: {sha} (task {t} of plan {owner})",
            worktree.display()
        )
    }
    let now = utc_now();
    for plan in doc.plans.iter_mut() {
        for task in plan.tasks.iter_mut().filter(|task| task.id == t) {
            task.commit = sha.clone();
            task.done_at = now.clone();
        }
    }

    let doc = target.write(doc)?;
    say!(
        ctx,
        "task {t} (plan {owner}): commit {sha} verified in {}, done at {now}",
        worktree.display()
    );
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}
