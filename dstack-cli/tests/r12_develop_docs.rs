// tests/r12_develop_docs.rs
// R12 (skill half): dstack-develop plans a Milestone with its goal, Plan purposes and E2E focus,
// shows `dstack milestone brief` and asks once, records `dstack milestone confirm`, and starts no
// worker before that; an excluded Plan is fixed and confirmed again, never worked around.
// R11 (P11 share): the Milestone close builds the runner's cases from `dstack e2e brief
// --milestone`, e2e-runner.md takes its cases from that output and guards its scratch
// directories, and both files stay within the 300-line skill cap.
#![allow(non_snake_case)]

use std::path::PathBuf;

const DEVELOP: &str = "claude/skills/dstack-develop/SKILL.md";
const RUNNER: &str = "claude/agents/e2e-runner.md";

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn read(path: &str) -> String {
    std::fs::read_to_string(repo().join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
}

/// Every phrase `text` must carry, reported together so one run names every gap.
fn assert_names(what: &str, text: &str, phrases: &[&str]) {
    let missing: Vec<&str> = phrases
        .iter()
        .copied()
        .filter(|phrase| !text.contains(phrase))
        .collect();
    assert!(missing.is_empty(), "{what} does not name {missing:?}");
}

/// The body of the `## <title>` section: up to the next `## ` heading or the end.
fn section<'a>(text: &'a str, title: &str) -> &'a str {
    let start = text
        .find(&format!("\n## {title}\n"))
        .unwrap_or_else(|| panic!("no '## {title}' section"));
    let body = &text[start + 1..];
    match body[3..].find("\n## ") {
        Some(end) => &body[..end + 3],
        None => body,
    }
}

fn position(text: &str, phrase: &str) -> usize {
    text.find(phrase)
        .unwrap_or_else(|| panic!("missing {phrase:?}"))
}

#[test]
fn R12_develop_docs_plan_stage_shows_the_brief_then_confirms() {
    let text = read(DEVELOP);
    let stages = section(&text, "2. The five-stage loop, once per Milestone (R61)");
    assert_names(
        "develop §2",
        stages,
        &[
            "dstack milestone add <slug> --goal",
            "--purpose",
            "--e2e-focus",
            "dstack task add",
            "dstack milestone brief M<n>",
            "dstack milestone confirm M<n>",
        ],
    );
    assert!(
        position(stages, "dstack milestone brief M<n>")
            < position(stages, "dstack milestone confirm M<n>"),
        "the brief is shown before the confirmation is recorded"
    );
}

#[test]
fn R12_develop_docs_no_worker_starts_before_confirmation() {
    let text = read(DEVELOP);
    let stages = section(&text, "2. The five-stage loop, once per Milestone (R61)");
    assert_names(
        "develop §2",
        stages,
        &["No worker starts before", "only confirmed Plans run"],
    );
    let waves = section(&text, "5. The wave loop (R66, R38)");
    assert_names(
        "develop §5",
        waves,
        &[
            "dstack plan edit P<n> --purpose <text>",
            "--e2e-focus <text>",
            "dstack milestone edit M<n> --goal <text>",
            "dstack milestone confirm M<n>",
            "never work around",
        ],
    );
}

#[test]
fn R11_develop_docs_milestone_close_builds_cases_from_e2e_brief() {
    let text = read(DEVELOP);
    let close = section(&text, "10. Closing a Milestone and the Goal");
    assert_names("develop §10", close, &["dstack e2e brief --milestone M<n>"]);
}

#[test]
fn R11_develop_docs_runner_takes_cases_from_e2e_brief_and_guards_temp_dirs() {
    assert_names(
        RUNNER,
        &read(RUNNER),
        &[
            "dstack e2e brief",
            "E2E focus",
            "set -euo pipefail",
            "mktemp -d",
            "pwd",
            "git init",
            "dstack init",
        ],
    );
}

#[test]
fn R11_develop_docs_files_stay_within_the_skill_cap() {
    for path in [DEVELOP, RUNNER] {
        let lines = read(path).lines().count();
        assert!(lines <= 300, "{path}: {lines} lines, over 300");
    }
}
