// verbs/decision.rs
// dstack decision add|list and check decisions: the D ledger (R51, R55) and its coverage check.
//
// A decision that nothing implements is a decision that was never made: the whole point of the
// ledger is that `check decisions` can prove each row reached the code or the evidence.

use std::path::{Path, PathBuf};

use crate::core::args::{is_option, opt, unknown_option};
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::target::resolve_target;
use crate::core::verb::Verb;
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};
use crate::store::cases::{self, CaseRow};
use crate::store::plan;
use crate::store::plan_graph::tasks_covering;
use crate::store::request::RequestDoc;
use crate::store::tables::{
    d_append, d_design_reason, d_design_rows, d_next_id, decisions, q_text_ok,
};

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(Error::failed(format!($($m)*))) }; }

/// The three roster entries this file answers; the struct carries nothing but its name.
macro_rules! decision_verb {
    ($handler:ident, $entry:literal, $body:ident) => {
        struct $handler;
        impl Verb for $handler {
            fn name(&self) -> &'static str {
                $entry
            }
            fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
                $body(ctx, args)
            }
        }
    };
}

decision_verb!(DecisionAdd, "decision add", add);
decision_verb!(DecisionList, "decision list", list);
decision_verb!(CheckDecisions, "check decisions", check);

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![
        Box::new(DecisionAdd),
        Box::new(DecisionList),
        Box::new(CheckDecisions),
    ]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(CheckDecisions)]
}

fn add(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut text, mut affects, mut reason) = (String::new(), String::new(), String::new());
    let (mut design, mut assumed) = (false, false);
    let mut i = 0;
    while i < rest.len() {
        let arg = rest[i].clone();
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(&arg, next, "affects")? {
            affects = value;
            i += eaten;
        } else if arg == "--design" {
            // The one option the shell reads without the two-word rule: a `--design` with no
            // operand left is an empty reason, warned about here and refused by the check from
            // the second design row on, not the end of the run a missing operand usually is.
            design = true;
            reason = next.unwrap_or_default().to_string();
            i += 1 + usize::from(next.is_some());
        } else if let Some((value, eaten)) = opt(&arg, next, "design")? {
            design = true;
            reason = value;
            i += eaten;
        } else if arg == "--assumed" {
            assumed = true;
            i += 1;
        } else if is_option(&arg) {
            return Err(unknown_option(&arg));
        } else if text.is_empty() {
            text = arg;
            i += 1;
        } else {
            fail!("unexpected argument: {arg}")
        }
    }
    if text.is_empty() {
        fail!("usage: dstack decision add \"<decision>\" --affects R01,R02|design [--design \"<reason>\"]")
    }
    if affects.is_empty() {
        fail!("--affects is required (R ids, or 'design' for a design decision)")
    }
    q_text_ok("decision", &text)?;
    q_text_ok("--affects", &affects)?;
    if !reason.is_empty() {
        q_text_ok("--design reason", &reason)?;
    }

    let file = dec_file(&target.dir);
    let id = d_next_id(&file, design)?;
    if design {
        // design.md §4.4 fixes the cell as "design round N: reason"; the decision itself follows
        // the same " — " separator the request rows use, so one parser reads both.
        let round = id.trim_start_matches("D-DESIGN-").trim_start_matches('0');
        text = format!("design round {round}: {reason} — {text}");
    }
    // --assumed records a default an implementer adopted without an interview question (the R51
    // path mints an R row through ask assume; this one only makes the assumption visible).
    let status = match assumed {
        true => "assumed",
        false => "answered",
    };
    d_append(&file, &id, &text, &affects, status)?;
    say!(ctx, "decisions: {}", file.display());
    say!(ctx, "  {id} | {text} | {affects} | {status}");
    let (rows, design_rows) = (decisions(&file)?.len(), d_design_rows(&file)?.len());
    say!(ctx, "  rows {rows}, design rows {design_rows}");
    if design && reason.is_empty() {
        ctx.out.warn(&format!(
            "design row {id} has an empty reason; check decisions rejects that from the second design row on (R55)"));
    }
    Ok(())
}

