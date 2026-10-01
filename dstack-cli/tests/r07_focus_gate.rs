// tests/r07_focus_gate.rs
// R07/R12: plan start and next keep a Plan off until its Milestone has confirmed it and, in a run
// whose e2e is not none, until it carries an E2E focus; a run with e2e none says the focus check
// is skipped.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::{Command, Output};

use serde_json::Value;
use support::Scratch;

const RUN: &str = "20261001T000000Z_gate";
const DIR: &str = ".dstack/runs/20261001T000000Z_gate";
const SKIPPED: &str = "e2e: none — the E2E focus check is skipped";

fn request(e2e: &str) -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: auto\nreview: on\ncodex_effort: high\ne2e: {e2e}\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\n# 검증 초점 시험\n\n\
         계획을 확인한 뒤에 워커를 띄워요.\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n"
    )
}

fn git(t: &Scratch, args: &[&str]) -> String {
    let out = Command::new("git").args(args).current_dir(&t.0).output().expect("run git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A git repository holding a store whose open run has the given e2e value (no request.md for
/// None) and one Milestone M1 without Plans.
fn scratch(e2e: Option<&str>) -> Scratch {
    let t = Scratch::new();
    git(&t, &["init", "-q"]);
    let who = ["-c", "commit.gpgsign=false", "-c", "user.email=t@t", "-c", "user.name=t"];
    git(&t, &[&who[..], &["commit", "-q", "--allow-empty", "-m", "init"]].concat());
    t.init();
    let table = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude/lint/ko-scope.tsv");
    let text = std::fs::read_to_string(table).expect("the shipped scope table");
    t.write(".dstack/project/ko-scope.tsv", &text);
    t.run_fixture(RUN, None, true);
    if let Some(e2e) = e2e {
        t.write(&format!("{DIR}/request.md"), &request(e2e));
    }
    t.ok(&["milestone", "add", "core"]);
    t
}

/// plan add <slug> in M1 on its own file, with a purpose and, when given, an E2E focus.
fn add_plan(t: &Scratch, slug: &str, focus: Option<&str>) {
    let file = format!("{slug}/a.sh");
    let mut args = vec!["plan", "add", slug, "--milestone", "M1", "--files", &file, "--purpose", "정리해요"];
    if let Some(focus) = focus {
        args.extend(["--e2e-focus", focus]);
    }
    t.ok(&args);
}

fn plan_json(t: &Scratch) -> String {
    t.read(&format!("{DIR}/plan.json"))
}

fn unconfirmed(p: &str) -> String {
    format!("milestone M1 is not confirmed for {p}: run dstack milestone brief M1, then dstack milestone confirm M1")
}

fn unfocused(p: &str) -> String {
    format!("{p} has no E2E focus: dstack plan edit {p} --e2e-focus <text>, then confirm again")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// plan start that must be refused for `reason`, with plan.json, the branches and the worktree
/// path all as they were.
fn assert_start_refused(t: &Scratch, p: &str, reason: &str) {
    let (before, branches) = (plan_json(t), git(t, &["branch", "--list"]));
    let out = t.run(&["plan", "start", p, "--worktree", "wt"]);
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(&out), stderr(&out));
    assert_eq!(stdout(&out), "");
    assert_eq!(stderr(&out), format!("dstack: refused: {reason}\n"));
    assert_eq!(plan_json(t), before, "a refused start wrote plan.json");
    assert_eq!(git(t, &["branch", "--list"]), branches, "a refused start made a branch");
    assert!(!t.0.join("wt").exists(), "a refused start made the worktree");
}

#[test]
fn R12_focus_gate_refuses_an_unconfirmed_plan_until_confirm() {
    let t = scratch(Some("cli"));
    add_plan(&t, "first", Some("출력을 봐요"));
    assert_start_refused(&t, "P1", &unconfirmed("P1"));
    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("excluded: P1 — {}\nschedulable: (none) — 0 of 3 free slot(s)\n", unconfirmed("P1"))),
        "{next}"
    );

    t.ok(&["milestone", "confirm", "M1"]);
    // Nothing excluded and e2e not none: next prints what it printed before the gate.
    let next = t.ok(&["next"]);
    assert_eq!(
        next,
        "ready:       P1\nin-progress: (none)\noverlaps:\n  (none)\n  overlapping file pairs: 0\n\
         cap:         3 (PROJECT.md max_concurrent); in-progress 0; free slots 3\n\
         schedulable: P1 — 1 of 3 free slot(s)\ncross-run warnings: 0\n"
    );
    let out = t.ok(&["plan", "start", "P1", "--worktree", "wt"]);
    assert!(out.starts_with("plan P1: ready → in-progress at "), "{out}");
    assert!(!out.contains("e2e:"), "{out}");
    assert!(t.0.join("wt").is_dir(), "the worktree");
    assert!(git(&t, &["branch", "--list"]).contains("plan/P1-first"));
}

