// verbs/e2e_brief.rs
// dstack e2e brief --milestone M<n>: each Plan's E2E focus, covered R rows and open cases (R07);
// --goal: every Goal QA scenario with its text and the open ones (R08).
//
// It prints the part of the e2e-runner brief the ledgers hold, verbatim, and only reads:
// plan.json, request.md, cases.tsv and the QA ledger stay as they are.

use std::path::Path;

use crate::core::args::{is_option, opt};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::paths::{base_name, parse_rid};
use crate::core::verb::Verb;
use crate::store::cases;
use crate::store::plan::Plan;
use crate::store::qa::{self, QaRow, NO_SCENARIO};
use crate::store::request::RequestDoc;
use crate::store::rows::Row;
use crate::verbs::plan::plan_target;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

const USAGE: &str = "usage: dstack e2e brief --milestone M<n> | --goal";

/// Case statuses that leave nothing for the runner to do.
const CLOSED: [&str; 3] = ["met", "skipped", "retired"];

struct E2eBrief;

impl Verb for E2eBrief {
    fn name(&self) -> &'static str {
        "e2e brief"
    }

    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        brief(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(E2eBrief)]
}

/// One live R row the Milestone's tasks cover: the row, its line exactly as request.md holds it,
/// and the Plans of the Milestone whose tasks cover it.
struct Covered {
    row: Row,
    line: String,
    plans: Vec<String>,
}

/// The reason on its own line, then the usage line the refusal ends with.
fn refuse(ctx: &mut Context, reason: &str) -> Error {
    ctx.out.err_line(&format!("dstack: {reason}"));
    Error::failed(USAGE)
}

fn or_none(value: &str) -> &str {
    match value.trim().is_empty() {
        true => "(none)",
        false => value,
    }
}

/// What the brief covers: one Milestone's Plans, or the Goal's QA scenarios.
enum Scope {
    Milestone(String),
    Goal,
}

/// The two options the verb takes, one of them exactly.
fn scope_arg(ctx: &mut Context, rest: &[String]) -> Result<Scope> {
    let (mut m, mut goal) = (None, false);
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].as_str();
        if let Some((value, eaten)) = opt(arg, rest.get(i + 1).map(String::as_str), "milestone")? {
            m = Some(value);
            i += eaten;
        } else if arg == "--goal" {
            goal = true;
            i += 1;
        } else if is_option(arg) {
            return Err(refuse(ctx, &format!("unknown option: {arg}")));
        } else {
            return Err(refuse(ctx, &format!("unexpected argument: {arg}")));
        }
    }
    match (m, goal) {
        (Some(_), true) => Err(refuse(ctx, "--milestone and --goal are mutually exclusive")),
        (None, true) => Ok(Scope::Goal),
        (Some(m), false) if !m.is_empty() => Ok(Scope::Milestone(m)),
        _ => Err(Error::failed(USAGE)),
    }
}

/// request.md as milestone brief reads it for the run's e2e value (verbs/plan/confirm.rs).
fn request_doc(dir: &Path) -> Result<RequestDoc> {
    let file = dir.join("request.md");
    if !file.is_file() {
        return Err(Error::failed(format!(
            "no request.md in {} (dstack request new --type <work_type>)",
            dir.display()
        )));
    }
    RequestDoc::load(&file)
}

/// Every live row some task of these Plans covers, first occurrence of an id only, in R order.
fn covered_rows(request: &RequestDoc, plans: &[&Plan]) -> Vec<Covered> {
    let lines: Vec<&str> = request.text().split('\n').collect();
    let mut covered: Vec<Covered> = Vec::new();
    for row in request.rows() {
        if !row.is_live() || covered.iter().any(|held| held.row.id == row.id) {
            continue;
        }
        let ids: Vec<String> = plans
            .iter()
            .filter(|plan| plan.tasks.iter().any(|task| task.covers.contains(&row.id)))
            .map(|plan| plan.id.clone())
            .collect();
        if ids.is_empty() {
            continue;
        }
        let line = lines[row.lineno - 1].to_string();
        covered.push(Covered { row, line, plans: ids });
    }
    covered.sort_by_key(|held| parse_rid(&held.row.id));
    covered
}

