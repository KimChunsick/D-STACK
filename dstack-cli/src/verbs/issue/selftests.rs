// verbs/issue/selftests.rs
// claude/lint/fixtures/issue-new/*.txt — the refusal paths `issue new` owns, and the file it
// writes when none of them fires (R06). The verdict each run earns is verdict.rs.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::selftest::sandbox::Sandbox;
use crate::selftest::{Selftest, Verdict};

use super::asked::Asked;
use super::file::Filing;
use super::filed::{issues, plant};
use super::run::Run;
use super::slug::slug;
use super::verdict::verdict;

pub fn all() -> Vec<Box<dyn Selftest>> {
    vec![Box::new(IssueNew)]
}

/// The fixture is a tiny `key: value` script, so one checker covers every rejection path without
/// hard-coding fixture names. `title` is the operand; `symptom`, `repro`, `source` and `proposal`
/// are the value of the option of that name — a key that is not there is an option that is not
/// passed at all, and a key with nothing behind it is the option with an empty value. `extra` is
/// appended as raw arguments, `repeat` is how many filings run — every one of them is judged and
/// they have to end the same way, since a repeat whose first filing was refused after writing
/// leaves the folder a clean repeat leaves — and `plant` writes a file dstack did not write where
/// this filing would land.
struct IssueNew;

impl Selftest for IssueNew {
    fn checker(&self) -> &'static str {
        "issue-new"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let script = std::fs::read_to_string(fixture).map_err(|e| {
            Error::cannot_decide(format!("selftest: cannot read {}: {e}", fixture.display()))
        })?;
        let sandbox = Sandbox::new(ctx)?;
        let dir = issues(&sandbox);
        let planted = directive(&script, "plant").is_some();
        if planted {
            plant(
                &dir,
                &slug(&directive(&script, "title").unwrap_or_default()),
            )?;
        }
        let repeat = directive(&script, "repeat")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(1);
        let args = argv(&script);
        let mut runs: Vec<Run> = Vec::new();
        for _ in 0..repeat {
            runs.push(file_it(&sandbox, ctx, &args)?);
        }
        let asked = Asked {
            filing: filing(&script),
            runs: repeat,
        };
        verdict(&runs, &dir, planted, &asked)
    }
}

/// What the fixture supplied, as the filing the file it lands on has to carry. A key that is not
/// there is an option that was not passed, which is a field the filing never had.
fn filing(script: &str) -> Filing {
    Filing {
        title: directive(script, "title").unwrap_or_default(),
        symptom: directive(script, "symptom").unwrap_or_default(),
        repro: directive(script, "repro").unwrap_or_default(),
        source: directive(script, "source").unwrap_or_default(),
        proposal: directive(script, "proposal").unwrap_or_default(),
    }
}

/// The arguments of one filing: the operand the fixture named, the options it filled, and the raw
/// extras behind them — an unknown option or a second operand is spelled as one of those.
fn argv(script: &str) -> Vec<String> {
    let mut args = vec!["issue".to_string(), "new".to_string()];
    if let Some(title) = directive(script, "title") {
        args.push(title);
    }
    for name in ["symptom", "repro", "source", "proposal"] {
        if let Some(value) = directive(script, name) {
            args.push(format!("--{name}"));
            args.push(value);
        }
    }
    args.extend(
        directive(script, "extra")
            .unwrap_or_default()
            .split_whitespace()
            .map(String::from),
    );
    args
}

