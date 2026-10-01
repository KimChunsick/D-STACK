// verbs/verify/qa_selftest.rs
// The QA staging of the verify fixtures (R08, D-41, D-42): `<!-- selftest-qa: S1=<state> -->`
// records the usage scenario's QA scenario through the real verbs before verify judges the run.

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;

/// The QA text of the staged scenario: preparation, steps and expected result, as qa add wants.
const BODY: &str = "준비: 자체 검사 저장소가 있어요.\n단계: dstack verify를 실행해요.\n기대 결과: 종료 코드가 QA 결과를 따라요.";

/// The reason a failed, skipped or blocked result is recorded with.
const NOTE: &str = "자체 검사가 남긴 사유예요";

/// Stage the fixture's QA state: `missing` records nothing, `open` only the QA scenario, `met`,
/// `failed`, `skipped` and `blocked` its result too, and `tampered` a met result whose artifact
/// is edited after it was recorded. A fixture without the directive stages nothing; a step that
/// does not succeed is the runner failing to decide, never a verdict.
pub(super) fn stage(sandbox: &Sandbox, ctx: &Context, fixture: &Path) -> Result<()> {
    let Some(directive) = Sandbox::directive(fixture, "qa") else {
        return Ok(());
    };
    let Some((scenario, state)) = directive.split_once('=') else {
        return Err(Error::cannot_decide(format!("selftest: selftest-qa wants S<n>=<state>, got {directive}")));
    };
    let (status, tampered) = match state {
        "missing" => return Ok(()),
        "open" => ("open", false),
        "tampered" => ("met", true),
        "met" | "failed" | "skipped" | "blocked" => (state, false),
        other => return Err(Error::cannot_decide(format!("selftest: unknown selftest-qa state {other}"))),
    };
    let body = sandbox.artifact("qa-body.md", BODY)?;
    expect_ok(sandbox, ctx, &["qa", "add", "--scenario", scenario, "--from", &body.to_string_lossy()])?;
    if status == "open" {
        return Ok(());
    }
    let artifact = sandbox.artifact("qa1-run.txt", "QA1 실행했어요: 종료 코드 0이에요.")?;
    let path = artifact.to_string_lossy();
    let mut args = vec!["evidence", "add", "--qa", "QA1", "--artifact", &path, "--produced-by", "selftest"];
    args.extend(["--status", status]);
    if status != "met" {
        args.extend(["--note", NOTE]);
    }
    expect_ok(sandbox, ctx, &args)?;
    if tampered {
        let text = std::fs::read_to_string(&artifact).unwrap_or_default();
        std::fs::write(&artifact, format!("{text}기록한 뒤에 손으로 고쳤어요.\n")).map_err(|e| {
            Error::cannot_decide(format!("selftest: cannot write {}: {e}", artifact.display()))
        })?;
    }
    Ok(())
}

fn expect_ok(sandbox: &Sandbox, ctx: &Context, args: &[&str]) -> Result<()> {
    match sandbox.dsx(ctx, args)? {
        (0, _) => Ok(()),
        (code, output) => Err(Error::cannot_decide(format!(
            "selftest: dstack {} exited {code}: {output}",
            args.join(" ")
        ))),
    }
}
