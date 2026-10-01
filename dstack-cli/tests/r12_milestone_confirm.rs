// tests/r12_milestone_confirm.rs
// R12/R16: milestone brief shows a Milestone's decomposition with visible placeholders, and
// milestone confirm records its Plan ids only when every Plan that has not started carries a
// purpose (and an E2E focus unless e2e is none) and the whole request.md passes lint-ko.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::Output;

use serde_json::{json, Value};
use support::{tree, Scratch};

const RUN: &str = "20261001T000000Z_confirm";
const DIR: &str = ".dstack/runs/20261001T000000Z_confirm";
const CLEAN: &str = "계획을 확인한 뒤에 워커를 띄워요.";

fn request(e2e: &str, background: &str) -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: auto\nreview: on\ncodex_effort: high\ne2e: {e2e}\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\n# 마일스톤 확인 시험\n\n\
         {background}\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n"
    )
}

/// A store whose open run has the given e2e value, with the scope table this checkout ships so
/// lint-ko classifies request.md as the approve-time tests do.
fn scratch(e2e: &str, background: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    let table = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude/lint/ko-scope.tsv");
    let text = std::fs::read_to_string(table).expect("the shipped scope table");
    t.write(".dstack/project/ko-scope.tsv", &text);
    t.run_fixture(RUN, None, true);
    t.write(&format!("{DIR}/request.md"), &request(e2e, background));
    t
}

/// M1 holds a filled P1 (with a task) and a bare P2 that depends on it; M2 holds a bare P3.
fn planned(e2e: &str) -> Scratch {
    let t = scratch(e2e, CLEAN);
    t.ok(&["milestone", "add", "core", "--goal", "설치를 한 번에 끝내요"]);
    t.ok(&["milestone", "add", "wrap"]);
    let first = ["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"];
    t.ok(&[&first[..], &["--purpose", "설치 스크립트를 정리해요", "--e2e-focus", "설치 출력이 그대로예요"]].concat());
    t.ok(&["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh", "--deps", "P1"]);
    t.ok(&["plan", "add", "third", "--milestone", "M2", "--files", "e/f.sh"]);
    let task = ["task", "add", "write-lib", "--plan", "P1", "--covers", "R01", "--files", "a/b.sh"];
    t.ok(&[&task[..], &["--purpose", "라이브러리를 써요"]].concat());
    t
}

fn plan_json(t: &Scratch) -> Value {
    serde_json::from_str(&t.read(&format!("{DIR}/plan.json"))).expect("plan.json parses")
}

