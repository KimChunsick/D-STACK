// verbs/ledger/worker.rs
// dstack worker report: what a delegated Plan covers against what its worker reported (R68).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::utc_now;
use crate::core::target::{resolve_target, TargetKind};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};
use crate::store::cases::{self, CaseRow};
use crate::store::{plan, plan_graph};

const USAGE: &str = "usage: dstack worker report --plan P<id> --from <report-file> [--run <id>]";

/// `cut -c1-120` over the report line the ledger echoes back.
const CUT: usize = 120;

ledger_verb!(WorkerReport, "worker report", report);

/// A worker's silence about a requirement is the quietest way to lose it, so silence becomes an
/// `unreported` ledger row that check coverage refuses to count as evidence.
fn report(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut plan_id, mut file) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        // The shell's loop takes only the two-word form, so `--plan=P1` is the usage error.
        match rest[i].as_str() {
            "--plan" => plan_id = operand(&rest, i)?,
            "--from" => file = operand(&rest, i)?,
            _ => fail!("{USAGE}"),
        }
        i += 2;
    }
    if plan_id.is_empty() || file.is_empty() {
        fail!("{USAGE}")
    }
    if !Path::new(&file).is_file() {
        fail!("report file not found: {file}")
    }
    if target.kind != TargetKind::Run {
        fail!("quick tasks have no plans")
    }
    if !plan::exists(&target.dir) {
        fail!("no plan.json in {}", target.dir.display())
    }
    // D-12: the file is there, so a load error is a corrupt store (exit 2), not "plan not found".
    let doc = plan::load(&target.dir)?;
    if doc.plan(&plan_id).is_none() {
        fail!("plan not found: {plan_id}")
    }

    let text = std::fs::read_to_string(&file)
        .map_err(|e| Error::cannot_decide(format!("cannot read {file}: {e}")))?;
    let mut unreported: Vec<String> = Vec::new();
    let mut reported = 0;
    for r in plan_graph::plan_covers(&doc, &plan_id) {
        match text.lines().find(|line| line.starts_with(&format!("{r}:"))) {
            Some(line) => {
                reported += 1;
                say!(ctx, "  {r}: {}", cut(line));
            }
            None => unreported.push(r),
        }
    }
    say!(
        ctx,
        "reported: {reported} / unreported: {} ({})",
        unreported.len(),
        unreported.join(" ")
    );
    if !unreported.is_empty() {
        cases::ensure(&target.dir)?;
        for r in &unreported {
            cases::append(&target.dir, &silence(r, &plan_id))?;
        }
        say!(
            ctx,
            "  wrote {} unreported row(s) to cases.tsv — the plan is not done until each is covered by evidence",
            unreported.len()
        );
    }
    // Blocked lines are questions for the user; surface them so the main session cannot miss them.
    let blocked = text.lines().filter(|line| is_blocked(line)).count();
    if blocked > 0 {
        say!(
            ctx,
            "  blocked: {blocked} R(s) need a user decision (see the report)"
        );
    }
    Ok(())
}

/// `--name value` where the value is not there: the shell's `shift 2` fails and set -e ends the
/// process with exit 1 and nothing printed.
fn operand(args: &[String], at: usize) -> Result<String> {
    match args.get(at + 1) {
        Some(value) => Ok(value.clone()),
        None => Err(Error::Exit(1)),
    }
}

/// The row that stands for a requirement nobody reported on.
fn silence(r: &str, plan_id: &str) -> CaseRow {
    CaseRow {
        r: r.to_string(),
        case_id: format!("c-worker-{plan_id}"),
        kind: "review".to_string(),
        status: "unreported".to_string(),
        artifact: "-".to_string(),
        sha256: "-".to_string(),
        produced_by: format!("dstack worker report --plan {plan_id}"),
        recorded_at: utc_now(),
        note: format!("not mentioned in the worker report for {plan_id}"),
    }
}

/// `cut -c1-120`: characters, which is what BSD cut counts under a UTF-8 locale.
fn cut(line: &str) -> String {
    line.chars().take(CUT).collect()
}

/// `grep -cE '^R[0-9]+: *blocked'`.
fn is_blocked(line: &str) -> bool {
    let rest = match line.strip_prefix('R') {
        Some(rest) => rest.trim_start_matches(|c: char| c.is_ascii_digit()),
        None => return false,
    };
    if rest.len() == line.len() - 1 {
        return false;
    }
    match rest.strip_prefix(':') {
        Some(rest) => rest.trim_start_matches(' ').starts_with("blocked"),
        None => false,
    }
}

/// worker-report: the fixture is a report file; "reject" means at least one covered R went
/// unreported, which is what the ledger row exists to catch.
pub(super) struct WorkerSelftest;

impl Selftest for WorkerSelftest {
    fn checker(&self) -> &'static str {
        "worker-report"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let run_dir = sandbox.run_dir()?;
        sandbox.write_request(&run_dir)?;
        sandbox.write_plan(&run_dir, &["R01", "R02"])?;
        let (code, out) = sandbox.dsx(
            ctx,
            &[
                "worker",
                "report",
                "--plan",
                "P1",
                "--from",
                &fixture.to_string_lossy(),
            ],
        )?;
        // worker report exits 0 whether or not an R went unreported, so the verdict is in its
        // output; a code past the refusal still means the checker could not run.
        if code > 1 {
            return Err(Error::cannot_decide(format!(
                "selftest: dstack worker report exited {code} instead of deciding"
            )));
        }
        Ok(match out.contains("unreported: 0 ") {
            true => Verdict::Pass,
            false => Verdict::Reject,
        })
    }
}