#[test]
fn R12_focus_gate_excludes_a_plan_added_after_confirm() {
    let t = scratch(Some("cli"));
    add_plan(&t, "first", Some("출력을 봐요"));
    t.ok(&["milestone", "confirm", "M1"]);
    add_plan(&t, "second", Some("보고서를 봐요"));
    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("excluded: P2 — {}\nschedulable: P1 — 1 of 3 free slot(s)\n", unconfirmed("P2"))),
        "{next}"
    );
    assert_start_refused(&t, "P2", &unconfirmed("P2"));

    t.ok(&["milestone", "confirm", "M1"]);
    let next = t.ok(&["next"]);
    assert!(!next.contains("excluded:"), "{next}");
    assert!(next.contains("schedulable: P1 P2 — 2 of 3 free slot(s)\n"), "{next}");
}

#[test]
fn R07_focus_gate_excludes_a_confirmed_plan_whose_focus_was_removed() {
    let t = scratch(Some("cli"));
    add_plan(&t, "first", Some("출력을 봐요"));
    add_plan(&t, "second", Some("보고서를 봐요"));
    t.ok(&["milestone", "confirm", "M1"]);
    // plan edit refuses a blank focus, so the focus is taken out of plan.json by hand.
    let mut doc: Value = serde_json::from_str(&plan_json(&t)).expect("plan.json parses");
    doc["plans"][1].as_object_mut().expect("P2").remove("e2e_focus");
    t.write(&format!("{DIR}/plan.json"), &doc.to_string());

    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("excluded: P2 — {}\nschedulable: P1 — 1 of 3 free slot(s)\n", unfocused("P2"))),
        "{next}"
    );
    assert!(!next.contains(SKIPPED), "{next}");
    assert_start_refused(&t, "P2", &unfocused("P2"));

    t.ok(&["plan", "edit", "P2", "--e2e-focus", "보고서를 봐요"]);
    let next = t.ok(&["next"]);
    assert!(!next.contains("excluded:"), "{next}");
    assert!(next.contains("schedulable: P1 P2 — 2 of 3 free slot(s)\n"), "{next}");
}

#[test]
fn R07_focus_gate_a_run_without_request_md_requires_focus() {
    let t = scratch(None);
    add_plan(&t, "first", Some("출력을 봐요"));
    add_plan(&t, "second", None);
    // milestone confirm needs request.md, so the confirmed list of a legacy run is written by hand.
    let mut doc: Value = serde_json::from_str(&plan_json(&t)).expect("plan.json parses");
    doc["milestones"][0]["confirmed"] = serde_json::json!(["P1", "P2"]);
    t.write(&format!("{DIR}/plan.json"), &doc.to_string());

    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("excluded: P2 — {}\nschedulable: P1 — 1 of 3 free slot(s)\n", unfocused("P2"))),
        "{next}"
    );
    assert_start_refused(&t, "P2", &unfocused("P2"));
    let out = t.ok(&["plan", "start", "P1"]);
    assert!(!out.contains("e2e:"), "{out}");
}

