// verbs/report/mod.rs
// dstack report: the completion report R79 asks for — one table row per requirement.
//
// The report computes no status of its own. Coverage comes from coverage_of (what
// `dstack check coverage` prints), the per-R state from verify::states (what `dstack verify`
// prints), the decision check from `dstack check decisions` as a self-call. A second opinion
// here would be a second truth.

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::target::{resolve_target, TargetKind};
use crate::core::verb::Verb;
use crate::selftest::Selftest;
use crate::store::cases;
use crate::store::plan::{self, PlanDoc};
use crate::store::plan_graph::tasks_covering;
use crate::store::request::RequestDoc;
use crate::store::tsv::undash;
use crate::verbs::ledger::coverage::coverage_of;
use crate::verbs::verify::reasons_pretty;
use crate::verbs::verify::states::{self, branch_line, kind_word, policy_violations};

use metrics::run_metrics;

pub mod metrics;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(Error::failed(format!($($m)*))) }; }

struct Report;

impl Verb for Report {
    fn name(&self) -> &'static str {
        "report"
    }
    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        report(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(Report)]
}

/// The report has no fixture of its own: every judgement it prints belongs to the checker that
/// computed it (verify, check coverage, check decisions), and those own the fixtures.
pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![]
}

/// Reason tokens from verify::states become check names, in the order R79 lists the checks.
fn reason_names(reasons: &str) -> String {
    let mut out = String::new();
    let mut named = false;
    for token in reasons.split(';') {
        let last = || token.rsplit(':').next().unwrap_or_default().to_string();
        let name = if token.starts_with("review:partial:") {
            format!("review partial round {}", last())
        } else if token.starts_with("review:absent:") {
            format!("review absent round {}", last())
        } else if token.starts_with("unreported:") {
            "worker report".to_string()
        } else if named {
            continue;
        } else {
            named = true;
            "verify".to_string()
        };
        if !out.is_empty() {
            out.push_str(", ");
        }
        out.push_str(&name);
    }
    out
}

