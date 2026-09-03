// verbs/plan/add.rs
// dstack plan add and plan insert: one plan minted, with an integer or a decimal id (R67).

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::paths::valid_slug;
use crate::store::plan::{ensure, Plan};
use crate::store::plan_graph::{counts_line, subtree_busy};
use crate::store::plan_ids::{
    assert_acyclic_plans, csv_list, next_decimal_id, next_int_id, validate_deps, validate_files,
};

const OPTION_USAGE: &str =
    "usage: dstack plan add <slug> --milestone M<n> --files a,b [--deps P..]";

plan_verb!(PlanAdd, "plan add", add);
plan_verb!(PlanInsert, "plan insert", insert);

fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    add_impl(ctx, args, false)
}

fn insert(ctx: &mut Context, args: &[String]) -> Result<()> {
    add_impl(ctx, args, true)
}

fn add_impl(ctx: &mut Context, args: &[String], inserting: bool) -> Result<()> {
    let (target, rest) = super::plan_target(ctx, args)?;
    let (mut slug, mut ms) = (String::new(), String::new());
    let (mut files, mut deps, mut after) = (String::new(), String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(arg, next, "milestone")? {
            ms = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "files")? {
            files = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "deps")? {
            deps = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(arg, next, "after")? {
            after = value;
            i += eaten;
        } else if is_option(arg) {
            fail!("unknown option: {arg} ({OPTION_USAGE})")
        } else if slug.is_empty() {
            slug = arg.to_string();
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if slug.is_empty() {
        let (verb, tail) = match inserting {
            true => ("insert", " --after P<n>"),
            false => ("add", ""),
        };
        fail!("usage: dstack plan {verb} <slug> --milestone M<n> --files a,b [--deps P..]{tail}")
    }
    if !valid_slug(&slug) {
        fail!("slug must match [a-z0-9][a-z0-9-]* (got '{slug}')")
    }
    if inserting {
        if after.is_empty() {
            fail!("dstack plan insert needs --after P<n> (which plan the new one follows)")
        }
        target.require()?;
    } else {
        if !after.is_empty() {
            fail!("--after belongs to dstack plan insert, not plan add")
        }
        ensure(&target.dir)?;
    }
    let files = validate_files(&files)?;

    let mut doc = target.load()?;
    let pids = doc.plan_ids();
    let id = if inserting {
        if !pids.contains(&after) {
            fail!("plan not found: {after} (known: {})", pids.join(" "))
        }
        let busy = subtree_busy(&doc, &after);
        if !busy.is_empty() {
            fail!("refused: the affected subtree of {after} is in progress ({busy}) — finish or reset those plans before inserting (R67)")
        }
        if ms.is_empty() {
            ms = doc.field(&after, "milestone");
        }
        next_decimal_id(&after, &pids)?
    } else {
        next_int_id(&doc, "P")
    };
    if ms.is_empty() {
        fail!("--milestone is required (dstack milestone add <slug> mints one)")
    }
    let mids: Vec<String> = doc.milestones.iter().map(|m| m.id.clone()).collect();
    if !mids.contains(&ms) {
        fail!("milestone not found: {ms} (known: {})", mids.join(" "))
    }

    let new = Plan {
        id: id.clone(),
        milestone: ms.clone(),
        slug: slug.clone(),
        files: files.clone(),
        deps: csv_list(&deps),
        status: "pending".to_string(),
        worktree: String::new(),
        started_at: String::new(),
        done_at: String::new(),
        tasks: Vec::new(),
    };
    let mut plans: Vec<Plan> = Vec::new();
    for plan in doc.plans {
        let here = plan.id == after;
        plans.push(plan);
        if here {
            plans.push(new.clone());
        }
    }
    if after.is_empty() {
        plans.push(new);
    }
    doc.plans = plans;
    // Existence is checked against the NEW graph on purpose: that is what makes a self-dependency
    // ("--deps P2" on the plan that becomes P2) reach the cycle check instead of reading as a typo.
    let deps = validate_deps(&deps, &doc.plan_ids(), "plan", &pids)?;
    assert_acyclic_plans(&doc)?;

    let doc = target.write(doc)?;
    say!(ctx, "plan {id}: {slug} (milestone {ms})");
    say!(ctx, "  files: {}", files.join(" "));
    say!(ctx, "  deps:  {}", deps.join(" "));
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}
