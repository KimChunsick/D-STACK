// verbs/plan/confirm.rs
// dstack milestone brief and milestone confirm: show a Milestone's decomposition with the Goal's
// QA scenarios and record the user's confirmation of its Plans, the list plan start and next read
// before a worker runs (R12).

use std::path::Path;

use crate::core::args::is_option;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::store::plan::{Milestone, Plan, PlanDoc};
use crate::store::plan_graph::counts_line;
use crate::store::qa;
use crate::store::request::RequestDoc;

const BRIEF_USAGE: &str = "usage: dstack milestone brief M<n>";
const CONFIRM_USAGE: &str = "usage: dstack milestone confirm M<n>";

plan_verb!(MilestoneBrief, "milestone brief", brief);
plan_verb!(MilestoneConfirm, "milestone confirm", confirm);

/// Why a Plan may not run yet, or None: its Milestone has not confirmed it, or the run checks
/// E2E (e2e is not none) and the Plan has no focus to check. plan start and next exclude on it.
pub fn gate_reason(doc: &PlanDoc, e2e: &str, plan_id: &str) -> Option<String> {
    let plan = doc.plan(plan_id);
    let m = plan.map(|plan| plan.milestone.as_str()).unwrap_or("");
    let confirmed = doc
        .milestones
        .iter()
        .any(|milestone| milestone.id == m && milestone.confirmed.iter().any(|id| id == plan_id));
    if !confirmed {
        return Some(format!(
            "milestone {m} is not confirmed for {plan_id}: run dstack milestone brief {m}, then dstack milestone confirm {m}"
        ));
    }
    if e2e != "none" && plan.is_none_or(|plan| blank(&plan.e2e_focus)) {
        return Some(focus_missing(plan_id));
    }
    None
}

/// The run request's `e2e` value, "" when its frontmatter does not set it (which checks focus).
pub(super) fn request_e2e(dir: &Path) -> Result<String> {
    let file = dir.join("request.md");
    if !file.is_file() {
        fail!(
            "no request.md in {} (dstack request new --type <work_type>)",
            dir.display()
        )
    }
    Ok(RequestDoc::load(&file)?.field("e2e").unwrap_or_default())
}

/// The e2e value plan start and next judge by: a run without request.md (a legacy run) has no
/// e2e value set, so its Plans need a focus; only milestone brief and confirm require the file.
pub(super) fn gate_e2e(dir: &Path) -> Result<String> {
    match dir.join("request.md").is_file() {
        true => request_e2e(dir),
        false => Ok(String::new()),
    }
}

/// The line that says which rule the run's e2e value sets for the E2E focus.
pub(super) fn e2e_line(e2e: &str) -> String {
    match e2e {
        "none" => "e2e: none — the E2E focus check is skipped".to_string(),
        "" => "e2e: (not set) — every Plan needs an E2E focus before it starts".to_string(),
        other => format!("e2e: {other} — every Plan needs an E2E focus before it starts"),
    }
}

fn focus_missing(plan_id: &str) -> String {
    format!("{plan_id} has no E2E focus: dstack plan edit {plan_id} --e2e-focus <text>, then confirm again")
}

fn blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn or_none(value: &str) -> &str {
    match blank(value) {
        true => "(none)",
        false => value,
    }
}

fn list(values: &[String]) -> String {
    or_none(&values.join(", ")).to_string()
}

/// The one positional M<n> both verbs take, and the Milestone it names in the loaded ledger.
fn milestone_arg(
    ctx: &mut Context,
    args: &[String],
    usage: &str,
) -> Result<(super::Target, PlanDoc, Milestone)> {
    let (target, rest) = super::plan_target(ctx, args)?;
    target.require()?;
    let mut m = String::new();
    for arg in &rest {
        if is_option(arg) {
            fail!("unknown option: {arg} ({usage})")
        } else if m.is_empty() {
            m = arg.to_string();
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if m.is_empty() {
        fail!("{usage}")
    }
    let doc = target.load()?;
    let milestone = match doc.milestones.iter().find(|milestone| milestone.id == m) {
        Some(milestone) => milestone.clone(),
        None => {
            let ids: Vec<&str> = doc.milestones.iter().map(|m| m.id.as_str()).collect();
            fail!("milestone not found: {m} (known: {})", ids.join(" "))
        }
    };
    Ok((target, doc, milestone))
}

fn plans_of<'a>(doc: &'a PlanDoc, milestone: &Milestone) -> Vec<&'a Plan> {
    doc.plans.iter().filter(|plan| plan.milestone == milestone.id).collect()
}

