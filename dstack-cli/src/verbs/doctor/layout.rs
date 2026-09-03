// verbs/doctor/layout.rs
// doctor section 8: one responsibility per source file, 350 lines each at most (R09, D-20).

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::Result;
use crate::core::paths::base_name;
use crate::selftest::{Selftest, Verdict};

/// A source file that keeps growing is several responsibilities in one place; the split is the
/// fix, never a longer file.
const MAX_LINES: usize = 350;

pub fn section(ctx: &mut Context) -> Result<bool> {
    let src = ctx.home.repo.join("dstack-cli/src");
    say!(
        ctx,
        "lib layout (≤ {MAX_LINES} lines per file): file | lines | responsibility"
    );
    let files = sources(&src);
    let (mut total, mut bad) = (0, 0);
    for file in &files {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        total += text.matches('\n').count();
        // The path below src, because one basename (mod.rs) names a dozen files in the tree.
        let shown = file.strip_prefix(&src).unwrap_or(file).to_string_lossy();
        if !row(ctx, &shown, &text) {
            bad += 1;
        }
    }
    say!(
        ctx,
        "  files {}, lines {total}, over the limit {bad}",
        files.len()
    );
    Ok(bad == 0)
}

/// One table row; false when the file is over the limit.
fn row(ctx: &mut Context, shown: &str, text: &str) -> bool {
    let lines = text.matches('\n').count();
    if lines > MAX_LINES {
        say!(
            ctx,
            "  {shown} | {lines} | FAIL: over {MAX_LINES} lines — split it by responsibility"
        );
        return false;
    }
    let what = responsibility(text);
    say!(ctx, "  {shown} | {lines} | {what}");
    true
}

/// Line 2 without its comment marker — `#` in the shell files, `//` in the Rust ones.
fn responsibility(text: &str) -> String {
    let second = text
        .lines()
        .nth(1)
        .unwrap_or_default()
        .trim_start_matches(['#', '/'])
        .trim_start_matches(' ');
    match second.is_empty() {
        true => "(no responsibility line)".to_string(),
        false => second.to_string(),
    }
}

/// Every .rs file below dstack-cli/src, as its path relative to that directory, in name order.
fn sources(src: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(src, &mut files);
    files.sort();
    files
}

fn collect(at: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(at) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

/// claude/lint/fixtures/lib-size/* — a file over the limit is rejected whatever it holds.
pub struct Checker;

impl Selftest for Checker {
    fn checker(&self) -> &'static str {
        "lib-size"
    }

    fn run(&self, ctx: &mut Context, fixture: &Path) -> Result<Verdict> {
        let text = std::fs::read_to_string(fixture).unwrap_or_default();
        let shown = base_name(fixture);
        Ok(match super::quiet(ctx, |ctx| row(ctx, &shown, &text)) {
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
    fn r09__no_source_file_of_the_crate_is_over_the_limit() {
        let (held, printed) = super::super::tests::printed(section);
        assert!(held, "a source file is over the limit:\n{printed}");
        let last = printed.lines().last().expect("the count line");
        assert!(last.starts_with("  files "), "unexpected line: {last}");
        assert!(last.ends_with(", over the limit 0"), "unexpected: {last}");
    }

    /// The row names the file, its line count and what line 2 says it is responsible for.
    #[test]
    fn r09__a_row_carries_the_responsibility_line() {
        let mut ctx = super::super::tests::context();
        ctx.out.begin_capture();
        let held = row(
            &mut ctx,
            "core/out.rs",
            "// core/out.rs\n// The output sink.\nuse x;\n",
        );
        let (printed, _) = ctx.out.end_capture();
        assert!(held);
        assert_eq!(printed, "  core/out.rs | 3 | The output sink.\n");
        assert_eq!(
            responsibility("#!/usr/bin/env bash\n# what it does\n"),
            "what it does"
        );
        assert_eq!(
            responsibility("one line only\n"),
            "(no responsibility line)"
        );
    }

    #[test]
    fn r05__the_checker_judges_every_fixture_by_its_name() {
        let mut ctx = super::super::tests::context();
        let dir = ctx.home.home.join("lint/fixtures/lib-size");
        let fixtures = super::super::glob(&dir, "");
        assert!(
            fixtures.iter().any(|file| base_name(file).ends_with(".rs")),
            "R09 asks for .rs fixtures"
        );
        for fixture in fixtures {
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
}
