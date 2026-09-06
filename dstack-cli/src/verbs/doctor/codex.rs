// verbs/doctor/codex.rs
// doctor section 3: every `codex exec` line carries the fixed model flags and JSON telemetry flag.

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::Result;
use crate::selftest::{Selftest, Verdict};

/// The flags a `codex exec` line is not allowed to be missing, in the order the note lists them.
const FLAGS: [&str; 4] = [
    "--ignore-user-config",
    "-m gpt-6-astra",
    "-c model_reasoning_effort=high",
    "--json",
];

pub fn section(ctx: &mut Context) -> Result<bool> {
    say!(
        ctx,
        "codex exec flags (R23): every line needs {}, {}, {}, {}",
        FLAGS[0],
        FLAGS[1],
        FLAGS[2],
        FLAGS[3]
    );
    let (mut total, mut bad) = (0, 0);
    for file in files(ctx) {
        let (lines, missing) = scan(ctx, &file);
        total += lines;
        bad += missing;
    }
    say!(ctx, "  codex exec lines: {total}, missing flags: {bad}");
    Ok(bad == 0)
}

/// The skills and agent definitions that may spawn Codex, in the order the shell globs them.
fn files(ctx: &Context) -> Vec<PathBuf> {
    let (home, repo) = (&ctx.home.home, &ctx.home.repo);
    let mut files = super::glob_sub(&home.join("skills"), "SKILL.md");
    files.extend(super::glob_sub(&repo.join("codex/skills"), "SKILL.md"));
    files.push(repo.join("codex/AGENTS.md"));
    files.extend(super::glob(&home.join("agents"), ".md"));
    files.retain(|file| file.is_file());
    files
}

/// `grep -n "codex exec"`: how many lines invoke Codex and how many of them are missing a flag.
fn scan(ctx: &mut Context, file: &Path) -> (usize, usize) {
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(_) => return (0, 0),
    };
    let (mut total, mut bad) = (0, 0);
    for (at, line) in text.lines().enumerate() {
        if !line.contains("codex exec") {
            continue;
        }
        total += 1;
        let missing: Vec<&str> = FLAGS
            .iter()
            .copied()
            .filter(|flag| !has_flag(line, flag))
            .collect();
        if !missing.is_empty() {
            bad += 1;
            say!(
                ctx,
                "  {}:{}: missing {}",
                file.display(),
                at + 1,
                missing.join(", ")
            );
        }
    }
    (total, bad)
}

/// Require whole flag values, allowing a closing Markdown code span after the final value.
fn has_flag(line: &str, flag: &str) -> bool {
    line.match_indices(flag).any(|(at, _)| {
        let before = line[..at].chars().next_back();
        let after = line[at + flag.len()..].chars().next();
        before.is_none_or(char::is_whitespace)
            && after.is_none_or(|ch| ch.is_whitespace() || ch == '`')
    })
}

/// claude/lint/fixtures/codex-flags/*.md — one Codex invocation per fixture.
pub struct Checker;

impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "codex-flags"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let (_, bad) = super::quiet(ctx, |ctx| scan(ctx, fixture));
        Ok(match bad {
            0 => Verdict::Pass,
            _ => Verdict::Reject,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::core::paths::base_name;

    #[test]
    fn r23__model_and_effort_are_exact() {
        let mut ctx = super::super::tests::context();
        let dir = ctx.home.home.join("lint/fixtures/codex-flags");
        for (name, bad) in [
            ("good-flags.md", 0),
            ("good-final-effort.md", 0),
            ("bad-old-model.md", 1),
            ("bad-medium-effort.md", 1),
            ("bad-xhigh-effort.md", 1),
            ("bad-model-suffix.md", 1),
            ("bad-effort-suffix.md", 1),
        ] {
            assert_eq!(scan(&mut ctx, &dir.join(name)), (1, bad), "{name}");
        }
    }

    #[test]
    fn r23__every_codex_line_of_this_repository_carries_the_flags() {
        let (held, printed) = super::super::tests::printed(section);
        assert!(held, "a codex exec line is missing a flag:\n{printed}");
        let last = printed.lines().last().expect("the count line");
        assert!(
            last.starts_with("  codex exec lines: ") && last.ends_with(", missing flags: 0"),
            "unexpected count line: {last}"
        );
    }

    #[test]
    fn r05__the_checker_judges_every_fixture_by_its_name() {
        let mut ctx = super::super::tests::context();
        let dir = ctx.home.home.join("lint/fixtures/codex-flags");
        for fixture in super::super::glob(&dir, ".md") {
            let wanted = match base_name(&fixture).starts_with("bad-") {
                true => Verdict::Reject,
                false => Verdict::Pass,
            };
            assert_eq!(
                Checker.run(&mut ctx, &fixture).expect("decides"),
                wanted,
                "{}",
                fixture.display()
            );
        }
    }

    /// The offending line is named with its file and its line number, as grep -n prints it.
    #[test]
    fn r23__a_missing_flag_names_the_line() {
        let mut ctx = super::super::tests::context();
        let fixture = ctx
            .home
            .home
            .join("lint/fixtures/codex-flags/bad-missing-flags.md");
        ctx.out.begin_capture();
        let (total, bad) = scan(&mut ctx, &fixture);
        let (printed, _) = ctx.out.end_capture();
        assert_eq!((total, bad), (1, 1));
        assert_eq!(
            printed,
            format!("  {}:3: missing --ignore-user-config\n", fixture.display())
        );
    }
}
