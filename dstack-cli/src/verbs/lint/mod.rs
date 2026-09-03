// verbs/lint/mod.rs
// dstack lint-ko: the Korean rule check for files, stdin and changed files, and its two checkers.

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::verb::Verb;
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

pub mod rules;
pub mod run;
pub mod scope;

pub fn verbs() -> Vec<Box<dyn Verb>> {
    run::verbs()
}

pub fn selftests() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(Fixtures), Box::new(Rules)]
}

/// claude/lint/fixtures/lint-ko/*.md — the fixture is written to README.md inside a sandbox, which
/// ko-scope.tsv maps to ko-haeyo, and lint-ko is run as a subprocess so scope resolution and the
/// exit code are exercised the way the hook exercises them. The sandbox has no store on purpose:
/// working without one is the property this noun has to keep.
struct Fixtures;

impl Selftest for Fixtures {
    fn checker(&self) -> &'static str {
        "lint-ko"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let sandbox = Sandbox::scratch()?;
        std::fs::copy(fixture, sandbox.dir.join("README.md")).map_err(|e| {
            Error::cannot_decide(format!(
                "selftest lint-ko: cannot copy {}: {e}",
                fixture.display()
            ))
        })?;
        let (code, output) = sandbox.dsx(ctx, &["lint-ko", "README.md"])?;
        // A checker that could not run is a failure of the runner, never a rejection: only the
        // exit codes the noun promises (0 pass, 1 the checked condition failed) are a verdict.
        match code {
            0 => Ok(Verdict::Pass),
            1 => Ok(Verdict::Reject),
            _ => Err(Error::cannot_decide(format!(
                "selftest lint-ko: {} exited {code}: {output}",
                fixture.display()
            ))),
        }
    }
}

/// claude/lint/fixtures/lint-ko-rules/*.tsv — every regex row must compile and must be matched by
/// its own example. A rule whose example stopped matching has silently stopped catching prose,
/// which is the one failure mode a rule table cannot report about itself. Both failures are a
/// rejection, never a refusal to decide: a table this engine cannot run is exactly what the
/// checker exists to catch, and each failing row is named by its id (R06).
struct Rules;

impl Selftest for Rules {
    fn checker(&self) -> &'static str {
        "lint-ko-rules"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        if !fixture.is_file() {
            return Ok(Verdict::Reject);
        }
        let mut bad = 0;
        for row in rules::rows(fixture)? {
            if row.kind != "regex" {
                continue;
            }
            match rules::Matcher::compile(&row.id, &row.pattern) {
                // The compile error of the regex crate spans several lines; the checker prints
                // one line per row, so it is folded into one.
                Err(error) => {
                    ctx.out.say(&one_line(error.message()));
                    bad += 1;
                }
                Ok(matcher) => {
                    if matcher.find(&row.example).is_empty() {
                        ctx.out.say(&format!(
                            "rule {}: example does not match its pattern",
                            row.id
                        ));
                        bad += 1;
                    }
                }
            }
        }
        Ok(match bad {
            0 => Verdict::Pass,
            _ => Verdict::Reject,
        })
    }
}

fn one_line(message: &str) -> String {
    message.split_whitespace().collect::<Vec<&str>>().join(" ")
}
