// verbs/quick/selftests.rs
// claude/lint/fixtures/quick-new/*.txt — the rejection paths `quick new` owns (R100).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

/// `deps: broken` swaps in a table whose review=on tool cannot be found, so `--review` has to be
/// refused before the task directory exists (R105).
const BROKEN_DEPS: &str = "name\tprobe\tinstall\tsource\tauth\tneeded_when\trequired_by\tgroup\n\
git\tcommand -v git\t-\t-\tno\tgoal-closing\talways\t\n\
codex\tcommand -v dstack-absent-tool\tnpm install -g @openai/codex\t-\tyes\tgoal-closing\treview=on\t\n";

/// The fixture is a tiny `key: value` script, so one checker covers every rejection path without
/// hard-coding fixture names: `args` are the arguments after `quick new`, `repeat` is how many
/// times to run them (the LAST run decides), `deps` names the table to run against.
pub struct QuickNew;

impl Selftest for QuickNew {
    fn checker(&self) -> &'static str {
        "quick-new"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let script = std::fs::read_to_string(fixture).map_err(|e| {
            Error::cannot_decide(format!("selftest: cannot read {}: {e}", fixture.display()))
        })?;
        let args = directive(&script, "args").unwrap_or_default();
        let repeat = directive(&script, "repeat")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(1);
        let sandbox = Sandbox::new(ctx)?;
        if directive(&script, "deps").as_deref() == Some("broken") {
            let table = sandbox.dir.join(".deps.tsv");
            std::fs::write(&table, BROKEN_DEPS).map_err(|e| {
                Error::cannot_decide(format!("selftest: cannot write {}: {e}", table.display()))
            })?;
        }
        let mut argv = vec!["quick", "new"];
        argv.extend(args.split_whitespace());
        let mut code = 0;
        for _ in 0..repeat {
            code = sandbox.dsx(ctx, &argv)?.0;
        }
        verdict(code)
    }
}

/// The exit-code contract of a checker run: 0 is the fixture passing, 1 is the refusal it was
/// written to provoke, and 2 is the runner not being able to decide at all.
///
/// selftest_quick_new called every nonzero exit a rejection, which was safe while `quick new`
/// had no path ending in 2. D-12 gave the port such paths — an unreadable template, a STATE.md
/// that cannot be read, a deps probe the port refuses to run — and a bad fixture that hit one of
/// them would otherwise read as "correctly rejected", which is exactly the reading the fixture
/// runner exists to rule out.
fn verdict(code: i32) -> Result<Verdict> {
    match code {
        0 => Ok(Verdict::Pass),
        1 => Ok(Verdict::Reject),
        other => Err(Error::cannot_decide(format!(
            "selftest: dstack quick new exited {other} instead of deciding"
        ))),
    }
}

/// `awk '/^<key>:/{sub(/^<key>: */,""); print; exit}'`: the first line that opens with the key.
fn directive(script: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    let line = script.lines().find(|line| line.starts_with(&prefix))?;
    Some(line[prefix.len()..].trim_start_matches(' ').to_string())
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r05__a_checker_that_could_not_run_is_not_a_rejection() {
        assert_eq!(verdict(0).expect("pass"), Verdict::Pass);
        assert_eq!(verdict(1).expect("reject"), Verdict::Reject);
        let cannot = verdict(2).expect_err("cannot decide");
        assert_eq!(cannot.code(), 2);
        assert_eq!(
            cannot.message(),
            "selftest: dstack quick new exited 2 instead of deciding"
        );
    }

    #[test]
    fn r05__the_fixture_script_reads_its_first_line_per_key() {
        let script = "# a comment\nargs: needs-review --review\nrepeat: 2\ndeps: broken\n";
        assert_eq!(
            directive(script, "args").as_deref(),
            Some("needs-review --review")
        );
        assert_eq!(directive(script, "repeat").as_deref(), Some("2"));
        assert_eq!(directive(script, "deps").as_deref(), Some("broken"));
        assert_eq!(directive(script, "missing"), None);
    }
}