#[test]
fn R07_focus_gate_e2e_none_skips_the_focus_check() {
    let t = scratch(Some("none"));
    add_plan(&t, "first", None);
    t.ok(&["milestone", "confirm", "M1"]);
    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("{SKIPPED}\nschedulable: P1 — 1 of 3 free slot(s)\n")),
        "{next}"
    );
    assert!(!next.contains("excluded:"), "{next}");
    let out = t.ok(&["plan", "start", "P1"]);
    assert!(out.starts_with("plan P1: ready → in-progress at "), "{out}");
    assert!(out.contains(&format!("\n  {SKIPPED}\n")), "{out}");
}

#[test]
fn R12_focus_gate_a_re_minted_id_needs_a_new_confirm() {
    let t = scratch(Some("cli"));
    add_plan(&t, "first", Some("출력을 봐요"));
    t.ok(&["milestone", "confirm", "M1"]);
    t.ok(&["plan", "remove", "P1"]);
    // With no Plan left, plan add mints P1 again for a different Plan.
    let out = t.ok(&["plan", "add", "replacement", "--milestone", "M1", "--files", "replacement/a.sh", "--purpose", "바꿔요", "--e2e-focus", "출력을 봐요"]);
    assert!(out.starts_with("plan P1: replacement (milestone M1)\n"), "{out}");
    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("excluded: P1 — {}\nschedulable: (none) — 0 of 3 free slot(s)\n", unconfirmed("P1"))),
        "{next}"
    );
    assert_start_refused(&t, "P1", &unconfirmed("P1"));

    t.ok(&["milestone", "confirm", "M1"]);
    let out = t.ok(&["plan", "start", "P1", "--worktree", "wt"]);
    assert!(out.starts_with("plan P1: ready → in-progress at "), "{out}");
    assert!(git(&t, &["branch", "--list"]).contains("plan/P1-replacement"));
}

#[test]
fn R12_focus_gate_a_re_minted_decimal_id_needs_a_new_confirm() {
    let t = scratch(Some("cli"));
    add_plan(&t, "first", Some("출력을 봐요"));
    let insert = ["plan", "insert", "inserted", "--after", "P1", "--files", "inserted/a.sh", "--purpose", "끼워요", "--e2e-focus", "보고서를 봐요"];
    t.ok(&insert);
    t.ok(&["milestone", "confirm", "M1"]);
    t.ok(&["plan", "remove", "P1.1"]);
    let out = t.ok(&insert);
    assert!(out.starts_with("plan P1.1: inserted (milestone M1)\n"), "{out}");
    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("excluded: P1.1 — {}\nschedulable: P1 — 1 of 3 free slot(s)\n", unconfirmed("P1.1"))),
        "{next}"
    );
    assert_start_refused(&t, "P1.1", &unconfirmed("P1.1"));
}

#[test]
fn R07_focus_gate_e2e_none_prints_the_skip_line_when_no_plan_is_ready() {
    let t = scratch(Some("none"));
    add_plan(&t, "first", None);
    t.ok(&["milestone", "confirm", "M1"]);
    t.ok(&["plan", "start", "P1"]);
    let next = t.ok(&["next"]);
    assert!(
        next.contains(&format!("{SKIPPED}\nschedulable: (none) — 0 of 2 free slot(s)\n")),
        "{next}"
    );
}

#[test]
fn R07_focus_gate_e2e_none_without_plans_prints_no_skip_line() {
    let t = scratch(Some("none"));
    let next = t.ok(&["next"]);
    assert_eq!(
        next,
        "ready:       (none)\nin-progress: (none)\noverlaps:\n  (none)\n  overlapping file pairs: 0\n\
         cap:         3 (PROJECT.md max_concurrent); in-progress 0; free slots 3\n\
         schedulable: (none) — 0 of 3 free slot(s)\ncross-run warnings: 0\n"
    );
}
