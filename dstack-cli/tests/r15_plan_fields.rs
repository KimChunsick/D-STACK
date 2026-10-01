// tests/r15_plan_fields.rs
// R15/R07: plan.json stores a milestone goal, a plan purpose and E2E focus and a task purpose;
// an older plan.json loads and writes back byte for byte, and ROADMAP.md never shows the values.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::Output;

use serde_json::Value;
use support::{tree, Scratch};

const RUN: &str = "20260930T000000Z_fields";
const DIR: &str = ".dstack/runs/20260930T000000Z_fields";

const REQUEST: &str = "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\n\
risk_axes: none\ndesign_review: auto\nreview: on\ncodex_effort: high\ne2e: cli\n\
unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\n# 계획 필드 시험\n\n\
- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n";

/// Every value a new flag refuses besides a missing one: empty, blank, a newline and other
/// control characters.
const BAD: [&str; 5] = ["", "   ", "두 줄\n이에요", "탭\t이에요", "벨\u{7}이에요"];

/// A plan.json written before the new fields existed, in the jq layout the store keeps.
const LEGACY: &str = r#"{
  "v": 2,
  "milestones": [
    {
      "id": "M1",
      "slug": "core",
      "order": 1
    }
  ],
  "plans": [
    {
      "id": "P1",
      "milestone": "M1",
      "slug": "alpha",
      "files": [
        "a/b.sh"
      ],
      "deps": [],
      "status": "ready",
      "worktree": "",
      "started_at": "",
      "done_at": "",
      "tasks": [
        {
          "id": "T1",
          "slug": "first",
          "covers": [
            "R01"
          ],
          "files": [
            "a/b.sh"
          ],
          "deps": [],
          "commit": "",
          "done_at": ""
        }
      ]
    }
  ]
}
"#;

/// The same ledger with every new field filled, each after the fields that were already there.
const FILLED: &str = r#"{
  "v": 2,
  "milestones": [
    {
      "id": "M1",
      "slug": "core",
      "order": 1,
      "goal": "설치를 한 번에 끝내요",
      "confirmed": [
        "P1"
      ]
    }
  ],
  "plans": [
    {
      "id": "P1",
      "milestone": "M1",
      "slug": "alpha",
      "files": [
        "a/b.sh"
      ],
      "deps": [],
      "status": "ready",
      "worktree": "",
      "started_at": "",
      "done_at": "",
      "tasks": [
        {
          "id": "T1",
          "slug": "first",
          "covers": [
            "R01"
          ],
          "files": [
            "a/b.sh"
          ],
          "deps": [],
          "commit": "",
          "done_at": "",
          "purpose": "스크립트를 써요"
        }
      ],
      "purpose": "설치 스크립트를 정리해요",
      "e2e_focus": "설치 출력이 그대로예요"
    }
  ]
}
"#;

/// A store with one open run named by CURRENT, whose request has a live R01 for task add.
fn scratch() -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(&format!("{DIR}/request.md"), REQUEST);
    t
}

/// The same store with one milestone and one plan already in it.
fn planned() -> Scratch {
    let t = scratch();
    t.ok(&["milestone", "add", "core"]);
    t.ok(&["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"]);
    t
}

fn plan_json(t: &Scratch) -> Value {
    serde_json::from_str(&t.read(&format!("{DIR}/plan.json"))).expect("plan.json parses")
}

fn run_tree(t: &Scratch) -> Vec<(PathBuf, Vec<u8>)> {
    tree(&t.0.join(DIR))
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// One refused call: exit 1, and every file of the run exactly as it was.
fn assert_refused(t: &Scratch, args: &[&str]) {
    let before = run_tree(t);
    let out = t.run(args);
    assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stderr(&out));
    assert_eq!(run_tree(t), before, "{args:?} wrote into the run");
}

/// Every refusal of one flag: nothing behind it, `--flag=`, and each bad value.
fn assert_value_refusals(t: &Scratch, args: &[&str], flag: &str) {
    let empty = format!("{flag}=");
    assert_refused(t, &[args, &[flag]].concat());
    assert_refused(t, &[args, &[empty.as_str()]].concat());
    for bad in BAD {
        assert_refused(t, &[args, &[flag, bad]].concat());
    }
}

