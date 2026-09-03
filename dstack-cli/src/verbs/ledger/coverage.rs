// verbs/ledger/coverage.rs
// dstack check coverage: every live R needs a covering task and an evidence row (R65).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::target::{resolve_target, TargetKind};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};
use crate::store::cases::CaseRow;
use crate::store::plan::PlanDoc;
use crate::store::request::RequestDoc;
use crate::store::{cases, plan, plan_graph};

use super::{kind_word, verdict};

/// The statuses that count as recorded evidence; open, skipped, unreported and retired do not.
const RECORDED: [&str; 3] = ["met", "abstain", "blocked"];

ledger_verb!(CheckCoverage, "check coverage", coverage);

/// "<tasks> <evidence rows> <ok|task|evidence|both>" for one R, reading both files itself.
/// `report` reads the same answer so its `coverage` column can never disagree with
/// `dstack check coverage` (R79); a caller with many R ids uses coverage_of instead.
pub fn coverage_for_r(dir: &Path, kind: TargetKind, r: &str) -> Result<(String, String, String)> {
    Ok(coverage_of(
        plan_doc(dir)?.as_ref(),
        &cases::rows(dir)?,
        kind,
        r,
    ))
}

/// D-12: a plan.json that is not there is "no plan at all" — the state before `plan add` — but
/// one that cannot be read or parsed is a store this command cannot decide on (exit 2). Reading
/// the second as the first would turn a corrupt file into a quiet "nothing covers this R".
fn plan_doc(dir: &Path) -> Result<Option<PlanDoc>> {
    match plan::exists(dir) {
        false => Ok(None),
        true => plan::load(dir).map(Some),
    }
}

/// The same answer from documents the caller already holds: `check coverage` walks every live R
/// and would otherwise reparse plan.json and cases.tsv once per row.
pub fn coverage_of(
    plan: Option<&PlanDoc>,
    rows: &[CaseRow],
    kind: TargetKind,
    r: &str,
) -> (String, String, String) {
    // A quick task has no plan.json (R99): the request row itself is the unit of work, so the
    // task half of coverage is satisfied by the request existing.
    let tasks = match kind {
        TargetKind::Quick => "n/a".to_string(),
        TargetKind::Run => plan
            .map(|doc| plan_graph::tasks_covering(doc, r).len())
            .unwrap_or(0)
            .to_string(),
    };
    let evidence = rows
        .iter()
        .filter(|row| row.r == r && RECORDED.contains(&row.status.as_str()))
        .count();
    let mut missing = String::new();
    if tasks == "0" {
        missing = "task".to_string();
    }
    if evidence == 0 {
        missing = match missing.is_empty() {
            true => "evidence".to_string(),
            false => "both".to_string(),
        };
    }
    if missing.is_empty() {
        missing = "ok".to_string();
    }
    (tasks, evidence.to_string(), missing)
}

fn coverage(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    if let Some(first) = rest.first() {
        fail!(
            "unexpected argument: {first} (usage: dstack check coverage [--run <id>|--quick <slug>])"
        )
    }
    let request = target.dir.join("request.md");
    if !request.is_file() {
        fail!(
            "no request.md in {} — nothing to cover (dstack request new --type <work_type>)",
            target.dir.display()
        )
    }
    let doc = RequestDoc::load(&request)?;
    let plan = plan_doc(&target.dir)?;
    let ledger = cases::rows(&target.dir)?;
    say!(
        ctx,
        "check coverage: {} {}",
        kind_word(target.kind),
        target.id
    );
    let (mut rows, mut covered, mut proven, mut missing) = (0, 0, 0, 0);
    for r in doc.live_ids() {
        rows += 1;
        let (tasks, evidence, miss) = coverage_of(plan.as_ref(), &ledger, target.kind, &r);
        if miss != "task" && miss != "both" {
            covered += 1;
        }
        if evidence != "0" {
            proven += 1;
        }
        match miss.as_str() {
            "ok" => say!(ctx, "{r} tasks={tasks} evidence={evidence} ok"),
            _ => {
                missing += 1;
                say!(ctx, "{r} tasks={tasks} evidence={evidence} MISSING({miss})");
            }
        }
    }
    // R68's unreported rows are not a coverage failure by themselves, but a coverage report that
    // hides them lets a worker's silence disappear.
    let unreported: Vec<String> = ledger
        .iter()
        .filter(|row| row.status == "unreported")
        .map(|row| format!("{}/{}", row.r, row.case_id))
        .collect();
    say!(
        ctx,
        "R {rows}: covered {covered}, evidence {proven}, missing {missing}"
    );
    let ids = match unreported.is_empty() {
        true => String::new(),
        false => format!(" ({})", unreported.join(" ")),
    };
    say!(ctx, "unreported rows: {}{ids}", unreported.len());
    if missing > 0 {
        return Err(Error::Exit(1));
    }
    Ok(())
}

/// check-coverage: the fixture is a request.md; its directives say which R ids get a covering
/// task and which get evidence, so "no evidence for R02" is data in the fixture.
pub(super) struct CoverageSelftest;

impl Selftest for CoverageSelftest {
    fn checker(&self) -> &'static str {
        "check-coverage"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let run_dir = sandbox.run_dir()?;
        std::fs::copy(fixture, run_dir.join("request.md")).map_err(|e| {
            Error::cannot_decide(format!("selftest: cannot stage {}: {e}", fixture.display()))
        })?;
        sandbox.approve(&run_dir)?;
        let _ = sandbox.dsx(ctx, &["cases", "sync"])?;
        let tasks = Sandbox::directive(fixture, "tasks").unwrap_or_default();
        sandbox.write_plan(&run_dir, &tasks.split_whitespace().collect::<Vec<&str>>())?;
        for r in Sandbox::directive(fixture, "evidence")
            .unwrap_or_default()
            .split_whitespace()
        {
            let artifact = sandbox.artifact(
                &format!("{r}.txt"),
                &format!("{r} verified: checked 1, missing 0"),
            )?;
            let _ = sandbox.dsx(
                ctx,
                &[
                    "evidence",
                    "add",
                    "--r",
                    r,
                    "--case",
                    "c1",
                    "--kind",
                    "cli",
                    "--artifact",
                    &artifact.to_string_lossy(),
                    "--produced-by",
                    "selftest",
                ],
            )?;
        }
        let (code, _) = sandbox.dsx(ctx, &["check", "coverage"])?;
        verdict(code, "dstack check coverage")
    }
}
