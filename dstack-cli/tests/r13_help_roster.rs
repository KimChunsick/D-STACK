// tests/r13_help_roster.rs
// R13: the Rust roster is the shell roster plus the hook line, and dispatch answers as the shell does.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::PathBuf;
use std::process::{Command, Output};

use dstack_cli::core::registry::{Registry, ROSTER};
use dstack_cli::verbs::all_verbs;

const HOOK_LINE: &str = "  hook                   the four Claude Code hook events in-process: dstack hook inject|stop|agent-model|pre-write, payload on stdin, verdict JSON on stdout";

/// The issue lines of M1, which the shell roster has no noun for: they sit as their own block
/// before status, so the expected roster is built by inserting them there.
const ISSUE_LINES: [&str; 2] = [
    "  issue new              file the friction you hit with dstack itself into ~/Documents/dstack-issues (--symptom, --repro, --source, --proposal)",
    "  issue list             what has been filed: one row per issue with its sightings count and last seen",
];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn dstack(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dstack"))
        .args(args)
        .current_dir(repo())
        .output()
        .expect("run the port")
}

fn shell(args: &[&str]) -> Output {
    Command::new("bash")
        .arg(shell_ref::dispatcher())
        .args(args)
        .current_dir(repo())
        .output()
        .expect("run the shell dstack")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("utf-8")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("utf-8")
}

#[test]
fn r13__the_roster_is_the_shell_roster_plus_the_hook_line() {
    let ported = dstack(&["help"]);
    let shell = shell(&["help"]);
    assert_eq!(ported.status.code(), Some(0));
    assert_eq!(shell.status.code(), Some(0));

    let mut expected: Vec<String> = stdout(&shell).lines().map(String::from).collect();
    let status = expected
        .iter()
        .position(|line| line.starts_with("  status "))
        .expect("the shell roster has a status line");
    for (n, line) in ISSUE_LINES.iter().enumerate() {
        expected.insert(status + n, line.to_string());
    }
    let gate = expected
        .iter()
        .position(|line| line.starts_with("  gate "))
        .expect("the shell roster has a gate line");
    expected.insert(gate + 1, HOOK_LINE.to_string());
    let last = expected.len() - 1;
    assert_eq!(expected[last], "verbs: 59");
    let count = format!("verbs: {}", 60 + ISSUE_LINES.len());
    expected[last] = count.clone();

    let actual: Vec<String> = stdout(&ported).lines().map(String::from).collect();
    assert_eq!(
        actual, expected,
        "the roster differs by more than the hook line and the issue lines"
    );
    assert_eq!(actual.last(), Some(&count));
}

#[test]
fn r13__no_arguments_print_the_roster() {
    let bare = dstack(&[]);
    let help = dstack(&["help"]);
    assert_eq!(bare.status.code(), Some(0));
    assert_eq!(stdout(&bare), stdout(&help));
    assert_eq!(stdout(&dstack(&["-h"])), stdout(&help));
    assert_eq!(stdout(&dstack(&["--help"])), stdout(&help));
}

#[test]
fn r13__an_unknown_noun_is_refused_as_the_shell_refuses_it() {
    let ported = dstack(&["bogus"]);
    let shell = shell(&["bogus"]);
    assert_eq!(
        stderr(&ported),
        "dstack: unknown command: bogus (dstack help)\n"
    );
    assert_eq!(stderr(&ported), stderr(&shell));
    assert_eq!(ported.status.code(), Some(1));
    assert_eq!(ported.status.code(), shell.status.code());
    assert_eq!(stdout(&ported), "");
}

#[test]
fn r13__an_unknown_verb_names_its_noun() {
    for (args, message) in [
        (
            vec!["run", "bogus"],
            "dstack: unknown verb for run: bogus (dstack help)\n",
        ),
        (
            vec!["run"],
            "dstack: unknown verb for run:  (dstack help)\n",
        ),
    ] {
        let ported = dstack(&args);
        let shell = shell(&args);
        assert_eq!(stderr(&ported), message);
        assert_eq!(stderr(&ported), stderr(&shell));
        assert_eq!(ported.status.code(), Some(1));
        assert_eq!(ported.status.code(), shell.status.code());
    }
}

#[test]
fn r13__a_roster_entry_without_a_handler_cannot_decide() {
    // The entry is looked up, never written down: an entry named by hand turns into a real call
    // the day that verb is ported, and this test drives dstack inside this repository. The
    // registry answers the question, so `help` — which the dispatcher renders itself and no
    // handler answers — is not mistaken for a gap.
    let registry = Registry::new(all_verbs());
    let entry = ROSTER
        .iter()
        .map(|(name, _)| *name)
        .find(|name| !registry.has_handler(name));
    let entry = match entry {
        Some(entry) => entry,
        None => return,
    };
    let ported = dstack(&entry.split(' ').collect::<Vec<&str>>());
    assert_eq!(stderr(&ported), format!("dstack: not ported yet: {entry}\n"));
    assert_eq!(ported.status.code(), Some(2));
    assert_eq!(stdout(&ported), "");
}