fn run_tree(t: &Scratch) -> Vec<(PathBuf, Vec<u8>)> {
    tree(&t.0.join(DIR))
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// STATE.md without the moment it was written.
fn state(t: &Scratch) -> String {
    let text = t.read(&format!("{DIR}/STATE.md"));
    text.lines().filter(|line| !line.starts_with("updated_at:")).collect::<Vec<_>>().join("\n")
}

#[test]
fn R12_milestone_confirm_brief_prints_goal_purposes_focus_and_placeholders() {
    let t = planned("cli");
    t.ok(&["milestone", "add", "empty"]);
    let before = run_tree(&t);
    let out = t.ok(&["milestone", "brief", "M1"]);
    let expected = "milestone M1: core\n\
                    \x20 goal: 설치를 한 번에 끝내요\n\
                    \x20 e2e: cli — every Plan needs an E2E focus before it starts\n\
                    plan P1: first\n\
                    \x20 status:    ready\n\
                    \x20 purpose:   설치 스크립트를 정리해요\n\
                    \x20 e2e focus: 설치 출력이 그대로예요\n\
                    \x20 deps:      (none)\n\
                    \x20 files:     a/b.sh\n\
                    \x20 task T1: write-lib\n\
                    \x20   purpose: 라이브러리를 써요\n\
                    \x20   covers:  R01\n\
                    plan P2: second\n\
                    \x20 status:    pending\n\
                    \x20 purpose:   (none)\n\
                    \x20 e2e focus: (none)\n\
                    \x20 deps:      P1\n\
                    \x20 files:     c/d.sh\n\
                    \x20 tasks:     (none)\n\
                    QA scenarios: (none yet)\n\
                    confirmed: no\n\
                    \x20 confirmed plans: (none)\n\
                    \x20 not confirmed:   P1, P2\n\
                    \x20 next: dstack milestone confirm M1\n";
    assert_eq!(out, expected);
    let out = t.ok(&["milestone", "brief", "M3"]);
    assert!(out.starts_with("milestone M3: empty\n  goal: (none)\n"), "{out}");
    assert!(out.contains("\n  plans: (none)\nQA scenarios: (none yet)\nconfirmed: no\n"), "{out}");
    assert_eq!(run_tree(&t), before, "brief writes nothing");
}

#[test]
fn R12_milestone_confirm_records_the_plan_ids() {
    let t = planned("cli");
    t.ok(&["plan", "edit", "P2", "--purpose", "마무리해요", "--e2e-focus", "보고서를 봐요"]);
    let roadmap = format!("{DIR}/ROADMAP.md");
    let (roadmap_before, state_before) = (t.read(&roadmap), state(&t));
    let out = t.ok(&["milestone", "confirm", "M1"]);
    assert!(out.contains("confirmed milestone M1: core\n  plans: P1, P2\n"), "{out}");
    let doc = plan_json(&t);
    assert_eq!(doc["milestones"][0]["confirmed"], json!(["P1", "P2"]));
    assert!(doc["milestones"][1].get("confirmed").is_none(), "{doc}");
    assert_eq!(t.read(&roadmap), roadmap_before);
    assert_eq!(state(&t), state_before);
    let brief = t.ok(&["milestone", "brief", "M1"]);
    assert!(
        brief.ends_with("confirmed: yes\n  confirmed plans: P1, P2\n  not confirmed:   (none)\n"),
        "{brief}"
    );
}

#[test]
fn R12_milestone_confirm_refusals_name_each_problem_and_write_nothing() {
    let t = planned("cli");
    t.ok(&["milestone", "add", "empty"]);
    let purpose = "P2 has no purpose: dstack plan edit P2 --purpose <text>, then confirm again";
    let focus = "P2 has no E2E focus: dstack plan edit P2 --e2e-focus <text>, then confirm again";
    let refused = |m: &str, n: usize| format!("dstack: refused: milestone {m} is not confirmed — {n} problem(s) above");
    let cases: Vec<(Vec<&str>, Vec<String>)> = vec![
        (vec!["milestone", "confirm", "M1"], vec![purpose.into(), focus.into(), refused("M1", 2)]),
        (
            vec!["milestone", "confirm", "M3"],
            vec!["M3 has no plans: dstack plan add <slug> --milestone M3, then confirm again".into(), refused("M3", 1)],
        ),
        (vec!["milestone", "confirm", "M9"], vec!["dstack: milestone not found: M9 (known: M1 M2 M3)".into()]),
        (vec!["milestone", "brief", "M9"], vec!["dstack: milestone not found: M9 (known: M1 M2 M3)".into()]),
        (vec!["milestone", "confirm"], vec!["dstack: usage: dstack milestone confirm M<n>".into()]),
        (vec!["milestone", "brief"], vec!["dstack: usage: dstack milestone brief M<n>".into()]),
    ];
    for (args, lines) in cases {
        let before = run_tree(&t);
        let out = t.run(&args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stderr(&out));
        assert_eq!(stderr(&out), format!("{}\n", lines.join("\n")), "{args:?}");
        assert_eq!(run_tree(&t), before, "{args:?} wrote into the run");
    }
    // With the purpose filled in, the missing focus alone still refuses.
    t.ok(&["plan", "edit", "P2", "--purpose", "마무리해요"]);
    let before = run_tree(&t);
    let out = t.run(&["milestone", "confirm", "M1"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(stderr(&out), format!("{focus}\n{}\n", refused("M1", 1)));
    assert_eq!(run_tree(&t), before);
}

#[test]
fn R12_milestone_confirm_e2e_none_skips_the_focus_check() {
    let t = planned("none");
    t.ok(&["plan", "edit", "P2", "--purpose", "마무리해요"]);
    let skipped = "  e2e: none — the E2E focus check is skipped\n";
    let brief = t.ok(&["milestone", "brief", "M1"]);
    assert!(brief.contains(skipped), "{brief}");
    let out = t.ok(&["milestone", "confirm", "M1"]);
    assert!(out.contains(skipped), "{out}");
    assert_eq!(plan_json(&t)["milestones"][0]["confirmed"], json!(["P1", "P2"]));
}

#[test]
fn R12_milestone_confirm_a_started_plan_without_purpose_does_not_block() {
    let t = scratch("cli", CLEAN);
    let plan = |id: &str, status: &str, extra: &str| {
        format!(
            r#"{{"id":"{id}","milestone":"M1","slug":"s{id}","files":["{id}.sh"],"deps":[],"status":"{status}","worktree":"","started_at":"","done_at":"","tasks":[]{extra}}}"#
        )
    };
    let filled = r#","purpose":"정리해요","e2e_focus":"출력을 봐요""#;
    let plans = [plan("P1", "done", ""), plan("P2", "in-progress", ""), plan("P3", "ready", filled)];
    let doc = format!(
        r#"{{"v":2,"milestones":[{{"id":"M1","slug":"core","order":1}}],"plans":[{}]}}"#,
        plans.join(",")
    );
    t.write(&format!("{DIR}/plan.json"), &doc);
    t.ok(&["milestone", "confirm", "M1"]);
    assert_eq!(plan_json(&t)["milestones"][0]["confirmed"], json!(["P1", "P2", "P3"]));
}

#[test]
fn R12_milestone_confirm_again_picks_up_a_new_plan() {
    let t = planned("none");
    t.ok(&["plan", "edit", "P2", "--purpose", "마무리해요"]);
    t.ok(&["milestone", "confirm", "M1"]);
    let fourth = ["plan", "add", "fourth", "--milestone", "M1", "--files", "g/h.sh"];
    t.ok(&[&fourth[..], &["--purpose", "덧붙여요"]].concat());
    let brief = t.ok(&["milestone", "brief", "M1"]);
    assert!(
        brief.contains("confirmed: no\n  confirmed plans: P1, P2\n  not confirmed:   P4\n"),
        "{brief}"
    );
    t.ok(&["milestone", "confirm", "M1"]);
    assert_eq!(plan_json(&t)["milestones"][0]["confirmed"], json!(["P1", "P2", "P4"]));
    // The list is replaced, not merged: a Plan removed since drops out of it.
    t.ok(&["plan", "remove", "P2"]);
    t.ok(&["milestone", "confirm", "M1"]);
    assert_eq!(plan_json(&t)["milestones"][0]["confirmed"], json!(["P1", "P4"]));
}

/// One milestone with one filled Plan in a store whose request has the given background.
fn one_plan(background: &str) -> Scratch {
    let t = scratch("cli", background);
    t.ok(&["milestone", "add", "core"]);
    let first = ["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"];
    t.ok(&[&first[..], &["--purpose", "정리해요", "--e2e-focus", "출력을 봐요"]].concat());
    t
}

#[test]
fn R16_milestone_confirm_refuses_a_lint_violation() {
    let t = one_plan("이 파일이 정본이라서 확인한 뒤에는 바뀌면 안 돼요.");
    let before = run_tree(&t);
    let out = t.run(&["milestone", "confirm", "M1"]);
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains("K01 (S1) matched '정본'"), "{}", stdout(&out));
    assert_eq!(
        stderr(&out),
        "request.md has S1 Korean lint hits above: fix the wording, then confirm again\n\
         dstack: refused: milestone M1 is not confirmed — 1 problem(s) above\n"
    );
    assert_eq!(run_tree(&t), before, "a refused confirm writes nothing");
}

#[test]
fn R16_milestone_confirm_passes_a_clean_request() {
    let t = one_plan(CLEAN);
    let out = t.ok(&["milestone", "confirm", "M1"]);
    // Classified and scanned, not passed as unclassified.
    assert!(out.contains("files 1, hits 0 (S1 0), unclassified 0"), "{out}");
    assert_eq!(plan_json(&t)["milestones"][0]["confirmed"], json!(["P1"]));
}
