// verbs/plan/selftests.rs
// The fixture checkers of the roadmap: plan add and task add, each in a scratch store (R100).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

pub fn all() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(PlanAdd), Box::new(TaskAdd)]
}

/// A request with two rows written by hand (_selftest_seed_request): `req add` and
/// `request approve` belong to other verbs, and live ids are read whether or not the file was
/// approved, so the fixture needs no approval flow.
const SEED_REQUEST: &str = "---
work_type: cli
route: new-goal
external_research: none
risk_axes: none
design_review: auto
review: on
codex_effort: high
e2e: cli
unit_tests: on
visual: none
korean_polish: on
---

# selftest request

- [ ] **R01** first requirement — accept: the command prints a count
- [ ] **R02** second requirement — accept: the command rejects a bad path — withdrawn: out of scope
";

/// The fixture holds one `plan add` (or `plan edit`) invocation per line, and is rejected when
/// ANY line is — which is what "these two lines together create a cycle" means.
pub struct PlanAdd;

impl Selftest for PlanAdd {
    fn checker(&self) -> &'static str {
        "plan-add"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        setup(&sandbox, ctx, &["milestone", "add", "core"])?;
        let mut verdict = Verdict::Pass;
        for line in read(fixture)?.lines() {
            let words = words(line);
            if words.is_empty() {
                continue;
            }
            let args: Vec<&str> = words.iter().map(String::as_str).collect();
            if judge(sandbox.dsx(ctx, &args)?.0, "plan add")? == Verdict::Reject {
                verdict = Verdict::Reject;
            }
        }
        Ok(verdict)
    }
}

/// The whole fixture is one `task add` invocation, judged against a plan that declares a/b.sh
/// and a request whose second row is withdrawn.
pub struct TaskAdd;

impl Selftest for TaskAdd {
    fn checker(&self) -> &'static str {
        "task-add"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let request = sandbox.run_dir()?.join("request.md");
        std::fs::write(&request, SEED_REQUEST).map_err(|e| {
            Error::cannot_decide(format!("cannot write {}: {e}", request.display()))
        })?;
        setup(&sandbox, ctx, &["milestone", "add", "core"])?;
        setup(
            &sandbox,
            ctx,
            &[
                "plan",
                "add",
                "first",
                "--milestone",
                "M1",
                "--files",
                "a/b.sh",
            ],
        )?;
        let words = words(&read(fixture)?);
        let args: Vec<&str> = words.iter().map(String::as_str).collect();
        judge(sandbox.dsx(ctx, &args)?.0, "task add")
    }
}

/// The exit-code contract of a checker run: 0 is the fixture passing, 1 is the refusal it was
/// written to provoke, and 2 (or a spawn failure) is the runner not being able to decide at all —
/// never a rejection.
fn judge(code: i32, what: &str) -> Result<Verdict> {
    match code {
        0 => Ok(Verdict::Pass),
        1 => Ok(Verdict::Reject),
        other => Err(Error::cannot_decide(format!(
            "{what} exited {other} in the fixture sandbox"
        ))),
    }
}

/// A command the scenario needs before the fixture is judged: anything but 0 means the checker
/// could not run, which is a runner failure and not a verdict.
fn setup(sandbox: &Sandbox, ctx: &mut Context, args: &[&str]) -> Result<()> {
    let (code, output) = sandbox.dsx(ctx, args)?;
    match code {
        0 => Ok(()),
        _ => Err(Error::cannot_decide(format!(
            "fixture sandbox: dstack {} exited {code}: {output}",
            args.join(" ")
        ))),
    }
}

fn read(fixture: &Path) -> Result<String> {
    std::fs::read_to_string(fixture)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", fixture.display())))
}

/// The words of one fixture line, as the shell's `eval "set -- $line"` splits them: whitespace
/// separates, and a quoted run keeps the spaces and the globs a plan path may not carry.
fn words(line: &str) -> Vec<String> {
    let (mut words, mut word, mut quote, mut open) = (Vec::new(), String::new(), '\0', false);
    for c in line.chars() {
        if quote != '\0' {
            if c == quote {
                quote = '\0';
            } else {
                word.push(c);
            }
        } else if c == '\'' || c == '"' {
            quote = c;
            open = true;
        } else if c.is_ascii_whitespace() {
            if open || !word.is_empty() {
                words.push(std::mem::take(&mut word));
                open = false;
            }
        } else {
            word.push(c);
        }
    }
    if open || !word.is_empty() {
        words.push(word);
    }
    words
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r05_a_fixture_line_splits_the_way_the_shell_splits_it() {
        assert_eq!(
            words("plan add first --files 'src/**/x.sh'"),
            vec!["plan", "add", "first", "--files", "src/**/x.sh"]
        );
        assert_eq!(words("  "), Vec::<String>::new());
        assert_eq!(words("--files ''"), vec!["--files", ""]);
    }

    #[test]
    fn r05_the_exit_code_contract_of_a_checker_run() {
        assert_eq!(judge(0, "plan add").expect("a verdict"), Verdict::Pass);
        assert_eq!(judge(1, "plan add").expect("a verdict"), Verdict::Reject);
        let cannot = judge(2, "plan add").expect_err("2 is not a verdict");
        assert_eq!(cannot.code(), 2);
        assert_eq!(cannot.message(), "plan add exited 2 in the fixture sandbox");
    }
}