fn brief(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, doc, milestone) = milestone_arg(ctx, args, BRIEF_USAGE)?;
    let e2e = request_e2e(&target.dir)?;
    let plans = plans_of(&doc, &milestone);
    say!(ctx, "milestone {}: {}", milestone.id, milestone.slug);
    say!(ctx, "  goal: {}", or_none(&milestone.goal));
    say!(ctx, "  {}", e2e_line(&e2e));
    if plans.is_empty() {
        say!(ctx, "  plans: (none)");
    }
    for plan in &plans {
        say!(ctx, "plan {}: {}", plan.id, plan.slug);
        say!(ctx, "  status:    {}", or_none(&plan.status));
        say!(ctx, "  purpose:   {}", or_none(&plan.purpose));
        say!(ctx, "  e2e focus: {}", or_none(&plan.e2e_focus));
        say!(ctx, "  deps:      {}", list(&plan.deps));
        say!(ctx, "  files:     {}", list(&plan.files));
        if plan.tasks.is_empty() {
            say!(ctx, "  tasks:     (none)");
        }
        for task in &plan.tasks {
            say!(ctx, "  task {}: {}", task.id, task.slug);
            say!(ctx, "    purpose: {}", or_none(&task.purpose));
            say!(ctx, "    covers:  {}", list(&task.covers));
        }
    }
    // The QA scenarios belong to the Goal, so every Milestone's brief shows all of them.
    let scenarios = qa::rows(&target.dir)?;
    if scenarios.is_empty() {
        say!(ctx, "QA scenarios: (none yet)");
    } else {
        say!(ctx, "QA scenarios:");
    }
    for row in &scenarios {
        say!(ctx, "  {}: scenario {}, status {}", row.qa, row.scenario, row.status);
    }
    let open: Vec<String> = plans
        .iter()
        .filter(|plan| !milestone.confirmed.contains(&plan.id))
        .map(|plan| plan.id.clone())
        .collect();
    let confirmed = !plans.is_empty() && open.is_empty();
    say!(ctx, "confirmed: {}", if confirmed { "yes" } else { "no" });
    say!(ctx, "  confirmed plans: {}", list(&milestone.confirmed));
    say!(ctx, "  not confirmed:   {}", list(&open));
    if !confirmed {
        say!(ctx, "  next: dstack milestone confirm {}", milestone.id);
    }
    Ok(())
}

/// Every reason the Milestone cannot be confirmed as it stands: a Plan that has not started
/// lacks its purpose or (e2e on) its E2E focus, or there is no Plan to confirm at all.
fn plan_problems(milestone: &Milestone, plans: &[&Plan], e2e: &str) -> Vec<String> {
    let mut problems = Vec::new();
    if plans.is_empty() {
        let m = &milestone.id;
        problems.push(format!(
            "{m} has no plans: dstack plan add <slug> --milestone {m}, then confirm again"
        ));
    }
    for plan in plans {
        if !matches!(plan.status.as_str(), "pending" | "ready" | "blocked") {
            continue;
        }
        let p = &plan.id;
        if blank(&plan.purpose) {
            problems.push(format!(
                "{p} has no purpose: dstack plan edit {p} --purpose <text>, then confirm again"
            ));
        }
        if e2e != "none" && blank(&plan.e2e_focus) {
            problems.push(focus_missing(p));
        }
    }
    problems
}

fn confirm(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, mut doc, milestone) = milestone_arg(ctx, args, CONFIRM_USAGE)?;
    let e2e = request_e2e(&target.dir)?;
    let plans = plans_of(&doc, &milestone);
    let mut problems = plan_problems(&milestone, &plans, &e2e);
    let ids: Vec<String> = plans.iter().map(|plan| plan.id.clone()).collect();

    // R16: the whole request.md through lint-ko, the same call request approve makes.
    ctx.out.say("== Korean lint (R16, the whole request.md)");
    let file = target.dir.join("request.md");
    let called = ctx.call("lint-ko", &[file.display().to_string()]);
    let merged = format!("{}{}", called.stdout, called.stderr);
    ctx.out.say(merged.trim_end_matches('\n'));
    match called.code {
        0 => {}
        2 => return Err(Error::Exit(2)),
        _ => problems.push(
            "request.md has S1 Korean lint hits above: fix the wording, then confirm again".into(),
        ),
    }
    if !problems.is_empty() {
        for problem in &problems {
            ctx.out.err_line(problem);
        }
        fail!(
            "refused: milestone {} is not confirmed — {} problem(s) above",
            milestone.id,
            problems.len()
        )
    }

    // Replaced, not merged: the list names what the user saw in this brief, so a Plan added
    // later stays out until confirm runs again.
    doc.milestones
        .iter_mut()
        .find(|m| m.id == milestone.id)
        .expect("the milestone was found above")
        .confirmed = ids.clone();
    let doc = target.write(doc)?;
    say!(ctx, "confirmed milestone {}: {}", milestone.id, milestone.slug);
    say!(ctx, "  plans: {}", ids.join(", "));
    say!(ctx, "  {}", e2e_line(&e2e));
    say!(ctx, "  {}", counts_line(&doc));
    Ok(())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn doc(confirmed: &[&str], focus: &str) -> PlanDoc {
        let milestone = Milestone {
            id: "M1".into(),
            confirmed: confirmed.iter().map(|id| id.to_string()).collect(),
            ..Milestone::default()
        };
        let plan = Plan {
            id: "P1".into(),
            milestone: "M1".into(),
            e2e_focus: focus.into(),
            ..Plan::default()
        };
        PlanDoc {
            v: 2,
            milestones: vec![milestone],
            plans: vec![plan],
        }
    }

    #[test]
    fn R12_gate_reason_names_an_unconfirmed_plan() {
        assert_eq!(
            gate_reason(&doc(&["P2"], "출력을 봐요"), "cli", "P1").as_deref(),
            Some("milestone M1 is not confirmed for P1: run dstack milestone brief M1, then dstack milestone confirm M1")
        );
    }

    #[test]
    fn R12_gate_reason_names_a_missing_focus_unless_e2e_is_none() {
        assert_eq!(
            gate_reason(&doc(&["P1"], ""), "cli", "P1").as_deref(),
            Some("P1 has no E2E focus: dstack plan edit P1 --e2e-focus <text>, then confirm again")
        );
        assert_eq!(gate_reason(&doc(&["P1"], ""), "none", "P1"), None);
    }

    #[test]
    fn R12_gate_reason_passes_a_confirmed_plan_with_focus() {
        assert_eq!(gate_reason(&doc(&["P1"], "출력을 봐요"), "cli", "P1"), None);
    }
}
