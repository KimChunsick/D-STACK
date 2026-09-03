// verbs/verify/mod.rs
// dstack verify: the printed sections, --accept-abstain and the exit-code contract of the verdict
// states::of computes (0 everything proven, 1 a failure, 2 nobody could decide).

use std::path::Path;

use crate::core::args::opt;
use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::fsx::{atomic_write, utc_now};
use crate::core::target::{resolve_target, TargetKind};
use crate::core::tools::policy_get;
use crate::core::verb::Verb;
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};
use crate::store::cases::{self, ACCEPTS_HEADER};
use crate::store::request::RequestDoc;

use states::{branch_line, kind_word, policy_violations, KINDS_E2E};

pub mod states;

/// say(): one stdout line.
macro_rules! say { ($ctx:expr, $($line:tt)*) => { $ctx.out.say(&format!($($line)*)) }; }

/// fail(): the checked condition that did not hold, on stderr, exit 1.
macro_rules! fail { ($($m:tt)*) => { return Err(Error::failed(format!($($m)*))) }; }

struct Verify;

impl Verb for Verify {
    fn name(&self) -> &'static str {
        "verify"
    }
    fn run(&self, ctx: &mut Context, args: &[String]) -> Result<()> {
        verify(ctx, args)
    }
}

pub fn verbs() -> Vec<Box<dyn Verb>> {
    vec![Box::new(Verify)]
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(VerifySelftest)]
}

/// `$(policy_get <key> || echo -)`: a store without a PROJECT.md prints the dash the shell's `||`
/// branch prints, while a policy block without the key prints the empty value awk found.
fn policy_shown(store: &Path, key: &str) -> String {
    match store.join("project/PROJECT.md").is_file() {
        false => "-".to_string(),
        true => policy_get(store, key).unwrap_or_default(),
    }
}