fn list(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    let file = dec_file(&target.dir);
    let rows = decisions(&file)?;
    say!(ctx, "decisions: {}", file.display());
    ctx.out.say("D | Decision | Affects | Status");
    let with = |status: &str| rows.iter().filter(|row| row.status == status).count();
    for row in &rows {
        say!(ctx, "{} | {} | {} | {}", row.id, row.text, row.affects, row.status);
    }
    say!(
        ctx,
        "rows {}, answered {}, assumed {}, design rows {}",
        rows.len(),
        with("answered"),
        with("assumed"),
        rows.iter()
            .filter(|row| row.id.starts_with("D-DESIGN-"))
            .count()
    );
    Ok(())
}

/// A D row is covered when something in the machine state acts on one of the R ids it affects: a
/// task in plan.json, or an evidence row that reached a terminal state. A row that affects only
/// "design" is covered by the existence of a design round, because that is the artifact it names.
///
/// A row whose every R id is withdrawn, deferred or superseded is moot: those rows take no task
/// and no evidence by design (R79, R103), so the decision has nothing left to reach. One live R
/// id that is neither tasked nor evidenced still leaves the row UNCOVERED.
fn check(ctx: &mut Context, args: &[String]) -> Result<()> {
    let (target, _rest) = resolve_target(ctx, args)?;
    let (dir, file) = (target.dir.clone(), dec_file(&target.dir));
    say!(ctx, "check decisions: {}", file.display());
    if !file.is_file() {
        say!(ctx, "  no decisions.md in {}", dir.display());
        ctx.out
            .say("  rows 0, covered 0, uncovered 0, design rows 0");
        return Ok(());
    }
    let has_design = !d_design_rows(&file)?.is_empty();
    // Only a missing plan.json is "no plan" (the shell's `plan_exists || return 0`). One that is
    // there but cannot be read is a cannot decide: the shell lets jq print its parse error and
    // reads the run as having no tasks at all, which turns a broken file into rows that report
    // UNCOVERED. The port refuses instead, with its own wording rather than jq's.
    let doc = match plan::exists(&dir) {
        true => Some(plan::load(&dir)?),
        false => None,
    };
    let ledger = cases::rows(&dir)?;
    // A run without a request.md has nothing marked; the moot rule then never fires. A request
    // that is there and cannot be read is a cannot decide (D-12), not a run that marked nothing.
    let request = match dir.join("request.md").is_file() {
        true => Some(RequestDoc::load(&dir.join("request.md"))?),
        false => None,
    };
    let (mut total, mut covered, mut uncovered) = (0, 0, 0);
    for row in decisions(&file)? {
        total += 1;
        let (mut why, mut moot, mut live_r) = (String::new(), String::new(), false);
        for token in affects_tokens(&row.affects) {
            if is_r_token(token) {
                if let Some(mark) = request.as_ref().and_then(|req| r_mark(req, token)) {
                    let sep = if moot.is_empty() { "" } else { ", " };
                    moot.push_str(&format!("{sep}{token} ({mark})"));
                    continue;
                }
                live_r = true;
                let tasks = doc
                    .as_ref()
                    .map(|d| tasks_covering(d, token))
                    .unwrap_or_default();
                if !tasks.is_empty() {
                    why = format!("task {} covers {token}", tasks.join(","));
                    break;
                }
                if let Some(found) = recorded(&ledger, token) {
                    why = format!("evidence {found} on {token}");
                    break;
                }
            } else if token == "design" && has_design {
                why = "a design round is recorded".to_string();
                break;
            }
        }
        if why.is_empty() && !moot.is_empty() && !live_r {
            why = format!("moot — every affected row is marked: {moot}");
        }
        if why.is_empty() {
            uncovered += 1;
            let (id, affects) = (&row.id, &row.affects);
            say!(ctx, "  {id:<14} UNCOVERED — no task and no evidence on: {affects}");
        } else {
            covered += 1;
            say!(ctx, "  {:<14} covered   — {why}", row.id);
        }
    }

    // R55: one design round may be self-evident; from the second on, "why another round" must be
    // written down, or design review becomes an unbounded loop nobody can audit.
    let (mut design_rows, mut missing) = (0, 0);
    for row in d_design_rows(&file)? {
        design_rows += 1;
        if design_rows >= 2 && d_design_reason(&row.text).is_empty() {
            missing += 1;
            say!(ctx, "  {}: design round {design_rows} has an empty reason (R55 requires one from round 2)", row.id);
        }
    }
    say!(ctx, "  rows {total}, covered {covered}, uncovered {uncovered}, design rows {design_rows}, design rows missing a reason {missing}");
    if uncovered != 0 {
        fail!("{uncovered} decision row(s) reach no task and no evidence (add a task with --covers, or record evidence)")
    }
    if missing != 0 {
        fail!("{missing} design row(s) carry no reason (dstack decision add \"…\" --affects design --design \"<why another round>\")")
    }
    Ok(())
}

