// verbs/request/selftests.rs
// The three fixture checkers of the noun: req add, check request and request approve (R100).

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

pub fn all() -> Vec<Box<dyn Selftest>> {
    vec![
        Box::new(ReqAdd),
        Box::new(CheckRequest),
        Box::new(RequestApprove),
    ]
}

/// The fixture is the --id option of a second row: R42 forbids reusing or lowering a number, so
/// a sandbox that already holds R01 must refuse "--id R01" and accept "--id R05".
struct ReqAdd;

impl Selftest for ReqAdd {
    fn checker(&self) -> &'static str {
        "req-add"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        setup(
            &sandbox,
            ctx,
            &[
                "request",
                "new",
                "--type",
                "cli",
                "--title",
                "req add fixture",
            ],
        )?;
        setup(
            &sandbox,
            ctx,
            &[
                "req",
                "add",
                "the first row",
                "--accept",
                "the first criterion",
            ],
        )?;
        let options = read(fixture)?;
        let mut args: Vec<&str> = vec![
            "req",
            "add",
            "the second row",
            "--accept",
            "the second criterion",
        ];
        args.extend(options.iter().map(String::as_str));
        verdict(sandbox.dsx(ctx, &args)?.0, "req add")
    }
}

struct CheckRequest;

impl Selftest for CheckRequest {
    fn checker(&self) -> &'static str {
        "check-request"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        copy(fixture, &sandbox.run_dir()?.join("request.md"))?;
        verdict(sandbox.dsx(ctx, &["check", "request"])?.0, "check request")
    }
}

/// The scenario, not just the file, is the fixture here: R46's claim is about what happens to an
/// approved file when someone edits it, so the bad case has to perform that edit.
struct RequestApprove;

impl Selftest for RequestApprove {
    fn checker(&self) -> &'static str {
        "request-approve"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::new(ctx)?;
        let request = sandbox.run_dir()?.join("request.md");
        copy(fixture, &request)?;
        setup(&sandbox, ctx, &["request", "approve"])?;
        if name(fixture).starts_with("bad-") {
            append(&request, "a line appended after approval\n")?;
        }
        verdict(sandbox.dsx(ctx, &["check", "request"])?.0, "check request")
    }
}

/// The exit-code contract of a checker run: 0 is the fixture passing, 1 is the refusal it was
/// written to provoke, and 2 (or a spawn failure) is the runner not being able to decide at all —
/// never a rejection.
fn verdict(code: i32, what: &str) -> Result<Verdict> {
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

fn name(fixture: &Path) -> String {
    fixture
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

/// The fixture's words, as the shell's unquoted `$opts` expands them.
fn read(fixture: &Path) -> Result<Vec<String>> {
    let text = std::fs::read_to_string(fixture)
        .map_err(|e| Error::cannot_decide(format!("cannot read {}: {e}", fixture.display())))?;
    Ok(text.split_whitespace().map(String::from).collect())
}

fn copy(fixture: &Path, to: &Path) -> Result<()> {
    std::fs::copy(fixture, to)
        .map(|_| ())
        .map_err(|e| Error::cannot_decide(format!("cannot copy {}: {e}", fixture.display())))
}

fn append(path: &Path, text: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))?;
    file.write_all(text.as_bytes())
        .map_err(|e| Error::cannot_decide(format!("cannot write {}: {e}", path.display())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r05_the_exit_code_contract_of_a_checker_run() {
        assert_eq!(verdict(0, "x").expect("a verdict"), Verdict::Pass);
        assert_eq!(verdict(1, "x").expect("a verdict"), Verdict::Reject);
        let cannot = verdict(2, "check request").expect_err("2 is not a verdict");
        assert_eq!(cannot.code(), 2);
        assert_eq!(
            cannot.message(),
            "check request exited 2 in the fixture sandbox"
        );
    }
}