/// One filing, with HOME naming the sandbox. `folder()` reads HOME and nothing else (D-05), and
/// Sandbox::dsx hands the subprocess the environment this process runs in — setting HOME here
/// would move the folder for every other checker of the same `doctor --self` run, so the one call
/// each fixture is judged by is spawned with an environment of its own instead. Its two streams
/// are captured the way dsx captures them, merged as `2>&1` merges them: a run this checker cannot
/// judge has to be diagnosable from the row it printed.
fn file_it(sandbox: &Sandbox, ctx: &Context, args: &[String]) -> Result<Run> {
    let log = sandbox.dir.join(".issue.out");
    let file = std::fs::File::create(&log).map_err(|e| {
        Error::cannot_decide(format!("selftest: cannot write {}: {e}", log.display()))
    })?;
    let merged = file
        .try_clone()
        .map_err(|e| Error::cannot_decide(format!("selftest: {e}")))?;
    let status = Command::new(&ctx.self_exe)
        .args(args)
        .current_dir(&sandbox.dir)
        .env("HOME", sandbox.dir.join("home"))
        .env("DSTACK_DEPS", sandbox.dir.join(".deps.tsv"))
        .env_remove("DSTACK_ROOT")
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(merged))
        .status()
        .map_err(|e| {
            Error::cannot_decide(format!(
                "selftest: cannot run {}: {e}",
                ctx.self_exe.display()
            ))
        })?;
    let said = last_line(&printed(&log)?);
    let _ = std::fs::remove_file(&log);
    Ok(Run::from_status(status, said))
}

/// What the child printed, read back from the capture. D-12: a capture that is not there is
/// honestly empty — a child may print nothing at all — but one that is there and cannot be read,
/// bytes that are not UTF-8 included, is no answer, and losing it silently would leave the row
/// that has to be diagnosed with nothing on it.
fn printed(log: &Path) -> Result<String> {
    match std::fs::read_to_string(log) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(Error::cannot_decide(format!(
            "selftest: cannot read {}: {e}",
            log.display()
        ))),
    }
}

/// `2>&1 | tail -1`: the last line the child put anything on.
fn last_line(printed: &str) -> String {
    printed
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or_default()
        .to_string()
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

    const FILING: &str = "# a comment\ntitle: the verb refuses a file worktree\n\
        symptom: it exits 1\nrepro: dstack plan start P4\nsource: lifecycle.rs\n";

    #[test]
    fn r06__the_fixture_script_becomes_the_arguments_of_one_filing() {
        assert_eq!(
            argv(FILING),
            [
                "issue",
                "new",
                "the verb refuses a file worktree",
                "--symptom",
                "it exits 1",
                "--repro",
                "dstack plan start P4",
                "--source",
                "lifecycle.rs"
            ]
        );
        // A key that is not there is an option that is not passed; one with nothing behind it is
        // the option with an empty value, which is the other half of what D-08 refuses.
        assert_eq!(
            argv("title: t\nsymptom:\n"),
            ["issue", "new", "t", "--symptom", ""]
        );
        assert_eq!(
            argv("title: t\nextra: --reason why\n"),
            ["issue", "new", "t", "--reason", "why"]
        );
        assert_eq!(directive(FILING, "missing"), None);
    }

    /// D-12: a capture that is there and cannot be read — bytes that are not UTF-8, a file the
    /// process may not open — is not the empty one a child that printed nothing leaves.
    #[test]
    fn r06__a_capture_that_cannot_be_read_is_not_an_empty_one() {
        let sandbox = Sandbox::scratch().expect("scratch repository");
        let log = sandbox.dir.join(".issue.out");
        assert_eq!(printed(&log).expect("a capture that is not there"), "");
        std::fs::write(&log, [b'o', b'k', 0xff, b'\n']).expect("bytes that are not UTF-8");
        let cannot = printed(&log).expect_err("cannot decide");
        assert_eq!(cannot.code(), 2);
        assert!(
            cannot
                .message()
                .starts_with(&format!("selftest: cannot read {}", log.display())),
            "{}",
            cannot.message()
        );
    }

    #[test]
    fn r06__the_last_line_the_child_printed_is_what_a_row_carries() {
        assert_eq!(
            last_line("issue: /x/y.md\n  sighting 1\n\n"),
            "  sighting 1"
        );
        assert_eq!(last_line(""), "");
    }
}
