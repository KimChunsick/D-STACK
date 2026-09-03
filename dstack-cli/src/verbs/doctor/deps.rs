// verbs/doctor/deps.rs
// doctor section 1: what deps.tsv declares against what the machine actually has (R13).

use std::path::Path;

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::tools::{deps_file, tool_present};
use crate::selftest::{Selftest, Verdict};

pub fn section(ctx: &mut Context) -> Result<bool> {
    let path = deps_file(&ctx.home);
    table(ctx, &path)
}

/// `_doctor_deps`: one row per declared tool, false when a goal-closing one is missing. A table
/// that is not there is a failing section too (the shell's `return 2` reaches the same `||`).
fn table(ctx: &mut Context, path: &Path) -> Result<bool> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(_) => {
            say!(ctx, "deps: no deps table at {} (R13)", path.display());
            return Ok(false);
        }
    };
    say!(ctx, "deps ({}):", path.display());
    say!(ctx, "  name | present | needed_when | install");
    let (mut rows, mut ok, mut missing, mut gmiss) = (0, 0, 0, 0);
    for line in text.lines() {
        let column: Vec<&str> = line.split('\t').collect();
        let name = column.first().copied().unwrap_or("");
        if name.is_empty() || name.starts_with('#') || name == "name" {
            continue;
        }
        let field = |at: usize| column.get(at).copied().unwrap_or("");
        rows += 1;
        let present = tool_present(field(1))?;
        if present {
            ok += 1;
        } else {
            missing += 1;
            if field(5) == "goal-closing" {
                gmiss += 1;
            }
        }
        let word = if present { "yes" } else { "no" };
        say!(ctx, "  {name} | {word} | {} | {}", field(5), field(2));
    }
    say!(
        ctx,
        "  deps: {rows} rows, present {ok}, missing {missing} (goal-closing missing {gmiss})"
    );
    Ok(gmiss == 0)
}

/// claude/lint/fixtures/deps/*.tsv — a table whose goal-closing tool cannot be found is rejected.
pub struct Checker;

impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "deps"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        Ok(match super::quiet(ctx, |ctx| table(ctx, fixture))? {
            true => Verdict::Pass,
            false => Verdict::Reject,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r13__the_section_counts_the_rows_of_the_repository_table() {
        let (held, printed) = super::super::tests::printed(section);
        let last = printed.lines().last().expect("the count line");
        assert!(
            last.starts_with("  deps: "),
            "unexpected count line: {last}"
        );
        assert!(
            last.ends_with("(goal-closing missing 0)") == held,
            "the verdict and the count disagree: {last}"
        );
        assert!(printed
            .lines()
            .any(|line| line == "  name | present | needed_when | install"));
    }

    #[test]
    fn r05__the_checker_judges_both_fixtures() {
        let mut ctx = super::super::tests::context();
        let dir = ctx.home.home.join("lint/fixtures/deps");
        assert_eq!(
            Checker
                .run(&mut ctx, &dir.join("bad-missing-tool.tsv"))
                .expect("decides"),
            Verdict::Reject
        );
        assert_eq!(
            Checker
                .run(&mut ctx, &dir.join("good-present.tsv"))
                .expect("decides"),
            Verdict::Pass
        );
        assert_eq!(
            Checker
                .run(&mut ctx, &dir.join("no-such-fixture.tsv"))
                .expect("decides"),
            Verdict::Reject
        );
    }
}
