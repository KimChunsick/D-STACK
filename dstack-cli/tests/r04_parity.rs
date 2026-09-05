// tests/r04_parity.rs
// R04: the parity harness drives both implementations, masks, diffs and counts truthfully —
// today every difference it reports is a verb the port has not reached yet, and a step that
// differs by construction is reported as differing.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::PathBuf;
use std::process::Command;

const NOT_PORTED: &str = "dstack: not ported yet: ";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// A directory this test owns: --out is never deleted by the harness, so the test cleans it up.
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-parity-{name}-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    dir
}

/// The roster nouns of the shell dispatcher: the first word of every indented help line.
fn roster_nouns() -> Vec<String> {
    let out = Command::new("bash")
        .arg(shell_ref::dispatcher())
        .arg("help")
        .current_dir(repo())
        .output()
        .expect("run the shell dstack");
    let help = String::from_utf8(out.stdout).expect("utf-8");
    let mut nouns: Vec<String> = Vec::new();
    for line in help.lines() {
        let entry = match line.strip_prefix("  ") {
            Some(entry) => entry,
            None => continue,
        };
        let noun = entry
            .split_whitespace()
            .next()
            .expect("a roster entry")
            .to_string();
        if !nouns.contains(&noun) {
            nouns.push(noun);
        }
    }
    nouns
}

/// Runs the harness against the test binary and returns its stdout and exit code.
fn harness(args: &[&str]) -> (String, i32) {
    let out = Command::new("bash")
        .arg(repo().join("dstack-cli/parity/run.sh"))
        .args(["--shell-ref", "shell-final"])
        .args(["--rust", env!("CARGO_BIN_EXE_dstack")])
        .args(args)
        .output()
        .expect("run the parity harness");
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    let stderr = String::from_utf8(out.stderr).expect("utf-8");
    assert!(stderr.is_empty(), "the harness aborted: {stderr}");
    (stdout, out.status.code().expect("an exit code"))
}

/// One reported block: the call it names, the stream, and the lines of the rust side.
struct Block {
    call: String,
    stream: String,
    rust: Vec<String>,
}

fn blocks(report: &str) -> Vec<Block> {
    let mut found: Vec<Block> = Vec::new();
    for line in report.lines() {
        if let Some(rest) = line.strip_prefix("differing: ") {
            let (call, stream) = rest
                .rsplit_once(' ')
                .expect("a block header names its stream");
            found.push(Block {
                call: call.to_string(),
                stream: stream.trim_matches(['(', ')']).to_string(),
                rust: Vec::new(),
            });
        } else if let Some(added) = line.strip_prefix('+') {
            if !line.starts_with("+++") {
                if let Some(block) = found.last_mut() {
                    block.rust.push(added.to_string());
                }
            }
        }
    }
    found
}

fn tail_counts(report: &str) -> (usize, usize) {
    let last = report.lines().last().expect("a report line");
    let rest = last
        .strip_prefix("steps ")
        .expect("the report ends with the steps line");
    let (steps, differing) = rest
        .split_once(", differing ")
        .expect("the steps line counts both");
    (
        steps.parse().expect("a step count"),
        differing.parse().expect("a differing count"),
    )
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r04__every_difference_of_the_help_step_is_a_verb_the_port_has_not_reached() {
    let (report, _) = harness(&["--only", "00-help"]);
    let blocks = blocks(&report);
    let mut calls: Vec<&str> = blocks
        .iter()
        .filter(|b| b.call.starts_with("00-help/"))
        .map(|b| b.call.as_str())
        .collect();
    calls.sort();
    calls.dedup();

    for call in &calls {
        let stderr = blocks
            .iter()
            .find(|b| &b.call.as_str() == call && b.stream == "stderr")
            .unwrap_or_else(|| panic!("{call} differs without a stderr block:\n{report}"));
        assert_eq!(
            stderr.rust.len(),
            1,
            "{call} answers more than one stderr line:\n{report}"
        );
        assert!(
            stderr.rust[0].starts_with(NOT_PORTED),
            "{call} differs for another reason than the port: {}",
            stderr.rust[0]
        );
        let exit = blocks
            .iter()
            .find(|b| &b.call.as_str() == call && b.stream == "exit")
            .unwrap_or_else(|| {
                panic!("{call} answers not-ported without an exit block:\n{report}")
            });
        assert_eq!(exit.rust, vec!["2".to_string()], "{call} exits with 2");
    }

    assert!(
        report.contains("expected: 00-help/help — "),
        "the roster call is not reported as expected:\n{report}"
    );
    // A store file may differ only as long as some verb of the step is still unported: the shell
    // touches the owner rows of meta.tsv where the port answers "not ported yet" and writes
    // nothing. Once every call matches, the two stores have to match too.
    let store = blocks
        .iter()
        .filter(|b| b.call.starts_with("store/"))
        .count();
    if calls.is_empty() {
        assert_eq!(store, 0, "the calls agree but the stores differ:\n{report}");
    }
    let (steps, differing) = tail_counts(&report);
    assert!(steps > 0, "the step ran no call:\n{report}");
    assert_eq!(
        differing,
        calls.len() + store,
        "the count is the differing calls plus the differing store files:\n{report}"
    );
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r04__the_help_step_carries_a_wrong_usage_call_for_every_roster_noun() {
    let dir = scratch("nouns");
    harness(&[
        "--only",
        "00-help",
        "--out",
        dir.to_str().expect("a utf-8 path"),
    ]);
    let calls = std::fs::read_to_string(dir.join("shell/calls.tsv")).expect("the recorded calls");
    let nouns = roster_nouns();
    assert_eq!(nouns.len(), 24, "the shell roster nouns: {nouns:?}");
    for noun in &nouns {
        let wanted = format!("00-help\tusage-{noun}");
        assert!(
            calls.lines().any(|line| line == wanted),
            "no wrong-usage call for the noun {noun} (R11)"
        );
    }
    std::fs::remove_dir_all(&dir).expect("the harness leaves --out to its caller");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r04__a_declared_difference_that_is_not_there_is_reported() {
    let (report, code) = harness(&["--only", "expectcheck"]);
    assert!(
        report.contains("expected-not-met: expectcheck/same-on-both: no difference to expect"),
        "a stale expect_diff is not reported:\n{report}"
    );
    assert_eq!(
        tail_counts(&report),
        (1, 1),
        "expectcheck report:\n{report}"
    );
    assert_eq!(code, 1, "a differing run exits 1:\n{report}");
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r04__a_step_that_differs_by_construction_is_reported() {
    let (report, code) = harness(&["--self-check"]);
    assert_eq!(tail_counts(&report), (1, 1), "self-check report:\n{report}");
    assert_eq!(code, 1, "a differing run exits 1:\n{report}");
}
