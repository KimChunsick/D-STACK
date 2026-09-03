// verbs/review/selftests.rs
// The fixture checkers of the noun: check review-bundle and review close (R100).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

pub fn all() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(CheckReviewBundle), Box::new(ReviewClose)]
}

/// The sandbox gets a hand-written plan.json with the exact shape of design §4.5, so the checker
/// has a real (b) to count; plan add belongs to another module and a fixture must not wait on it.
const PLAN: &str = "{ \"v\": 2,
  \"milestones\": [ {\"id\":\"M1\",\"slug\":\"first\",\"order\":1} ],
  \"plans\": [ {\"id\":\"P1\",\"milestone\":\"M1\",\"slug\":\"bundle-check\",\"files\":[\"a/b.sh\"],\"deps\":[],
              \"status\":\"in-progress\",\"worktree\":\"\",\"started_at\":\"\",\"done_at\":\"\",
              \"tasks\":[ {\"id\":\"T1\",\"slug\":\"two-covers\",\"covers\":[\"R01\",\"R02\"],\"files\":[\"a/b.sh\"],
                         \"deps\":[],\"commit\":\"\",\"done_at\":\"\"} ] } ] }
";

/// The fixture is a whole bundle text: the checker has to read it the way it reads what `review`
/// just wrote, so the fixture is copied into the sandbox and judged as a path.
struct CheckReviewBundle;

impl Selftest for CheckReviewBundle {
    fn checker(&self) -> &'static str {
        "check-review-bundle"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let run_dir = sandbox.run_dir()?;
        write(&run_dir.join("plan.json"), PLAN)?;
        copy(fixture, &sandbox.dir.join("bundle.txt"))?;
        let (code, _) = sandbox.dsx(ctx, &["check", "review-bundle", "bundle.txt"])?;
        verdict(code, "dstack check review-bundle")
    }
}

/// The quick task the shell checker builds with `dstack quick new qq --type cli` followed by one
/// sed on the review field: quick new belongs to another module and a fixture must not wait on
/// it, so the file it writes is written here, with review already on so a close means something.
const QUICK_REQUEST: &str = r#"---
work_type: cli
route: quick
external_research: none
risk_axes: none
design_review: skip
review: on
codex_effort: medium
e2e: none
unit_tests: off
visual: none
korean_polish: on
---
# qq

One paragraph: which command changes, who runs it, and what it prints and exits with after this Goal.

## Requirements

<!-- Add rows with: dstack req add "<one line>" --accept "<criterion>". Never write a row by hand. -->
<!-- accept: is the captured output and exit code of a named command, never "the code was written". -->
<!-- design_review (R55) fixes the module boundaries: which file owns which verb, and what they share. -->
<!-- 12 live rows and 60 lines are the ceiling (R43): split a Milestone rather than growing this file. -->
"#;

/// Approving a `review: on` request needs codex among the goal-closing tools, and no sandbox may
/// depend on it being installed. The shell appends a `true` probe; the port runs only the two
/// probe forms deps.tsv fixes (R01), so the probe that always succeeds is spelled as one of them.
const CODEX_DEP: &str = "codex\tcommand -v git\t-\t-\tyes\tgoal-closing\treview=on\t\n";

/// What `verify` prints for the row the close abstained, reason and round included: exit 2 alone
/// says nothing, since every unrelated abstain leaves the same code behind.
const CLOSED: &str = "R01 ABSTAIN: review closed after round 000: fixture";

/// The fixture's first line is the --scope to close with: "quick" is the one that fits a quick
/// target and anything else has to be refused there.
struct ReviewClose;

impl Selftest for ReviewClose {
    fn checker(&self) -> &'static str {
        "review-close"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let scope = scope_of(fixture)?;
        let sandbox = Sandbox::new(ctx)?;
        let quick = sandbox.dir.join(".dstack/quick/qq");
        std::fs::create_dir_all(&quick).map_err(|e| {
            Error::cannot_decide(format!("selftest: cannot create {}: {e}", quick.display()))
        })?;
        write(&quick.join("request.md"), QUICK_REQUEST)?;
        allow_codex(&sandbox)?;
        setup(
            &sandbox,
            ctx,
            &["req", "add", "--quick", "qq", "one", "--accept", "a"],
        )?;
        setup(&sandbox, ctx, &["request", "approve", "--quick", "qq"])?;
        let (code, _) = sandbox.dsx(
            ctx,
            &[
                "review", "close", "--quick", "qq", "--scope", &scope, "--id", "qq", "--why",
                "fixture",
            ],
        )?;
        if code != 0 {
            return verdict(code, "dstack review close");
        }
        // The close must turn the R into ABSTAIN, and only the line says which abstain it is.
        let (checked, output) = sandbox.dsx(ctx, &["verify", "--quick", "qq"])?;
        Ok(match checked {
            2 if output.lines().any(|line| line == CLOSED) => Verdict::Pass,
            0 | 1 | 2 => Verdict::Reject,
            other => {
                return Err(Error::cannot_decide(format!(
                    "selftest: dstack verify exited {other} instead of judging the closed row"
                )))
            }
        })
    }
}

/// The deps table the sandbox wrote, plus the codex row the fixture needs.
fn allow_codex(sandbox: &Sandbox) -> Result<()> {
    let path = sandbox.dir.join(".deps.tsv");
    let table = std::fs::read_to_string(&path).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot read {}: {e}", path.display()))
    })?;
    write(&path, &format!("{table}{CODEX_DEP}"))
}

/// A verb the fixture only builds its ground with: it has no verdict to give, so a non-zero exit
/// leaves the checker unable to decide rather than silently judging a half-built quick task.
fn setup(sandbox: &Sandbox, ctx: &Context, args: &[&str]) -> Result<()> {
    let (code, output) = sandbox.dsx(ctx, args)?;
    if code == 0 {
        return Ok(());
    }
    Err(Error::cannot_decide(format!(
        "selftest: dstack {} exited {code}: {output}",
        args.join(" ")
    )))
}

/// `head -1 "$fixture" | tr -d '[:space:]'`.
fn scope_of(fixture: &Path) -> Result<String> {
    let text = std::fs::read_to_string(fixture).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot read {}: {e}", fixture.display()))
    })?;
    Ok(text
        .lines()
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect())
}

/// The exit-code contract of a checker run: 0 is the fixture passing, 1 is the refusal it was
/// written to provoke, and 2 (or a spawn failure) is the runner not being able to decide at all —
/// never a rejection, because a broken checker would otherwise read as a working one.
fn verdict(code: i32, what: &str) -> Result<Verdict> {
    match code {
        0 => Ok(Verdict::Pass),
        1 => Ok(Verdict::Reject),
        other => Err(Error::cannot_decide(format!(
            "selftest: {what} exited {other} instead of deciding"
        ))),
    }
}

fn write(path: &Path, text: &str) -> Result<()> {
    std::fs::write(path, text).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot write {}: {e}", path.display()))
    })
}

fn copy(fixture: &Path, to: &Path) -> Result<()> {
    std::fs::copy(fixture, to).map(|_| ()).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot stage {}: {e}", fixture.display()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r05_a_checker_that_could_not_run_is_not_a_rejection() {
        assert_eq!(verdict(0, "x").expect("pass"), Verdict::Pass);
        assert_eq!(verdict(1, "x").expect("reject"), Verdict::Reject);
        let cannot = verdict(2, "dstack check review-bundle").expect_err("cannot decide");
        assert_eq!(cannot.code(), 2);
        assert_eq!(
            cannot.message(),
            "selftest: dstack check review-bundle exited 2 instead of deciding"
        );
    }
}
