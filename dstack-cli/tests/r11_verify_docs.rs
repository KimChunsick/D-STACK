// tests/r11_verify_docs.rs
// R11 (P14 share): dstack-verify builds the milestone runner's cases from `dstack e2e brief
// --milestone`, runs the Goal QA at Goal close (qa add with its three labels, `dstack e2e brief
// --goal`, evidence add --qa), names verify's Goal QA refusals and when they apply, drops the old
// "Goal close records no new evidence" sentence and stays within the 300-line skill cap;
// runtime.md says verify also checks Goal QA. The pointers: dstack-develop §10 and dstack-workflow
// send Goal close to dstack-verify §7, e2e-runner.md takes a Goal QA brief, and the help roster
// names --goal, --qa, the Goal QA check and the QA table. After round 041 (D-46, D-47): the runner
// runs only the open QA and never overwrites an artifact, §5 and §7 reopen a QA result with
// `evidence retire --qa`, and dstack-verify and dstack-develop say a milestone-close verify passes
// before the Goal QA, which the last milestone close runs, and that run close checks --at-close.
#![allow(non_snake_case)]

use std::path::PathBuf;
use std::process::Command;

const VERIFY: &str = "claude/skills/dstack-verify/SKILL.md";
const RUNTIME: &str = "claude/runtime.md";
const DEVELOP: &str = "claude/skills/dstack-develop/SKILL.md";
const WORKFLOW: &str = "claude/skills/dstack-workflow/SKILL.md";
const RUNNER: &str = "claude/agents/e2e-runner.md";
const RECEIPT: &str = "Compact receipt: location/HEAD; R outcomes; changed files/commit; commands/exits; artifact paths; blockers/skips. Raw logs stay in artifacts.";
const OLD_CLOSE: &str = "records no new evidence";

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
fn R11_verify_docs_milestone_and_goal_briefs_come_from_e2e_brief() {
    let text = read(VERIFY);
    let rhythm = section(&text, "1. When this runs");
    assert_names(
        "verify §1",
        rhythm,
        &["dstack e2e brief --milestone M<n>", "Goal QA", "dstack e2e brief --goal"],
    );
    let brief = section(&text, "4. Running the cases — delegate to the native e2e-runner");
    assert_names(
        "verify §4",
        brief,
        &[
            "dstack e2e brief --milestone M<n>",
            "dstack e2e brief --goal",
            "set -euo pipefail",
            "mktemp -d",
            RECEIPT,
        ],
    );
    assert!(
        !brief.contains("| R03 | c1 |"),
        "verify §4 still carries a hand-written case table"
    );
}

#[test]
fn R11_verify_docs_records_goal_qa_scenarios_and_results() {
    let text = read(VERIFY);
    let record = section(&text, "5. Recording evidence (R104)");
    assert_names(
        "verify §5",
        record,
        &[
            "dstack qa add --scenario S<n>|none --from <file>",
            "준비:",
            "단계:",
            "기대 결과:",
            "dstack evidence add --qa QA<n>",
            "--status",
            "--note",
            "never overwritten",
        ],
    );
}

#[test]
fn R11_verify_docs_verify_refuses_goal_close_without_qa_results() {
    let text = read(VERIFY);
    let verify = section(&text, "6. verify, accept-abstain, report, metrics");
    assert_names(
        "verify §6",
        verify,
        &[
            "Goal QA",
            "### S<n>",
            "## 사용 시나리오",
            "| QA | scenario | status | reason | artifact |",
            "skipped",
            "blocked",
        ],
    );
}

#[test]
fn R11_verify_docs_goal_close_runs_the_goal_qa_in_order() {
    let text = read(VERIFY);
    assert!(!text.contains(OLD_CLOSE), "{VERIFY} still says Goal close {OLD_CLOSE}");
    let close = section(&text, "7. Milestone and Goal close checklist");
    let steps = [
        "dstack qa add",
        "dstack e2e brief --goal",
        "dstack evidence add --qa",
        "dstack run close",
    ];
    assert_names("verify §7", close, &steps);
    for pair in steps.windows(2) {
        assert!(
            position(close, pair[0]) < position(close, pair[1]),
            "verify §7 names {} before {}",
            pair[0],
            pair[1]
        );
    }
}

#[test]
fn R11_verify_docs_runtime_names_the_goal_qa_check() {
    assert_names(RUNTIME, &read(RUNTIME), &["`dstack verify` also checks Goal QA"]);
}

