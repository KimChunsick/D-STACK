// tests/r17_worker_cap.rs
// R17: the default cap on concurrent implementation workers is 5 — dstack init writes
// max_concurrent: 5, dstack next falls back to 5 without a policy value, a PROJECT.md value and
// --max still win, and runtime.md and the dstack-develop skill follow the PROJECT.md cap instead
// of hard-coding --max 3.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::Command;

use support::Scratch;

const PROJECT: &str = ".dstack/project/PROJECT.md";
const RUNTIME: &str = "claude/runtime.md";
const DEVELOP: &str = "claude/skills/dstack-develop/SKILL.md";

fn git(t: &Scratch, args: &[&str]) {
    let out = Command::new("git").args(args).current_dir(&t.0).output().expect("run git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// A git repository holding a store from `dstack init` whose open run has one Milestone M1.
fn scratch() -> Scratch {
    let t = Scratch::new();
    git(&t, &["init", "-q"]);
    let who = ["-c", "commit.gpgsign=false", "-c", "user.email=t@t", "-c", "user.name=t"];
    git(&t, &[&who[..], &["commit", "-q", "--allow-empty", "-m", "init"]].concat());
    t.init();
    let table = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude/lint/ko-scope.tsv");
    let text = std::fs::read_to_string(table).expect("the shipped scope table");
    t.write(".dstack/project/ko-scope.tsv", &text);
    t.run_fixture("20261002T000000Z_cap", None, true);
    t.ok(&["milestone", "add", "core"]);
    t
}

/// PROJECT.md as `dstack init` wrote it, with its max_concurrent line replaced (or dropped).
fn set_policy(t: &Scratch, line: Option<&str>) {
    let text = t.read(PROJECT);
    let kept: Vec<&str> = text
        .lines()
        .filter_map(|l| match l.starts_with("max_concurrent:") {
            true => line,
            false => Some(l),
        })
        .collect();
    t.write(PROJECT, &format!("{}\n", kept.join("\n")));
}

fn doc(path: &str) -> String {
    let full = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join(path);
    std::fs::read_to_string(full).unwrap_or_else(|e| panic!("{path}: {e}"))
}

#[test]
fn R17_worker_cap_init_writes_max_concurrent_5() {
    let t = scratch();
    let project = t.read(PROJECT);
    assert!(project.lines().any(|l| l == "max_concurrent: 5"), "{project}");
    let next = t.ok(&["next"]);
    assert!(
        next.contains("cap:         5 (PROJECT.md max_concurrent); in-progress 0; free slots 5\n"),
        "{next}"
    );
}

#[test]
fn R17_worker_cap_next_falls_back_to_5_without_a_policy_value() {
    let t = scratch();
    set_policy(&t, None);
    assert!(!t.read(PROJECT).contains("max_concurrent"));
    let next = t.ok(&["next"]);
    assert!(
        next.contains("cap:         5 (default); in-progress 0; free slots 5\n"),
        "{next}"
    );
    assert!(next.contains("schedulable: (none) — 0 of 5 free slot(s)\n"), "{next}");
}

#[test]
fn R17_worker_cap_policy_beats_default_and_max_beats_policy() {
    let t = scratch();
    set_policy(&t, Some("max_concurrent: 2"));
    let next = t.ok(&["next"]);
    assert!(
        next.contains("cap:         2 (PROJECT.md max_concurrent); in-progress 0; free slots 2\n"),
        "{next}"
    );
    let next = t.ok(&["next", "--max", "4"]);
    assert!(next.contains("cap:         4 (--max); in-progress 0; free slots 4\n"), "{next}");
}

#[test]
fn R17_worker_cap_docs_follow_the_project_cap() {
    for path in [RUNTIME, DEVELOP] {
        let text = doc(path);
        assert!(!text.contains("--max 3"), "{path} still hard-codes --max 3");
        assert!(
            text.contains("PROJECT.md `max_concurrent` (default 5)"),
            "{path} does not point dstack next at the PROJECT.md cap"
        );
    }
    let lines = doc(DEVELOP).lines().count();
    assert!(lines <= 300, "{DEVELOP} has {lines} lines");
}