/// The first evidence row of an R that reached a terminal state, as `case(status)`. skipped,
/// open, unreported and retired are not evidence. The ledger is read once per run of the check,
/// not once per R id a decision row names.
fn recorded(rows: &[CaseRow], r: &str) -> Option<String> {
    rows.iter()
        .find(|row| row.r == r && matches!(row.status.as_str(), "met" | "abstain" | "blocked"))
        .map(|row| format!("{}({})", row.case_id, row.status))
}

/// The shell's `for tok in $(printf '%s' "$affects" | tr ',' ' ')`: tr turns the commas into
/// spaces and the default IFS then splits on ASCII space, tab and newline only. A non-breaking
/// space belongs to the token, so Rust's Unicode-aware split_whitespace would read one name as
/// two.
fn affects_tokens(affects: &str) -> Vec<&str> {
    affects
        .split([',', ' ', '\t', '\n'])
        .filter(|token| !token.is_empty())
        .collect()
}

/// The mark that takes an R id out of the live set, read from its request row in the order the
/// shell's case arms are written; an unknown id and an unmarked row are both None.
fn r_mark(request: &RequestDoc, r: &str) -> Option<&'static str> {
    let markers = format!(";{};", request.row(r)?.markers_string());
    [(";withdrawn=", "withdrawn"), (";deferred=", "deferred"), (";superseded-by=", "superseded")]
        .into_iter()
        .find(|(needle, _)| markers.contains(needle))
        .map(|(_, mark)| mark)
}

/// The shell's `R[0-9]*` glob: an R, a digit, then anything.
fn is_r_token(token: &str) -> bool {
    let mut chars = token.chars();
    chars.next() == Some('R') && chars.next().is_some_and(|c| c.is_ascii_digit())
}

fn dec_file(dir: &Path) -> PathBuf {
    dir.join("decisions.md")
}

// ── self-test (R100) ────────────────────────────────────────────────────────────────────
// The sandbox always plans one task covering R01; the fixtures differ only in which R their
// decision affects, so no branch on the fixture name is needed to tell covered from uncovered.
// The request is what the moot rule reads, and a fixture asks for a mark with the directive
// `<!-- selftest-withdraw: R02 -->`.
impl Selftest for CheckDecisions {
    fn checker(&self) -> &'static str {
        "check-decisions"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let run_dir = sandbox.run_dir()?;
        sandbox.write_request(&run_dir)?;
        sandbox.write_plan(&run_dir, &["R01"])?;
        std::fs::copy(fixture, run_dir.join("decisions.md")).map_err(|e| {
            Error::cannot_decide(format!("sandbox: cannot copy {}: {e}", fixture.display()))
        })?;
        if let Some(id) = Sandbox::directive(fixture, "withdraw") {
            sandbox.dsx(ctx, &["req", "withdraw", &id, "--why", "selftest fixture"])?;
        }
        let (code, output) = sandbox.dsx(ctx, &["check", "decisions"])?;
        verdict(code, &output)
    }
}

/// The runner's contract (selftest/mod.rs): 0 is a pass and 1 is the rejection a bad fixture has
/// to provoke. Anything else — the checker could not decide, or the sandbox broke — is a failure
/// of the checker itself, so it leaves as an Err instead of being counted as a rejection.
fn verdict(code: i32, output: &str) -> Result<Verdict> {
    match code {
        0 => Ok(Verdict::Pass),
        1 => Ok(Verdict::Reject),
        other => Err(Error::cannot_decide(format!(
            "check decisions exited {other}: {}",
            output.trim_end()
        ))),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r05_only_zero_and_one_are_verdicts() {
        assert_eq!(verdict(0, "").expect("a pass"), Verdict::Pass);
        assert_eq!(
            verdict(1, "uncovered").expect("a rejection"),
            Verdict::Reject
        );
        let error = verdict(2, "dstack: run not found: x\n").expect_err("a checker failure");
        assert_eq!(error.code(), 2);
        assert_eq!(
            error.message(),
            "check decisions exited 2: dstack: run not found: x"
        );
    }
}