fn brief(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = plan_target(ctx, args)?;
    let m = match scope_arg(ctx, &rest)? {
        Scope::Milestone(m) => m,
        Scope::Goal => return goal_brief(ctx, &target.dir),
    };
    target.require()?;
    let doc = target.load()?;
    let milestone = match doc.milestones.iter().find(|milestone| milestone.id == m) {
        Some(milestone) => milestone,
        None => {
            let ids: Vec<&str> = doc.milestones.iter().map(|m| m.id.as_str()).collect();
            let reason = format!("milestone not found: {m} (known: {})", ids.join(" "));
            return Err(refuse(ctx, &reason));
        }
    };
    let plans: Vec<&Plan> = doc.plans.iter().filter(|plan| plan.milestone == m).collect();
    if plans.is_empty() {
        let reason = format!("milestone {m} has no plans: dstack plan add <slug> --milestone {m}");
        return Err(refuse(ctx, &reason));
    }
    let request = request_doc(&target.dir)?;
    let e2e = request.field("e2e").unwrap_or_default();

    say!(
        ctx,
        "e2e brief: run {} — milestone {} {} — goal: {} — e2e: {}",
        base_name(&target.dir),
        milestone.id,
        milestone.slug,
        or_none(&milestone.goal),
        if e2e.is_empty() { "(not set)" } else { &e2e }
    );
    if e2e == "none" {
        say!(ctx, "e2e: none — the milestone has no E2E cases to run");
        return Ok(());
    }

    say!(ctx, "\n## Plans");
    for plan in &plans {
        say!(
            ctx,
            "- {} {} ({}) — purpose: {} — E2E focus: {}",
            plan.id,
            plan.slug,
            or_none(&plan.status),
            or_none(&plan.purpose),
            or_none(&plan.e2e_focus)
        );
    }

    let covered = covered_rows(&request, &plans);
    say!(ctx, "\n## R rows (verbatim from request.md)");
    if covered.is_empty() {
        say!(ctx, "(none)");
    }
    for held in &covered {
        ctx.out.say(&held.line);
    }

    // Only the cases of the run's own e2e kind are the runner's; test and review cases belong
    // to the workers and the reviewers.
    let ledger = cases::rows(&target.dir)?;
    let mut open = Vec::new();
    for held in &covered {
        for case in &ledger {
            if case.r == held.row.id && case.kind == e2e && !CLOSED.contains(&case.status.as_str()) {
                open.push((held, &case.case_id));
            }
        }
    }
    say!(ctx, "\n## Cases");
    if open.is_empty() {
        say!(ctx, "(no open cases)");
        return Ok(());
    }
    say!(ctx, "| R | case | acceptance criterion (verbatim from the request) | Plans |");
    say!(ctx, "|---|---|---|---|");
    for (held, case_id) in open {
        say!(
            ctx,
            "| {} | {} | {} | {} |",
            held.row.id,
            case_id,
            held.row.accept,
            held.plans.join(", ")
        );
    }
    Ok(())
}

/// The Goal QA scenarios for the runner (D-40): each one's usage scenario, status and text as
/// written, then the ones still open. QA scenarios can be recorded before any plan, so this
/// needs no plan.json, and the e2e value is shown, not obeyed: Goal-close QA runs either way.
fn goal_brief(ctx: &mut Context, dir: &Path) -> Result<()> {
    let request = request_doc(dir)?;
    let e2e = request.field("e2e").unwrap_or_default();
    say!(
        ctx,
        "e2e brief: run {} — Goal QA — e2e: {}",
        base_name(dir),
        if e2e.is_empty() { "(not set)" } else { &e2e }
    );
    let rows = qa::rows(dir)?;
    if rows.is_empty() {
        say!(ctx, "(no QA scenarios — run dstack qa add)");
        return Ok(());
    }
    // Only a QA scenario tied to a usage scenario needs the request's ## 사용 시나리오 outline.
    let titles = match rows.iter().any(|row| row.scenario != NO_SCENARIO) {
        true => qa::scenarios(request.text())?,
        false => Vec::new(),
    };
    let usage = |row: &QaRow| {
        if row.scenario == NO_SCENARIO {
            return "(none)".to_string();
        }
        match titles.iter().find(|(id, _)| *id == row.scenario) {
            Some((id, title)) if !title.is_empty() => format!("{id} {title}"),
            _ => row.scenario.clone(),
        }
    };

    say!(ctx, "\n## QA scenarios");
    for row in &rows {
        say!(ctx, "\n### {} — usage scenario: {} — status: {}", row.qa, usage(row), row.status);
        // The runner works from this text, so a ledger row whose text is gone cannot be briefed.
        let text = qa::body_text(dir, row)?.ok_or_else(|| {
            Error::cannot_decide(format!("QA text missing: {}", dir.join(&row.body).display()))
        })?;
        ctx.out.say(text.strip_suffix('\n').unwrap_or(&text));
    }
    say!(ctx, "\n## Open QA scenarios");
    let open: Vec<&QaRow> = rows.iter().filter(|row| row.status == "open").collect();
    if open.is_empty() {
        say!(ctx, "(no open QA scenarios)");
        return Ok(());
    }
    say!(ctx, "| QA | usage scenario |");
    say!(ctx, "|---|---|");
    for row in open {
        say!(ctx, "| {} | {} |", row.qa, usage(row));
    }
    Ok(())
}
