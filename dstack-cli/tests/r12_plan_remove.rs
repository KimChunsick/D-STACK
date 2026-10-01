// tests/r12_plan_remove.rs
// R12 (D-36): plan remove drops the removed id from every milestone's confirmed list, so a Plan
// that later reuses the id does not inherit the confirmation; its refusals still write nothing.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;

use serde_json::{json, Value};
use support::{tree, Scratch};

const RUN: &str = "20260930T000000Z_remove";
const DIR: &str = ".dstack/runs/20260930T000000Z_remove";

/// A store with one open run, two milestones and three plans: P1 and P2 in M1, P3 in M2.
fn planned() -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.ok(&["milestone", "add", "core"]);
    t.ok(&["milestone", "add", "wrap"]);
    t.ok(&["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"]);
    t.ok(&["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh"]);
    t.ok(&["plan", "add", "third", "--milestone", "M2", "--files", "e/f.sh"]);
    t
}

fn plan_json(t: &Scratch) -> Value {
    serde_json::from_str(&t.read(&format!("{DIR}/plan.json"))).expect("plan.json parses")
}

/// milestone confirm is not on this branch, so the test writes the confirmed lists (and any
/// other setup) into plan.json directly.
fn set_up(t: &Scratch, change: impl FnOnce(&mut Value)) {
    let mut doc = plan_json(t);
    change(&mut doc);
    let mut text = serde_json::to_string_pretty(&doc).unwrap();
    text.push('\n');
    t.write(&format!("{DIR}/plan.json"), &text);
}

fn confirm(t: &Scratch, m1: &[&str], m2: &[&str]) {
    set_up(t, |doc| {
        doc["milestones"][0]["confirmed"] = json!(m1);
        doc["milestones"][1]["confirmed"] = json!(m2);
    });
}

fn confirmed(t: &Scratch, milestone: usize) -> Value {
    plan_json(t)["milestones"][milestone]
        .get("confirmed")
        .cloned()
        .unwrap_or(Value::Null)
}

fn run_tree(t: &Scratch) -> Vec<(PathBuf, Vec<u8>)> {
    tree(&t.0.join(DIR))
}

#[test]
fn R12_plan_remove_drops_the_id_from_the_confirmed_list() {
    let t = planned();
    confirm(&t, &["P1", "P2"], &["P3"]);
    let out = t.ok(&["plan", "remove", "P2"]);
    assert!(out.starts_with("removed plan P2 (was ready)\n"), "{out}");
    assert_eq!(confirmed(&t, 0), json!(["P1"]));
    assert_eq!(confirmed(&t, 1), json!(["P3"]));

    // The last confirmed id of a milestone leaves no empty list behind.
    t.ok(&["plan", "remove", "P3"]);
    assert_eq!(confirmed(&t, 0), json!(["P1"]));
    assert!(plan_json(&t)["milestones"][1].get("confirmed").is_none());
}

#[test]
fn R12_plan_remove_a_re_minted_id_is_not_confirmed() {
    let t = planned();
    confirm(&t, &["P1", "P2"], &["P3"]);
    t.ok(&["plan", "remove", "P3"]);
    let out = t.ok(&["plan", "add", "again", "--milestone", "M2", "--files", "g/h.sh"]);
    assert!(out.starts_with("plan P3: again"), "{out}");
    assert_eq!(confirmed(&t, 0), json!(["P1", "P2"]));
    assert_eq!(confirmed(&t, 1), Value::Null);
}

#[test]
fn R12_plan_remove_an_unconfirmed_plan_leaves_the_list_unchanged() {
    let t = planned();
    confirm(&t, &["P1"], &[]);
    t.ok(&["plan", "remove", "P2"]);
    assert_eq!(confirmed(&t, 0), json!(["P1"]));
    assert!(plan_json(&t)["milestones"][1].get("confirmed").is_none());
}

#[test]
fn R12_plan_remove_refusals_still_write_nothing() {
    let t = planned();
    set_up(&t, |doc| {
        doc["milestones"][0]["confirmed"] = json!(["P1", "P2"]);
        doc["milestones"][1]["confirmed"] = json!(["P3"]);
        doc["plans"][0]["status"] = json!("in-progress");
        doc["plans"][2]["deps"] = json!(["P2"]);
    });
    for (p, refusal) in [
        ("P1", "dstack: refused: P1 is in-progress"),
        ("P2", "dstack: refused: these plans depend on P2 (P3)"),
    ] {
        let before = run_tree(&t);
        let out = t.run(&["plan", "remove", p]);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(out.status.code(), Some(1), "{p}: {stderr}");
        assert!(stderr.starts_with(refusal), "{p}: {stderr}");
        assert_eq!(run_tree(&t), before, "plan remove {p} wrote into the run");
    }
}