#[test]
fn R11_verify_docs_skill_stays_within_the_cap() {
    let lines = read(VERIFY).lines().count();
    assert!(lines <= 300, "{VERIFY}: {lines} lines, over 300");
}

#[test]
fn R11_verify_docs_develop_sends_goal_close_to_verify() {
    let text = read(DEVELOP);
    assert!(!text.contains(OLD_CLOSE), "{DEVELOP} still says Goal close {OLD_CLOSE}");
    let close = section(&text, "10. Closing a Milestone and the Goal");
    assert_names("develop §10", close, &["Goal QA", "dstack-verify §7"]);
}

#[test]
fn R11_verify_docs_workflow_points_at_the_goal_qa() {
    assert_names(WORKFLOW, &read(WORKFLOW), &["Goal QA", "dstack-verify §7"]);
}

#[test]
fn R11_verify_docs_runner_takes_a_goal_qa_brief() {
    assert_names(
        RUNNER,
        &read(RUNNER),
        &[
            "dstack e2e brief --goal",
            "준비:",
            "단계:",
            "기대 결과:",
            "QA id",
            "never edit",
            "failed",
            RECEIPT,
        ],
    );
}

#[test]
fn R11_verify_docs_runner_runs_only_the_open_qa_and_keeps_artifacts() {
    let runner = read(RUNNER);
    let text = read(VERIFY);
    let brief = section(&text, "4. Running the cases — delegate to the native e2e-runner");
    for (what, body) in [(RUNNER, runner.as_str()), ("verify §4", brief)] {
        assert_names(
            what,
            body,
            &[
                "## Open QA scenarios",
                "never rerun a recorded QA",
                "never overwrite an existing file",
                "QA<n>-2.txt",
            ],
        );
    }
}

#[test]
fn R11_verify_docs_retire_reopens_a_qa_result() {
    let text = read(VERIFY);
    let record = section(&text, "5. Recording evidence (R104)");
    assert_names(
        "verify §5",
        record,
        &[
            "dstack evidence retire --qa QA<n> --why \"<reason>\"",
            "qa-history.tsv",
            "rerun only that QA",
            "interrupted retire",
            "repeating",
        ],
    );
    let close = section(&text, "7. Milestone and Goal close checklist");
    assert_names("verify §7", close, &["dstack evidence retire --qa QA<n>"]);
}

#[test]
fn R11_verify_docs_milestone_close_passes_before_the_goal_qa() {
    let text = read(VERIFY);
    for (what, title) in [
        ("verify §1", "1. When this runs"),
        ("verify §7", "7. Milestone and Goal close checklist"),
    ] {
        assert_names(
            what,
            section(&text, title),
            &["last milestone close", "every Plan is done", "--at-close"],
        );
    }
    assert_names(
        "verify §6",
        section(&text, "6. verify, accept-abstain, report, metrics"),
        &["while later Plans remain", "dstack verify --at-close", "dstack run close"],
    );
    let develop = read(DEVELOP);
    assert_names(
        "develop §10",
        section(&develop, "10. Closing a Milestone and the Goal"),
        &[
            "while later Plans remain",
            "last milestone close",
            "every Plan is done",
            "--at-close",
        ],
    );
}

#[test]
fn R11_verify_docs_help_names_the_goal_qa_options() {
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .arg("help")
        .current_dir(repo())
        .output()
        .expect("run dstack help");
    let help = String::from_utf8_lossy(&out.stdout);
    let line = |verb: &str| {
        help.lines()
            .find(|line| line.starts_with(&format!("  {verb} ")))
            .unwrap_or_else(|| panic!("no help line for {verb}"))
            .to_string()
    };
    for (verb, phrase) in [
        ("e2e brief", "--goal"),
        ("evidence add", "--qa"),
        ("verify", "Goal QA"),
        ("verify", "--at-close"),
        ("evidence retire", "--qa"),
        ("report", "QA table"),
    ] {
        assert!(line(verb).contains(phrase), "help line of {verb} does not name {phrase:?}");
    }
}

#[test]
fn R11_verify_docs_pointer_files_stay_within_the_skill_cap() {
    for path in [DEVELOP, WORKFLOW, RUNNER] {
        let lines = read(path).lines().count();
        assert!(lines <= 300, "{path}: {lines} lines, over 300");
    }
}