fn verify(ctx: &mut Context, args: &[String]) -> Result<()> {
    let roots = ctx.roots()?;
    roots.require_store()?;
    let (target, rest) = resolve_target(ctx, args)?;
    let (mut accept, mut why) = (String::new(), String::new());
    let mut i = 0;
    while i < rest.len() {
        let next = rest.get(i + 1).map(String::as_str);
        if let Some((value, eaten)) = opt(&rest[i], next, "accept-abstain")? {
            accept = value;
            i += eaten;
        } else if let Some((value, eaten)) = opt(&rest[i], next, "why")? {
            why = value;
            i += eaten;
        } else {
            fail!("unknown argument: {} (usage: dstack verify [--run <id>|--quick <slug>] [--accept-abstain R01,R02 --why \"<reason>\"])", rest[i])
        }
    }
    let dir = &target.dir;
    let request = dir.join("request.md");
    if !request.is_file() {
        fail!("no request.md in {} — nothing to verify (dstack request new --type <work_type>)", dir.display())
    }
    let doc = RequestDoc::load(&request)?;
    let field = |key: &str| doc.field(key).unwrap_or_default();
    say!(ctx, "verify: {} {} — {}", kind_word(target.kind), target.id, dir.display());
    say!(ctx, "  fields: e2e={} unit_tests={} visual={} review={}",
        field("e2e"), field("unit_tests"), field("visual"), field("review"));

    // (1) policy ceiling: a request above the ceiling makes every R unverifiable, so it is folded
    // into the per-R state instead of being a separate silent failure.
    let violations = policy_violations(&roots.store, &doc);
    if violations.is_empty() {
        say!(ctx, "policy ceiling (R75): ok (surfaces={}, e2e_evidence={}, visual_diff={})",
            policy_shown(&roots.store, "surfaces"),
            policy_shown(&roots.store, "e2e_evidence"),
            policy_shown(&roots.store, "visual_diff"));
    } else {
        ctx.out.say("policy ceiling (R75): REFUSED");
        for line in &violations {
            say!(ctx, "  {line}");
        }
        say!(ctx, "  policy: {}/project/PROJECT.md", roots.store.display());
    }

    // (2)+(3) per-R evidence and the sha256 recheck.
    let states = states::of(dir, &roots.main_root, target.kind, !violations.is_empty())?;
    // --accept-abstain first, so the summary already reflects what the owner just accepted.
    let mut accepted = 0;
    if !accept.is_empty() {
        if why.is_empty() {
            fail!("--accept-abstain needs --why \"<reason>\" — R79 puts the reason in the report")
        }
        let file = dir.join("accepts.tsv");
        if !file.is_file() {
            atomic_write(&file, format!("{ACCEPTS_HEADER}\n").as_bytes())
                .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", file.display())))?;
        }
        for r in accept.replace(',', " ").split_whitespace() {
            let state = match states.iter().find(|row| row.r == r) {
                Some(row) => row.state,
                None => fail!("{r} is not a live requirement of {} — nothing to accept", request.display()),
            };
            if state != "ABSTAIN" && state != "BLOCKED" {
                fail!("{r} is {state}, not ABSTAIN or BLOCKED — --accept-abstain only takes what evidence could not decide")
            }
            cases::accepts_append(dir, r, &why, &utc_now())?;
            say!(ctx, "accepted {r} ({state}): {why}");
            accepted += 1;
        }
        say!(ctx, "accepts: {accepted} written to {}", file.display());
    }

    let (mut ok, mut failed, mut abstain, mut blocked) = (0, 0, 0, 0);
    for row in &states {
        let excused = cases::accepts_why(dir, &row.r)?.filter(|why| !why.is_empty());
        match (row.state, excused) {
            ("PASS", _) => {
                ok += 1;
                say!(ctx, "{} ok", row.r);
            }
            ("FAIL", _) => {
                failed += 1;
                say!(ctx, "{} FAIL: {}", row.r, reasons_pretty(&row.r, &row.reasons));
            }
            (state, Some(why)) => {
                ok += 1;
                say!(ctx, "{} {state} accepted: {why}", row.r);
            }
            ("BLOCKED", None) => {
                blocked += 1;
                say!(ctx, "{} BLOCKED: {}", row.r, reasons_pretty(&row.r, &row.reasons));
            }
            (state, None) => {
                abstain += 1;
                say!(ctx, "{} {state}: {}", row.r, reasons_pretty(&row.r, &row.reasons));
            }
        }
    }

    // (4) containment is a property of the run, not of an R (R38): quick tasks have no branch.
    let mut contained = true;
    if target.kind == TargetKind::Run {
        let (line, held) = branch_line(dir, &roots.wt_root)?;
        ctx.out.say(&line);
        contained = held;
    }
    let code = if failed > 0 || !contained {
        1
    } else if abstain > 0 || blocked > 0 {
        2
    } else {
        0
    };
    say!(ctx, "verify: checked {} R, ok {ok}, failed {failed}, abstain {abstain}, blocked {blocked}, accepted {accepted} → exit {code}", states.len());
    if code == 2 {
        ctx.out.say("  accept them one at a time: dstack verify --accept-abstain <R> --why \"<reason>\" (R79)");
    }
    match code {
        0 => Ok(()),
        code => Err(Error::Exit(code)),
    }
}

/// Reason tokens are compact so report can map them; humans get the long form here.
pub fn reasons_pretty(r: &str, reasons: &str) -> String {
    let mut out = String::new();
    // Split on ";" and keep every token: a reason ("no sealed review round") carries spaces, so
    // word splitting would turn one reason into four.
    for token in reasons.split(';') {
        let tail = |prefix: &str| token[prefix.len()..].to_string();
        let last = || token.rsplit(':').next().unwrap_or_default().to_string();
        let long = if token == "policy-ceiling" {
            "request exceeds the repository policy ceiling".to_string()
        } else if let Some(e2e) = token.strip_prefix("evidence:e2e=") {
            format!("no {} evidence row (e2e: {e2e})", KINDS_E2E.join("|"))
        } else if token == "evidence:unit_tests" {
            "no test evidence row (unit_tests: on)".to_string()
        } else if let Some(visual) = token.strip_prefix("evidence:visual=") {
            format!("no visual evidence row (visual: {visual})")
        } else if token.starts_with("sha256:") {
            format!("sha256 mismatch on case {} — the artifact changed after it was recorded", tail("sha256:"))
        } else if token.starts_with("artifact-missing:") {
            format!("artifact of case {} is gone", tail("artifact-missing:"))
        } else if token.starts_with("unreported:") {
            format!("worker did not report case {} — fill it: dstack evidence add --r {r} --case <id> …", tail("unreported:"))
        } else if token.starts_with("review:partial:") {
            format!("sealed review round {} says partial", last())
        } else if token.starts_with("review:absent:") {
            format!("sealed review round {} says absent", last())
        } else if token.starts_with("abstain:") || token.starts_with("blocked:") {
            token.split_once(':').map(|(_, rest)| rest.to_string()).unwrap_or_default()
        } else {
            token.to_string()
        };
        if !out.is_empty() {
            out.push_str(", ");
        }
        out.push_str(&long);
    }
    match out.is_empty() {
        true => "no reason recorded".to_string(),
        false => out,
    }
}

