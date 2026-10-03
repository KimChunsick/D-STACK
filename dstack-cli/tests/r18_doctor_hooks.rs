// tests/r18_doctor_hooks.rs
// R18: `dstack doctor` shows a hook another program registered as a note, not a failure, while a
// dstack hook that is missing, registered twice, registered under a stale event or named in a form
// doctor cannot verify still fails the hooks section. Every doctor run here reads a scratch home,
// never the settings.json of this machine, so the result is the same whatever that file holds.
#![allow(non_snake_case)]

#[path = "support/doctor_home.rs"]
mod doctor_home;

use std::path::PathBuf;
use std::process::Output;
use std::rc::Rc;

use doctor_home::ScratchHome;
use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::verbs;
use dstack_cli::verbs::doctor::selfrun;

const NOTE: &str = "  | note: registered by another program; dstack does not manage it";
const UNVERIFIABLE: &str = "  | FAIL: names the dstack wrapper in a form doctor cannot verify \
     — register it exactly as claude/settings.enforced.json does";

/// Doctor under a scratch home holding `settings`: the home (kept alive), the run and its stdout.
fn doctor(settings: &str) -> (ScratchHome, Output, String) {
    let home = ScratchHome::new(settings);
    let out = home.doctor().output().expect("run dstack doctor");
    let printed = String::from_utf8_lossy(&out.stdout).into_owned();
    (home, out, printed)
}

/// The hooks section: from its header up to the hook last results it closes with.
fn section(printed: &str) -> Vec<&str> {
    printed
        .lines()
        .skip_while(|line| !line.starts_with("hooks registered in "))
        .take_while(|line| !line.starts_with("hook last results"))
        .collect()
}

fn header(home: &ScratchHome) -> String {
    format!("hooks registered in {}:", home.settings().display())
}