#[test]
fn R15_plan_fields_milestone_goal_is_stored_and_edited() {
    let t = scratch();
    let out = t.ok(&["milestone", "add", "core", "--goal", "설치를 한 번에 끝내요"]);
    assert!(out.starts_with("milestone M1: core\n"), "{out}");
    t.ok(&["milestone", "add", "wrap"]);
    let doc = plan_json(&t);
    assert_eq!(doc["milestones"][0]["goal"], "설치를 한 번에 끝내요");
    assert!(doc["milestones"][1].get("goal").is_none(), "{doc}");

    let out = t.ok(&["milestone", "edit", "M2", "--goal=마무리를 점검해요"]);
    assert!(out.starts_with("edited milestone M2: wrap\n"), "{out}");
    t.ok(&["milestone", "edit", "M1", "--goal", "바뀐 목표예요"]);
    let doc = plan_json(&t);
    assert_eq!(doc["milestones"][0]["goal"], "바뀐 목표예요");
    assert_eq!(doc["milestones"][1]["goal"], "마무리를 점검해요");
    assert_eq!(doc["milestones"][1]["slug"], "wrap");
    assert_eq!(doc["milestones"][1]["order"], 2);
}

#[test]
fn R15_plan_fields_milestone_edit_refuses_what_it_cannot_edit() {
    let t = scratch();
    let out = t.run(&["milestone", "edit", "M1", "--goal", "목표예요"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("no plan.json"), "{}", stderr(&out));

    t.ok(&["milestone", "add", "core"]);
    t.ok(&["milestone", "add", "wrap"]);
    let before = run_tree(&t);
    for (args, message) in [
        (
            vec!["milestone", "edit", "M9", "--goal", "목표예요"],
            "dstack: milestone not found: M9 (known: M1 M2)\n",
        ),
        (
            vec!["milestone", "edit", "M1"],
            "dstack: nothing to edit: pass --goal\n",
        ),
        (
            vec!["milestone", "edit"],
            "dstack: usage: dstack milestone edit M<n> --goal <text>\n",
        ),
        (
            vec!["milestone", "edit", "M1", "--slug", "other"],
            "dstack: unknown option: --slug (usage: dstack milestone edit M<n> --goal <text>)\n",
        ),
    ] {
        let out = t.run(&args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert_eq!(stderr(&out), message, "{args:?}");
    }
    assert_eq!(run_tree(&t), before);
}

#[test]
fn R15_plan_fields_milestone_goal_refuses_bad_values() {
    let t = scratch();
    // Refused before the first milestone creates plan.json, so nothing appears at all.
    assert_value_refusals(&t, &["milestone", "add", "core"], "--goal");
    t.ok(&["milestone", "add", "core"]);
    assert_value_refusals(&t, &["milestone", "add", "wrap"], "--goal");
    assert_value_refusals(&t, &["milestone", "edit", "M1"], "--goal");
}

#[test]
fn R15_plan_fields_plan_purpose_is_stored_and_edited() {
    let t = scratch();
    t.ok(&["milestone", "add", "core"]);
    let first = ["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"];
    t.ok(&[&first[..], &["--purpose", "설치 스크립트를 정리해요"]].concat());
    t.ok(&["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh"]);
    t.ok(&["plan", "insert", "between", "--after", "P1", "--files", "e/f.sh", "--purpose=사이에 끼워요"]);
    let doc = plan_json(&t);
    let ids: Vec<&str> = doc["plans"].as_array().unwrap().iter().map(|p| p["id"].as_str().unwrap()).collect();
    assert_eq!(ids, ["P1", "P1.1", "P2"]);
    assert_eq!(doc["plans"][0]["purpose"], "설치 스크립트를 정리해요");
    assert_eq!(doc["plans"][1]["purpose"], "사이에 끼워요");
    assert!(doc["plans"][2].get("purpose").is_none(), "{doc}");

    // A purpose alone is something to edit, and it changes nothing else of the plan.
    t.ok(&["plan", "edit", "P2", "--purpose", "나중에 정한 목적이에요"]);
    t.ok(&["plan", "edit", "P1", "--purpose", "바꾼 목적이에요"]);
    let doc = plan_json(&t);
    assert_eq!(doc["plans"][0]["purpose"], "바꾼 목적이에요");
    assert_eq!(doc["plans"][0]["slug"], "first");
    assert_eq!(doc["plans"][0]["files"], serde_json::json!(["a/b.sh"]));
    assert_eq!(doc["plans"][2]["purpose"], "나중에 정한 목적이에요");

    let out = t.run(&["plan", "edit", "P1"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        stderr(&out),
        "dstack: nothing to edit: pass --slug, --files, --deps, --purpose or --e2e-focus\n"
    );
}

#[test]
fn R15_plan_fields_plan_purpose_refuses_bad_values() {
    let t = planned();
    let second = ["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh"];
    assert_value_refusals(&t, &second, "--purpose");
    let between = ["plan", "insert", "between", "--after", "P1", "--files", "e/f.sh"];
    assert_value_refusals(&t, &between, "--purpose");
    assert_value_refusals(&t, &["plan", "edit", "P1"], "--purpose");
}

#[test]
fn R15_plan_fields_plan_edit_keeps_the_done_gate() {
    let t = scratch();
    let done = LEGACY.replace("\"status\": \"ready\"", "\"status\": \"done\"");
    t.write(&format!("{DIR}/plan.json"), &done);
    for flag in ["--purpose", "--e2e-focus"] {
        let before = run_tree(&t);
        let out = t.run(&["plan", "edit", "P1", flag, "늦은 값이에요"]);
        assert_eq!(out.status.code(), Some(1));
        assert!(stderr(&out).starts_with("dstack: refused: P1 is done"), "{}", stderr(&out));
        assert_eq!(run_tree(&t), before);
    }
}

#[test]
fn R15_plan_fields_task_purpose_is_stored() {
    let t = planned();
    let task = ["task", "add", "write-lib", "--plan", "P1", "--covers", "R01", "--files", "a/b.sh"];
    t.ok(&[&task[..], &["--purpose", "라이브러리를 써요"]].concat());
    t.ok(&["task", "add", "plain", "--plan", "P1", "--covers", "R01", "--files", "a/b.sh"]);
    let doc = plan_json(&t);
    assert_eq!(doc["plans"][0]["tasks"][0]["purpose"], "라이브러리를 써요");
    assert!(doc["plans"][0]["tasks"][1].get("purpose").is_none(), "{doc}");
    let third = ["task", "add", "third", "--plan", "P1", "--covers", "R01", "--files", "a/b.sh"];
    assert_value_refusals(&t, &third, "--purpose");
}

#[test]
fn R15_plan_fields_legacy_plan_json_loads_and_writes_back_unchanged() {
    let t = scratch();
    let file = format!("{DIR}/plan.json");
    t.write(&file, LEGACY);
    t.ok(&["plan", "render"]);
    // A mutation that sets none of the new values rewrites the older file byte for byte.
    t.ok(&["plan", "edit", "P1", "--slug", "alpha"]);
    assert_eq!(t.read(&file), LEGACY);
    t.ok(&["milestone", "add", "wrap"]);
    t.ok(&["plan", "add", "beta", "--milestone", "M2", "--files", "c/d.sh"]);
    let text = t.read(&file);
    for key in ["\"goal\"", "\"confirmed\"", "\"purpose\"", "\"e2e_focus\""] {
        assert!(!text.contains(key), "{key} appeared:\n{text}");
    }
    // Filled values, the confirmed list P7 only stores among them, survive the same rewrite.
    t.write(&file, FILLED);
    t.ok(&["plan", "edit", "P1", "--slug", "alpha"]);
    assert_eq!(t.read(&file), FILLED);
}

/// One ledger built twice: with every new value, and with none of them.
fn build(t: &Scratch, values: bool) {
    let steps: [(&[&str], &[&str]); 6] = [
        (&["milestone", "add", "core"], &["--goal", "설치를 끝내요"]),
        (&["milestone", "add", "wrap"], &[]),
        (
            &["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"],
            &["--purpose", "정리해요", "--e2e-focus", "출력을 봐요"],
        ),
        (
            &["plan", "add", "second", "--milestone", "M2", "--files", "c/d.sh", "--deps", "P1"],
            &[],
        ),
        (
            &["task", "add", "write-lib", "--plan", "P1", "--covers", "R01", "--files", "a/b.sh"],
            &["--purpose", "라이브러리를 써요"],
        ),
        (
            &["plan", "edit", "P2", "--slug", "second"],
            &["--purpose", "마무리해요", "--e2e-focus", "보고서를 봐요"],
        ),
    ];
    for (args, extra) in steps {
        match values {
            true => t.ok(&[args, extra].concat()),
            false => t.ok(args),
        };
    }
}

#[test]
fn R15_plan_fields_roadmap_and_state_do_not_show_the_new_values() {
    let (plain, filled) = (scratch(), scratch());
    build(&plain, false);
    build(&filled, true);
    let roadmap = format!("{DIR}/ROADMAP.md");
    assert_eq!(plain.read(&roadmap), filled.read(&roadmap));
    // STATE.md differs only in the moment it was written.
    let state = |t: &Scratch| -> String {
        let text = t.read(&format!("{DIR}/STATE.md"));
        text.lines().filter(|line| !line.starts_with("updated_at:")).collect::<Vec<_>>().join("\n")
    };
    assert_eq!(state(&plain), state(&filled));
    // The table plan render prints is the same; only the scratch path it names differs.
    let render = |t: &Scratch| t.ok(&["plan", "render"]).replace(&*t.0.to_string_lossy(), "<root>");
    assert_eq!(render(&plain), render(&filled));
    assert_eq!(plain.read(&roadmap), filled.read(&roadmap));
}

#[test]
fn R07_plan_fields_e2e_focus_is_stored_by_plan_add_insert_and_edit() {
    let t = scratch();
    t.ok(&["milestone", "add", "core"]);
    let first = ["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"];
    t.ok(&[&first[..], &["--e2e-focus", "설치 출력이 그대로예요"]].concat());
    t.ok(&["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh"]);
    t.ok(&["plan", "insert", "between", "--after", "P1", "--files", "e/f.sh", "--e2e-focus=사이 단계 출력을 봐요"]);
    let doc = plan_json(&t);
    assert_eq!(doc["plans"][0]["e2e_focus"], "설치 출력이 그대로예요");
    assert_eq!(doc["plans"][1]["e2e_focus"], "사이 단계 출력을 봐요");
    assert!(doc["plans"][2].get("e2e_focus").is_none(), "{doc}");

    t.ok(&["plan", "edit", "P2", "--e2e-focus", "나중에 정한 초점이에요"]);
    t.ok(&["plan", "edit", "P1", "--purpose", "목적이에요", "--e2e-focus", "바꾼 초점이에요"]);
    let doc = plan_json(&t);
    assert_eq!(doc["plans"][0]["e2e_focus"], "바꾼 초점이에요");
    assert_eq!(doc["plans"][0]["purpose"], "목적이에요");
    assert_eq!(doc["plans"][2]["e2e_focus"], "나중에 정한 초점이에요");
    assert!(doc["plans"][2].get("purpose").is_none(), "{doc}");
}

#[test]
fn R07_plan_fields_e2e_focus_refuses_bad_values() {
    let t = planned();
    let second = ["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh"];
    assert_value_refusals(&t, &second, "--e2e-focus");
    let between = ["plan", "insert", "between", "--after", "P1", "--files", "e/f.sh"];
    assert_value_refusals(&t, &between, "--e2e-focus");
    assert_value_refusals(&t, &["plan", "edit", "P1"], "--e2e-focus");
}
