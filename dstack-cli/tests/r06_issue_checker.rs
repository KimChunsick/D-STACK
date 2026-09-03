// tests/r06_issue_checker.rs
// R06: the issue-new checker judged the way `dstack doctor --self` judges it — every fixture under
// claude/lint/fixtures/issue-new gets the verdict its name asks for — plus the mutation that keeps
// those fixtures load-bearing: a whole filing reads pass, and the same filing with one field taken
// out reads reject, so a checker that answered the same word to everything is caught here.
// D-05 fixes the folder at $HOME/Documents/dstack-issues, which on this machine is the
// maintainer's own: the sweep below is held to leaving it exactly as it found it.
#![allow(non_snake_case)]

use std::path::{Path, PathBuf};
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::{Selftest, Verdict};
use dstack_cli::verbs;
use dstack_cli::verbs::doctor::selfrun;

const CHECKER: &str = "issue-new";

/// A filing with nothing missing, spelled the way a fixture spells one.
const WHOLE: &str = "title: the mutation files what a worker hit
symptom: it printed nothing and exited 1
repro: dstack issue new
source: dstack-cli/src/verbs/issue/new.rs
";

fn context() -> Context {
    let home = Home::resolve().expect("the repository of this test binary");
    Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    )
}

/// The registered checker, by the name the fixture directory carries.
fn checker() -> Box<dyn Selftest> {
    verbs::all_selftests()
        .into_iter()
        .find(|registered| registered.checker() == CHECKER)
        .unwrap_or_else(|| panic!("no {CHECKER} checker is registered"))
}

fn fixtures_dir() -> PathBuf {
    Home::resolve()
        .expect("the repository of this test binary")
        .home
        .join("lint/fixtures")
        .join(CHECKER)
}

/// The verdict of one fixture. An Err is the runner's cannot-decide, which is never a verdict:
/// the reason is what the assertion has to carry.
fn verdict_of(fixture: &Path) -> Verdict {
    checker()
        .run(&mut context(), fixture)
        .unwrap_or_else(|e| panic!("{}: {}", fixture.display(), e.message()))
}

/// What the maintainer's own issue folder holds right now — None when it is not there at all, so
/// a fixture that created it is as visible as one that filed into it.
fn documents() -> Option<Vec<String>> {
    let dir = PathBuf::from(std::env::var("HOME").expect("HOME")).join("Documents/dstack-issues");
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .ok()?
        .flatten()
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    Some(names)
}

fn scratch() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-r06-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch directory");
    dir
}

/// One fixture script written into the scratch directory, named by the verdict it asks for.
fn fixture(dir: &Path, name: &str, script: &str) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, script).expect("write the fixture");
    path
}

/// The filing without one of its fields: the line that carries it is dropped, which is the same
/// miss as never passing the option.
fn without(script: &str, field: &str) -> String {
    let prefix = format!("{field}:");
    script
        .lines()
        .filter(|line| !line.starts_with(&prefix))
        .map(|line| format!("{line}\n"))
        .collect()
}

#[test]
fn r06__every_fixture_gets_the_verdict_its_name_asks_for() {
    let fixtures = selfrun::fixtures(&fixtures_dir());
    let counted = |wanted: Verdict| fixtures.iter().filter(|(_, v)| *v == wanted).count();
    assert!(
        counted(Verdict::Reject) >= 1 && counted(Verdict::Pass) >= 1,
        "{CHECKER} needs at least one bad-* and one good-* fixture, and has {} of them",
        fixtures.len()
    );
    let before = documents();
    for (fixture, expected) in &fixtures {
        assert_eq!(verdict_of(fixture), *expected, "{}", fixture.display());
    }
    // D-05: every fixture runs against a HOME of its own, so the real folder is untouched.
    assert_eq!(
        documents(),
        before,
        "a fixture reached the maintainer's ~/Documents/dstack-issues"
    );
}

#[test]
fn r06__a_field_taken_out_of_a_filing_flips_the_verdict() {
    let dir = scratch();
    let whole = fixture(&dir, "good-whole.txt", WHOLE);
    assert_eq!(
        verdict_of(&whole),
        Verdict::Pass,
        "a filing with nothing missing"
    );
    for field in ["title", "symptom", "repro", "source"] {
        let cut = fixture(
            &dir,
            &format!("bad-without-{field}.txt"),
            &without(WHOLE, field),
        );
        assert_eq!(
            verdict_of(&cut),
            Verdict::Reject,
            "a filing without {field}"
        );
    }
    std::fs::remove_dir_all(&dir).expect("clean up");
}