#[test]
fn R18__a_hook_of_another_program_is_a_note_and_doctor_passes() {
    let (home, out, printed) = doctor(&doctor_home::fixture("good-foreign-hook-noted.json"));
    let hooks = section(&printed);
    assert_eq!(hooks.first().copied(), Some(header(&home).as_str()), "{printed}");
    let foreign: Vec<&&str> = hooks.iter().filter(|line| line.contains("notch-hook")).collect();
    assert_eq!(foreign.len(), 5, "every foreign hook is listed:\n{printed}");
    assert!(
        foreign.iter().all(|line| line.ends_with(NOTE)),
        "a foreign hook is not a note:\n{printed}"
    );
    assert!(
        !hooks.iter().any(|line| line.contains("FAIL")),
        "the hooks section failed:\n{printed}"
    );
    assert!(
        hooks.contains(&"  registered: 9, dstack: 4, other: 5"),
        "the count line:\n{printed}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "dstack doctor:\n{printed}{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn R18__a_missing_dstack_hook_fails_the_section() {
    let (_home, out, printed) = doctor(&doctor_home::fixture("bad-dstack-hook-missing.json"));
    let hooks = section(&printed);
    assert!(
        hooks.iter().any(|line| {
            line.starts_with("  FAIL: dstack hook missing: PreToolUse [Agent|Task] → ")
                && line.ends_with("dstack-hook.sh agent-model")
        }),
        "the missing hook is not named:\n{printed}"
    );
    assert_eq!(out.status.code(), Some(1), "dstack doctor:\n{printed}");
}

#[test]
fn R18__a_duplicated_dstack_hook_fails_the_section() {
    let (_home, out, printed) = doctor(&doctor_home::fixture("bad-dstack-hook-duplicated.json"));
    let hooks = section(&printed);
    assert!(
        hooks.iter().any(|line| {
            line.starts_with("  FAIL: dstack hook registered 2 times: Stop [*] → ")
                && line.ends_with("dstack-hook.sh stop (expected once)")
        }),
        "the duplicated hook is not named:\n{printed}"
    );
    assert_eq!(out.status.code(), Some(1), "dstack doctor:\n{printed}");
}

/// The Stop hook named as registered twice and every Stop row counted as a dstack hook; the
/// output of the run.
fn assert_stop_registered_twice(fixture: &str) -> String {
    let (_home, out, printed) = doctor(&doctor_home::fixture(fixture));
    let hooks = section(&printed);
    assert!(
        hooks.iter().any(|line| {
            line.starts_with("  FAIL: dstack hook registered 2 times: Stop [*] → ")
                && line.ends_with("dstack-hook.sh stop (expected once)")
        }),
        "{fixture}: the duplicated hook is not named:\n{printed}"
    );
    assert!(
        hooks.contains(&"  registered: 5, dstack: 5, other: 0"),
        "{fixture}: the second Stop is not a dstack hook:\n{printed}"
    );
    assert_eq!(out.status.code(), Some(1), "{fixture}: dstack doctor:\n{printed}");
    printed
}

#[test]
fn R18__a_duplicate_after_a_double_dash_fails_the_section() {
    let printed = assert_stop_registered_twice("bad-dstack-hook-duplicated-double-dash.json");
    let row = "  Stop [*] → bash -- \"$HOME/.claude/hooks/dstack-hook.sh\" stop";
    assert!(
        section(&printed).contains(&row),
        "the double-dash row is a note, not the dstack hook:\n{printed}"
    );
}

#[test]
fn R18__a_duplicate_behind_env_or_sh_c_fails_the_section() {
    for fixture in [
        "bad-dstack-hook-duplicated-env-prefix.json",
        "bad-dstack-hook-duplicated-sh-c.json",
    ] {
        assert_stop_registered_twice(fixture);
    }
}

#[test]
fn R18__every_invocation_form_counts_as_the_dstack_hook() {
    let (_home, out, printed) =
        doctor(&doctor_home::fixture("good-dstack-invocation-option-forms.json"));
    let hooks = section(&printed);
    assert!(!hooks.iter().any(|line| line.contains("FAIL")), "{printed}");
    assert!(hooks.contains(&"  registered: 4, dstack: 4, other: 0"), "{printed}");
    assert_eq!(out.status.code(), Some(0), "dstack doctor:\n{printed}");
}

/// Every expected dstack hook named as missing, after the line saying why none is registered.
fn assert_every_dstack_hook_missing(out: &Output, printed: &str, why: &str) {
    let hooks = section(printed);
    assert!(hooks.contains(&why), "the reason line:\n{printed}");
    for want in ["inject", "stop", "agent-model", "pre-write"] {
        assert!(
            hooks.iter().any(|line| {
                line.starts_with("  FAIL: dstack hook missing: ")
                    && line.ends_with(&format!("dstack-hook.sh {want}"))
            }),
            "the {want} hook is not named missing:\n{printed}"
        );
    }
    assert_eq!(out.status.code(), Some(1), "dstack doctor:\n{printed}");
}

#[test]
fn R18__an_empty_hooks_table_leaves_every_dstack_hook_missing() {
    let (_home, out, printed) = doctor(&doctor_home::fixture("bad-no-hooks.json"));
    assert_every_dstack_hook_missing(&out, &printed, "  (settings.json registers no hook)");
}

#[test]
fn R18__no_settings_json_leaves_every_dstack_hook_missing() {
    let home = ScratchHome::without_settings();
    let out = home.doctor().output().expect("run dstack doctor");
    let printed = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(section(&printed).first().copied(), Some(header(&home).as_str()));
    assert_every_dstack_hook_missing(
        &out,
        &printed,
        "  (no settings.json: no hook is registered on this machine)",
    );
}

/// A command that names dstack-hook.sh in a form doctor cannot read as the wrapper fails on its
/// own row and counts as a dstack hook; the output of the run.
fn assert_unverifiable(fixture: &str, row: &str, count: &str) -> String {
    let (_home, out, printed) = doctor(&doctor_home::fixture(fixture));
    let hooks = section(&printed);
    assert!(
        hooks.contains(&format!("{row}{UNVERIFIABLE}").as_str()),
        "{fixture}: the row does not fail as unverifiable:\n{printed}"
    );
    assert!(hooks.contains(&count), "{fixture}: the count line:\n{printed}");
    assert_eq!(out.status.code(), Some(1), "{fixture}: dstack doctor:\n{printed}");
    printed
}

#[test]
fn R18__a_command_that_only_mentions_the_wrapper_fails_the_section() {
    assert_unverifiable(
        "bad-mentions-wrapper.json",
        "  Notification [*] → logger dstack-hook.sh",
        "  registered: 5, dstack: 5, other: 0",
    );
}

#[test]
fn R18__a_duplicate_behind_an_option_argument_fails_the_section() {
    assert_unverifiable(
        "bad-dstack-hook-duplicated-option-argument.json",
        "  Stop [*] → bash -o pipefail \"$HOME/.claude/hooks/dstack-hook.sh\" stop",
        "  registered: 5, dstack: 5, other: 0",
    );
}

#[test]
fn R18__echoing_the_wrapper_does_not_register_the_stop_hook() {
    let printed = assert_unverifiable(
        "bad-echo-not-invoked.json",
        "  Stop [*] → echo dstack-hook.sh stop",
        "  registered: 4, dstack: 4, other: 0",
    );
    assert!(
        section(&printed).iter().any(|line| {
            line.starts_with("  FAIL: dstack hook missing: Stop [*] → ")
                && line.ends_with("dstack-hook.sh stop")
        }),
        "the Stop hook is not named missing:\n{printed}"
    );
}

/// The tests that run doctor never read this machine's settings.json: the section names the
/// scratch file, and the four dstack hooks give the same verdict with or without foreign ones.
#[test]
fn R18__doctor_reads_the_scratch_settings_whatever_this_machine_holds() {
    let (alone, alone_out, alone_printed) = doctor(&doctor_home::dstack_only());
    let (mixed, mixed_out, mixed_printed) =
        doctor(&doctor_home::fixture("good-foreign-hook-noted.json"));
    for (home, printed) in [(&alone, &alone_printed), (&mixed, &mixed_printed)] {
        assert_eq!(section(printed).first().copied(), Some(header(home).as_str()));
    }
    assert!(
        section(&alone_printed).contains(&"  registered: 4, dstack: 4, other: 0"),
        "{alone_printed}"
    );
    assert_eq!(alone_out.status.code(), mixed_out.status.code());
    assert_eq!(alone_printed.lines().last(), mixed_printed.lines().last());
}

/// doctor --self proves the judgement on claude/lint/fixtures/doctor-hooks: every bad-* is
/// rejected and every good-* passes.
#[test]
fn R18__the_doctor_hooks_checker_proves_its_fixtures() {
    let home = Home::resolve().expect("the repository of this test binary");
    let dir = home.home.join("lint/fixtures/doctor-hooks");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let checkers = verbs::all_selftests();
    let checker = checkers
        .iter()
        .find(|checker| checker.checker() == "doctor-hooks")
        .expect("a registered doctor-hooks checker");
    let fixtures = selfrun::fixtures(&dir);
    assert!(fixtures.len() >= 5, "the fixtures of R18 are there");
    for (fixture, expected) in fixtures {
        let actual = checker.run(&mut ctx, &fixture).expect("the checker decides");
        assert_eq!(actual, expected, "{}", fixture.display());
    }
}