/// A file of the fixture sandbox: the scratch directory belongs to this process, so a write that
/// fails there is the runner failing to run, never a verdict.
fn write_file(path: &Path, text: &str) -> Result<()> {
    std::fs::write(path, text)
        .map_err(|e| Error::cannot_decide(format!("selftest: cannot write {}: {e}", path.display())))
}

/// verify: the fixture is a request.md; `<!-- selftest-tamper: yes -->` makes the driver edit a
/// recorded artifact after recording it, which is exactly the hand-edit the sha256 recheck owes
/// us a failure for.
struct VerifySelftest;

impl Selftest for VerifySelftest {
    fn checker(&self) -> &'static str {
        "verify"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let run_dir = sandbox.run_dir()?;
        let request = run_dir.join("request.md");
        std::fs::copy(fixture, &request).map_err(|e| {
            Error::cannot_decide(format!("selftest: cannot stage {}: {e}", fixture.display()))
        })?;
        sandbox.approve(&run_dir)?;
        let _ = sandbox.dsx(ctx, &["cases", "sync"])?;
        let live = RequestDoc::load(&request)?.live_ids();
        let mut first = None;
        for r in &live {
            let artifact = sandbox.artifact(&format!("{r}.txt"), &format!("{r} verified: checked 1, missing 0"))?;
            if first.is_none() {
                first = Some(artifact.clone());
            }
            let _ = sandbox.dsx(ctx, &["evidence", "add", "--r", r, "--case", "c1", "--kind", "cli",
                "--artifact", &artifact.to_string_lossy(), "--produced-by", "selftest"])?;
        }
        if Sandbox::directive(fixture, "tamper").as_deref() == Some("yes") {
            if let Some(path) = &first {
                let text = std::fs::read_to_string(path).unwrap_or_default();
                write_file(path, &format!("{text}edited by hand after recording\n"))?;
            }
        }
        // The per-R review verdict is part of MET for every Goal run (R69/R79), so the sandbox
        // gets one sealed round that covers every live R; the fixtures then exercise the policy,
        // the evidence and the sha.
        let mut round = String::from("| R | verdict | evidence in the diff |\n|---|---|---|\n");
        for r in &live {
            round.push_str(&format!("| {r} | covered | selftest |\n"));
        }
        round.push_str("\nVERDICT: approve\n");
        let sealed = sandbox.dir.join("review-out.md");
        write_file(&sealed, &round)?;
        // review seal is ported by P11; until it lands the round is copied where seal puts it.
        let (code, _) = sandbox.dsx(ctx, &["review", "seal", "--from", &sealed.to_string_lossy(),
            "--scope", "plan", "--id", "P0"])?;
        if code != 0 {
            write_file(&run_dir.join("review/codex-review-001.md"), &round)?;
        }
        // The shell's selftest reads every non-zero exit as a rejection; 2 is the runner failing
        // to decide, and reporting that as a rejection would let a broken checker read as working.
        match sandbox.dsx(ctx, &["verify"])?.0 {
            0 => Ok(Verdict::Pass),
            1 => Ok(Verdict::Reject),
            code => Err(Error::cannot_decide(format!("selftest: dstack verify exited {code} instead of deciding"))),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_a_reason_list_reads_as_the_shell_prints_it() {
        assert_eq!(reasons_pretty("R01", ""), "no reason recorded");
        assert_eq!(
            reasons_pretty("R01", "policy-ceiling;evidence:e2e=cli;review:partial:003"),
            "request exceeds the repository policy ceiling, no cli|capture|transcript evidence row (e2e: cli), sealed review round 003 says partial"
        );
        assert_eq!(
            reasons_pretty("R01", "abstain:review closed after round 001: the reviewer stopped"),
            "review closed after round 001: the reviewer stopped"
        );
    }
}
