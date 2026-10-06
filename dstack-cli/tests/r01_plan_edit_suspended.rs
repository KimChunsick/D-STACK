#![allow(non_snake_case)]
// A suspended Plan's declaration is editable; resume rechecks it, and an in-progress Plan's
// self-edit refusal points at the suspend → edit → resume path.

#[path = "support/mode_settings.rs"]
mod support;

use std::process::Command;

use dstack_cli::store::plan::{self, PlanDoc, Task};
use support::{tree, Scratch};

const RUN: &str = "20261007T000000Z_edit_suspended";
const DIR: &str = ".dstack/runs/20261007T000000Z_edit_suspended";

fn fixture() -> Scratch {
    let t = Scratch::new();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .args(args)
            .current_dir(&t.0)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q"]);
    git(&[
        "-c",
        "commit.gpgsign=false",
        "-c",
        "user.email=t@t",
        "-c",
        "user.name=t",
        "commit",
        "-q",
        "--allow-empty",
        "-m",
        "init",
    ]);
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(&format!("{DIR}/request.md"), "---\ne2e: none\n---\n");
    t.ok(&["milestone", "add", "core"]);
    t.ok(&["plan", "add", "first", "--milestone", "M1", "--files", "shared/a.rs"]);
    t.ok(&["plan", "add", "second", "--milestone", "M1", "--files", "other/x.rs"]);
    let mut doc = plan::load(&t.0.join(DIR)).unwrap();
    doc.milestones[0].confirmed = vec!["P1".into(), "P2".into()];
    t.write(&format!("{DIR}/plan.json"), &doc.to_json());
    t
}

fn doc(t: &Scratch) -> PlanDoc {
    plan::load(&t.0.join(DIR)).unwrap()
}

fn refused(t: &Scratch, args: &[&str], reasons: &[&str]) {
    let before = tree(&t.0.join(".dstack"));
    let out = t.run(args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(1), "{args:?}: {stderr}");
    for reason in reasons {
        assert!(stderr.contains(reason), "{args:?} lacks {reason}: {stderr}");
    }
    assert_eq!(tree(&t.0.join(".dstack")), before, "{args:?} mutated the store");
}

/// Starts P1 in the scratch checkout, records a committed task on shared/a.rs, then suspends it.
fn suspended_with_task(t: &Scratch) {
    let root = t.0.to_str().unwrap();
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&t.0)
        .output()
        .unwrap();
    let mut saved = doc(t);
    saved.plan_mut("P1").unwrap().tasks.push(Task {
        id: "T1".into(),
        slug: "retained".into(),
        files: vec!["shared/a.rs".into()],
        commit: String::from_utf8(sha.stdout).unwrap().trim().to_string(),
        done_at: "2026-10-07T00:00:00Z".into(),
        ..Task::default()
    });
    t.write(&format!("{DIR}/plan.json"), &saved.to_json());
    t.ok(&["plan", "edit", "P1", "--suspend", "--reason", "review cap", "--worker-stopped"]);
}

#[test]
fn R01_suspended_plan_declaration_is_editable_and_resume_rechecks_it() {
    let t = fixture();
    let root = t.0.to_str().unwrap();
    suspended_with_task(&t);
    let record = format!("{DIR}/suspensions/P1.json");
    let sidecar = t.read(&record);

    t.ok(&["plan", "edit", "P1", "--files", "shared/a.rs,shared/b.rs"]);
    t.ok(&["plan", "edit", "P1", "--slug", "renamed", "--purpose", "widened scope"]);
    t.ok(&["plan", "edit", "P1", "--e2e-focus", "edit while suspended"]);
    t.ok(&["plan", "edit", "P1", "--deps", "P2"]);
    let edited = doc(&t);
    let p1 = edited.plan("P1").unwrap();
    assert_eq!(p1.status, "suspended");
    assert_eq!(p1.files, vec!["shared/a.rs", "shared/b.rs"]);
    assert_eq!(p1.slug, "renamed");
    assert_eq!(p1.purpose, "widened scope");
    assert_eq!(p1.e2e_focus, "edit while suspended");
    assert_eq!(p1.deps, vec!["P2"]);
    assert_eq!(t.read(&record), sidecar, "the edit rewrote the suspension record");
    refused(&t, &["plan", "start", "P1", "--resume", "--confirm"], &["unfinished dependencies"]);

    t.ok(&["plan", "edit", "P1", "--deps", ""]);
    assert!(doc(&t).plan("P1").unwrap().deps.is_empty());
    assert_eq!(t.read(&record), sidecar);
    t.ok(&["plan", "start", "P1", "--resume", "--confirm"]);
    let resumed = doc(&t);
    assert_eq!(resumed.field("P1", "status"), "in-progress");
    assert_eq!(resumed.field("P1", "worktree"), root);
    assert_eq!(resumed.plan("P1").unwrap().files, vec!["shared/a.rs", "shared/b.rs"]);
}

#[test]
fn R02_narrowing_below_task_files_is_refused_without_mutation() {
    let t = fixture();
    suspended_with_task(&t);
    refused(
        &t,
        &["plan", "edit", "P1", "--files", "shared/c.rs"],
        &["outside the new --files"],
    );
    assert_eq!(doc(&t).field("P1", "status"), "suspended");
}

#[test]
fn R03_in_progress_self_edit_points_at_suspend() {
    let t = fixture();
    let root = t.0.to_str().unwrap();
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    refused(
        &t,
        &["plan", "edit", "P1", "--files", "shared/a.rs,shared/b.rs"],
        &["--suspend", "--worker-stopped", "--resume --confirm"],
    );
    assert_eq!(doc(&t).field("P1", "status"), "in-progress");
}

#[test]
fn R04_resume_refuses_when_widened_files_overlap_an_in_progress_plan() {
    let t = fixture();
    let root = t.0.to_str().unwrap();
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    t.ok(&["plan", "start", "P2", "--worktree", root]);
    t.ok(&["plan", "edit", "P1", "--suspend", "--reason", "shift", "--worker-stopped"]);
    t.ok(&["plan", "edit", "P1", "--files", "shared/a.rs,other/x.rs"]);
    assert_eq!(doc(&t).field("P1", "status"), "suspended");
    refused(&t, &["plan", "start", "P1", "--resume", "--confirm"], &["overlap"]);
    assert_eq!(doc(&t).field("P1", "status"), "suspended");
    assert_eq!(doc(&t).field("P2", "status"), "in-progress");
}