fn report(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let mut metrics = false;
    for arg in &rest {
        match arg.as_str() {
            "--metrics" => metrics = true,
            _ => fail!("unknown argument: {arg} (usage: dstack report [--run <id>|--quick <slug>] [--metrics])"),
        }
    }
    let dir = target.dir.clone();
    let request = dir.join("request.md");
    if !request.is_file() {
        fail!("no request.md in {} — nothing to report (dstack request new --type <work_type>)", dir.display())
    }

    // D-07: `check decisions` is a run-level check. When it fails, every R carries the failure,
    // because an undecided question is not a property of one requirement.
    let flag = match target.kind {
        TargetKind::Quick => "--quick",
        TargetKind::Run => "--run",
    };
    let called = ctx.call("check decisions", &[flag.to_string(), target.id.clone()]);
    let merged = format!("{}{}", called.stdout, called.stderr);
    let decided = called.code;
    let dec_first = match decided {
        0 => String::new(),
        _ => merged.trim_end_matches('\n').lines().next_back().unwrap_or("").to_string(),
    };

    let doc = RequestDoc::load(&request)?;
    let violations = policy_violations(&roots.store, &doc);
    let states = states::of(&dir, &roots.main_root, target.kind, !violations.is_empty())?;
    // Containment is run-level like the decision check: an unrebased Goal branch means no row of
    // it is proven on top of the base, so it lands in every row's status rather than nowhere.
    let (branch_out, contained) = match target.kind {
        TargetKind::Run => branch_line(&dir, &roots.wt_root)?,
        TargetKind::Quick => (String::new(), true),
    };

    let plan = match plan::exists(&dir) {
        true => Some(plan::load(&dir)?),
        false => None,
    };
    let ledger = cases::rows(&dir)?;
    // Both ledgers are read once, before the first row is printed: a table half printed over an
    // unreadable accepts.tsv is output the shell never produces either (D-12).
    let accepts = cases::accepts_rows(&dir)?;
    let accepted = |r: &str| {
        accepts
            .iter()
            .find(|row| row.r == r)
            .map(|row| row.why.clone())
            .filter(|why| !why.is_empty())
    };
    let (mut met, mut unmet, mut abst, mut blk) = (0, 0, 0, 0);
    let (mut skip, mut defer, mut wdrawn, mut total) = (0, 0, 0, 0);
    ctx.out.say("| R | text | tasks | evidence | status |");
    ctx.out.say("|---|---|---|---|---|");
    for row in doc.rows() {
        total += 1;
        let id = &row.id;
        let marker = |key: &str| row.marker(key).filter(|value| !value.is_empty());
        let cases_of = |status: &str| ledger.iter().find(|c| c.r == *id && c.status == status);
        let tasks = match target.kind {
            TargetKind::Quick => "n/a".to_string(),
            TargetKind::Run => dashed(tasks_covering_of(plan.as_ref(), id)),
        };
        let evidence = dashed(
            ledger
                .iter()
                .filter(|c| c.r == *id && !matches!(c.status.as_str(), "open" | "retired" | "unreported"))
                .filter(|c| c.artifact != "-" && !c.artifact.is_empty())
                .map(|c| c.artifact.clone())
                .collect(),
        );
        let status = if marker("withdrawn").is_some() {
            wdrawn += 1;
            "WITHDRAWN".to_string()
        } else if marker("deferred").is_some() {
            defer += 1;
            "DEFERRED".to_string()
        } else if marker("status").as_deref() == Some("pending-approval") {
            unmet += 1;
            "UNMET: pending-approval".to_string()
        } else if let Some(into) = marker("superseded-by") {
            skip += 1;
            format!("SKIPPED: superseded-by {into}")
        } else if let Some(case) = cases_of("skipped") {
            skip += 1;
            format!("SKIPPED: {}", undash(&case.note))
        } else {
            let (_, _, missing) = coverage_of(plan.as_ref(), &ledger, target.kind, id);
            let state = states.iter().find(|s| s.r == *id);
            let mut names: Vec<String> = Vec::new();
            if missing != "ok" {
                names.push("coverage".to_string());
            }
            // A live row that verify did not judge is not silently MET (§3-2: no quiet pass).
            if state.is_none() {
                names.push("verify (no state computed)".to_string());
            }
            let mut tokens = match state.map(|s| s.state) {
                Some("FAIL") => state.map(|s| s.reasons.clone()).unwrap_or_default(),
                _ => String::new(),
            };
            if !contained {
                tokens = match tokens.is_empty() {
                    true => "branch-containment".to_string(),
                    false => format!("{tokens};branch-containment"),
                };
            }
            if !tokens.is_empty() {
                let named = reason_names(&tokens);
                if !named.is_empty() {
                    names.push(named);
                }
            }
            if decided != 0 {
                names.push("check decisions".to_string());
            }
            // An accepted ABSTAIN still counts as ABSTAIN in the table: R79 wants the reason on
            // the page, it just no longer blocks the exit code.
            let why = accepted(id);
            let pretty = || reasons_pretty(id, &state.map(|s| s.reasons.clone()).unwrap_or_default());
            if !names.is_empty() {
                unmet += 1;
                format!("UNMET: {}", names.join(", "))
            } else if state.map(|s| s.state) == Some("BLOCKED") {
                blk += 1;
                match why {
                    Some(why) => format!("BLOCKED (accepted: {why})"),
                    None => format!("BLOCKED: {}", pretty()),
                }
            } else if state.map(|s| s.state) == Some("ABSTAIN") {
                abst += 1;
                match why {
                    Some(why) => format!("ABSTAIN (accepted: {why})"),
                    None => format!("ABSTAIN: {}", pretty()),
                }
            } else {
                met += 1;
                "MET".to_string()
            }
        };
        say!(ctx, "| {id} | {} | {tasks} | {evidence} | {status} |", row.text.replace('|', "\\|"));
    }

    ctx.out.say("");
    say!(ctx, "report: {} {} — {}", kind_word(target.kind), target.id, dir.display());
    if decided != 0 {
        let first = match dec_first.is_empty() {
            true => "no output".to_string(),
            false => dec_first,
        };
        say!(ctx, "check decisions: FAILED (exit {decided}) — {first}; every R above reads UNMET with it (D-07)");
    }
    if let Some(first) = violations.first() {
        say!(ctx, "policy ceiling (R75): REFUSED — {first}");
    }
    if !contained {
        say!(ctx, "{branch_out} (R38); every R above reads UNMET with verify");
    }
    say!(ctx, "MET {met}, UNMET {unmet}, ABSTAIN {abst}, BLOCKED {blk}, SKIPPED {skip}, DEFERRED {defer}, WITHDRAWN {wdrawn} (rows {total})");
    let base = total - wdrawn - defer;
    let rate = match base > 0 {
        true => format!("{:.1}", (met as f64 * 100.0) / base as f64),
        false => "0.0".to_string(),
    };
    say!(ctx, "requirement coverage rate = MET / (total - WITHDRAWN - DEFERRED) = {met}/{base} = {rate}%");

    let mut code = if unmet > 0 {
        1
    } else if abst > 0 || blk > 0 {
        // Only rows nobody accepted keep the report at 2.
        let unaccepted = doc.live_ids().iter().any(|r| {
            matches!(states.iter().find(|s| s.r == *r).map(|s| s.state), Some("ABSTAIN") | Some("BLOCKED"))
                && accepted(r).is_none()
        });
        match unaccepted {
            true => 2,
            false => 0,
        }
    } else {
        0
    };
    if metrics {
        ctx.out.say("");
        if !run_metrics(ctx, &roots, &target, &format!("{met}/{base} ({rate}%)"))? {
            code = 1;
        }
    }
    match code {
        0 => Ok(()),
        code => Err(Error::Exit(code)),
    }
}

/// The joined cell of a column that reads `-` when it is empty.
fn dashed(values: Vec<String>) -> String {
    match values.is_empty() {
        true => "-".to_string(),
        false => values.join(","),
    }
}

fn tasks_covering_of(plan: Option<&PlanDoc>, r: &str) -> Vec<String> {
    plan.map(|doc| tasks_covering(doc, r)).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_reason_tokens_become_the_check_names_r79_lists() {
        assert_eq!(reason_names("evidence:e2e=cli;evidence:unit_tests"), "verify");
        assert_eq!(reason_names("unreported:c1"), "worker report");
        assert_eq!(
            reason_names("sha256:c1;review:partial:002;unreported:c1;review:absent:003"),
            "verify, review partial round 002, worker report, review absent round 003"
        );
        assert_eq!(reason_names(""), "verify");
    }
}
