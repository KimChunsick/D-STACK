// verbs/doctor/selfrun.rs
// doctor --self: the fixture runner that proves every checker can fail (R05, R100).

use std::path::{Path, PathBuf};

use crate::core::context::Context;
use crate::core::error::{Error, Result};
use crate::core::paths::base_name;
use crate::selftest::{Selftest, Verdict};

/// What the closing line of the runner reports.
pub struct Counts {
    pub checkers: usize,
    pub fixtures: usize,
    pub passed: usize,
    pub failed: usize,
    pub zero: usize,
}

pub fn run(ctx: &mut Context) -> Result<()> {
    let counts = sweep(ctx, &crate::verbs::all_selftests())?;
    // The shell's last command is the `[ … ]` test: exit 1, nothing more printed.
    match counts.failed == 0 && counts.zero == 0 {
        true => Ok(()),
        false => Err(Error::Exit(1)),
    }
}

/// Every fixture directory against the registered checkers: bad-* must be rejected, good-* must
/// pass. The sweep is driven by both sides at once — every fixture directory and every registered
/// checker — so a directory nobody registered is a failing row, and a checker whose directory is
/// missing or holds only one kind of fixture is a zero-fixture checker. Neither can pass silently:
/// walking the directories alone would hide a checker that never proved it can fail.
pub fn sweep(ctx: &mut Context, checkers: &[Box<dyn Selftest>]) -> Result<Counts> {
    say!(ctx, "doctor --self: fixture runner (R100) — bad-* must be rejected, good-* must pass; this mode runs no other section");
    say!(ctx, "  checker | fixture | expected | actual | ok");
    let mut counts = Counts {
        checkers: 0,
        fixtures: 0,
        passed: 0,
        failed: 0,
        zero: 0,
    };
    let root = ctx.home.home.join("lint/fixtures");
    for checker in swept(&root, checkers) {
        let dir = root.join(&checker);
        counts.checkers += 1;
        let fixtures = fixtures(&dir);
        if !fixtures.iter().any(|(_, kind)| *kind == Verdict::Reject)
            || !fixtures.iter().any(|(_, kind)| *kind == Verdict::Pass)
        {
            counts.zero += 1;
            say!(
                ctx,
                "  {checker} | - | - | needs at least one bad-* and one good-* fixture | FAIL"
            );
        }
        let found = checkers.iter().find(|found| found.checker() == checker);
        let registered = match found {
            Some(registered) => registered,
            None => {
                counts.failed += 1;
                say!(ctx, "  {checker} | - | - | no registered checker | FAIL");
                continue;
            }
        };
        for (fixture, expected) in fixtures {
            counts.fixtures += 1;
            // One broken checker must not hide the others: a checker that could not decide is a
            // failing row carrying its reason, never an abort of the runner.
            let actual = match super::quiet(ctx, |ctx| registered.run(ctx, &fixture)) {
                Ok(verdict) => verdict.as_str().to_string(),
                Err(error) => error.message().to_string(),
            };
            let ok = actual == expected.as_str();
            match ok {
                true => counts.passed += 1,
                false => counts.failed += 1,
            }
            say!(
                ctx,
                "  {checker} | {} | {} | {actual} | {}",
                base_name(&fixture),
                expected.as_str(),
                match ok {
                    true => "ok",
                    false => "FAIL",
                }
            );
        }
    }
    say!(
        ctx,
        "checkers {}, fixtures {}, passed {}, failed {}, zero-fixture checkers {}",
        counts.checkers,
        counts.fixtures,
        counts.passed,
        counts.failed,
        counts.zero
    );
    Ok(counts)
}

/// Every name the sweep has to answer for: the fixture directories and the registered checkers,
/// merged and in name order, so neither side of the pair can be the only one asked.
fn swept(root: &Path, checkers: &[Box<dyn Selftest>]) -> Vec<String> {
    let mut names: Vec<String> = super::subdirs(root).iter().map(|dir| base_name(dir)).collect();
    for checker in checkers {
        if !names.iter().any(|name| name == checker.checker()) {
            names.push(checker.checker().to_string());
        }
    }
    names.sort();
    names
}

/// The fixtures of one directory with the verdict their name asks for, in name order. A file
/// named neither bad-* nor good-* is not a fixture and is not counted. A directory that is not
/// there holds none, which is what makes a registered checker without one a zero-fixture row.
pub fn fixtures(dir: &Path) -> Vec<(PathBuf, Verdict)> {
    super::glob(dir, "")
        .into_iter()
        .filter_map(|file| match base_name(&file) {
            name if name.starts_with("bad-") => Some((file, Verdict::Reject)),
            name if name.starts_with("good-") => Some((file, Verdict::Pass)),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn r05__a_fixture_is_named_by_what_it_has_to_prove() {
        let home = crate::core::roots::Home::resolve().expect("repository");
        let found = fixtures(&home.home.join("lint/fixtures/lib-size"));
        assert!(found.len() >= 2, "the bad-* and the good-* fixture");
        for (file, expected) in found {
            let wanted = match base_name(&file).starts_with("bad-") {
                true => Verdict::Reject,
                false => Verdict::Pass,
            };
            assert_eq!(expected, wanted, "{}", file.display());
        }
        assert!(fixtures(&home.home.join("lint/no-such-directory")).is_empty());
    }

    /// A checker whose fixture directory is not there is a zero-fixture checker, not an
    /// invisible one: the row it would have proved itself with was never written.
    struct Fixtureless;
    impl Selftest for Fixtureless {
        fn checker(&self) -> &'static str {
            "no-such-fixture-directory"
        }
        fn run(&self, _ctx: &mut Context, _fixture: &Path) -> Result<Verdict> {
            Ok(Verdict::Pass)
        }
    }

    #[test]
    fn r05__a_checker_without_a_fixture_directory_is_counted() {
        let mut ctx = super::super::tests::context();
        ctx.out.begin_capture();
        let counts = sweep(&mut ctx, &[Box::new(Fixtureless)]).expect("the runner decides");
        let (printed, _) = ctx.out.end_capture();
        assert_eq!(counts.zero, 1, "the fixture-less checker is the one:\n{printed}");
        assert!(
            printed.contains(
                "  no-such-fixture-directory | - | - | needs at least one bad-* and one good-* fixture | FAIL"
            ),
            "the row does not name the miss:\n{printed}"
        );
        assert!(
            counts.checkers > 1,
            "the fixture directories are swept next to it"
        );
    }

    /// A directory with no registered checker is a failing row, not an invisible one.
    #[test]
    fn r05__an_unregistered_checker_fails_its_directory() {
        let mut ctx = super::super::tests::context();
        ctx.out.begin_capture();
        let counts = sweep(&mut ctx, &[]).expect("the runner decides");
        let (printed, _) = ctx.out.end_capture();
        assert!(counts.checkers > 0, "the fixture directories are there");
        assert_eq!(counts.failed, counts.checkers, "every directory failed");
        assert_eq!(counts.fixtures, 0, "nothing ran");
        assert!(
            printed.contains(" | - | - | no registered checker | FAIL"),
            "the row does not name the miss:\n{printed}"
        );
    }
}
