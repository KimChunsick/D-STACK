#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};

use dstack_cli::store::plan::{self, PlanDoc, Task};
use support::{tree, Scratch};

const RUN: &str = "20261004T000000Z_suspend";
const DIR: &str = ".dstack/runs/20261004T000000Z_suspend";

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
    t.ok(&[
        "plan",
        "add",
        "first",
        "--milestone",
        "M1",
        "--files",
        "shared/a.rs",
    ]);
    t.ok(&[
        "plan",
        "add",
        "second",
        "--milestone",
        "M1",
        "--files",
        "shared",
    ]);
    let mut doc = plan::load(&t.0.join(DIR)).unwrap();
    doc.milestones[0].confirmed = vec!["P1".into(), "P2".into()];
    t.write(&format!("{DIR}/plan.json"), &doc.to_json());
    t
}

fn doc(t: &Scratch) -> PlanDoc {
    plan::load(&t.0.join(DIR)).unwrap()
}

fn refused(t: &Scratch, args: &[&str], reason: &str) {
    let before = tree(&t.0.join(".dstack"));
    let out = t.run(args);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains(reason),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        tree(&t.0.join(".dstack")),
        before,
        "{args:?} mutated the store"
    );
}

fn undecidable(t: &Scratch, args: &[&str], reason: &str) {
    let before = tree(&t.0.join(".dstack"));
    let out = t.run(args);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains(reason));
    assert_eq!(tree(&t.0.join(".dstack")), before);
}

#[test]
fn R24_suspend_releases_only_active_lock_and_resume_reacquires_it() {
    let t = fixture();
    let root = t.0.to_str().unwrap();
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    let sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(&t.0)
        .output()
        .unwrap();
    let sha = String::from_utf8(sha.stdout).unwrap().trim().to_string();
    let mut saved = doc(&t);
    saved.plan_mut("P1").unwrap().tasks.push(Task {
        id: "T1".into(),
        slug: "retained".into(),
        commit: sha.clone(),
        done_at: "2026-10-04T00:00:00Z".into(),
        ..Task::default()
    });
    t.write(&format!("{DIR}/plan.json"), &saved.to_json());
    t.write(&format!("{DIR}/review/round.md"), "sealed review\n");
    t.write(&format!("{DIR}/artifacts/proof.txt"), "evidence\n");
    refused(&t, &["plan", "start", "P2", "--worktree", root], "overlap");
    refused(
        &t,
        &["plan", "edit", "P1", "--suspend", "--worker-stopped"],
        "reason",
    );
    refused(
        &t,
        &["plan", "edit", "P1", "--suspend", "--reason", "  ", "--worker-stopped"],
        "blank",
    );
    refused(
        &t,
        &["plan", "edit", "P1", "--suspend", "--reason", "review cap"],
        "worker-stopped",
    );
    t.ok(&[
        "plan",
        "edit",
        "P1",
        "--suspend",
        "--reason",
        "review cap",
        "--worker-stopped",
    ]);
    assert_eq!(doc(&t).field("P1", "status"), "suspended");
    assert_eq!(doc(&t).field("P1", "done_at"), "");
    assert!(t.ok(&["next"]).contains("schedulable: P2"));
    assert!(t.ok(&["status"]).contains("suspended [P1]"));
    assert!(t.read(&format!("{DIR}/STATE.md")).contains("suspended: P1"));
    assert!(t.ok(&["plan", "render"]).contains("suspended P1: review cap"));
    t.ok(&["plan", "start", "P2", "--worktree", root]);
    refused(
        &t,
        &["plan", "start", "P1", "--resume", "--confirm"],
        "overlap",
    );
    t.ok(&[
        "plan",
        "edit",
        "P2",
        "--suspend",
        "--reason",
        "shift",
        "--worker-stopped",
    ]);
    refused(&t, &["plan", "start", "P1", "--resume"], "confirm");
    t.ok(&["plan", "start", "P1", "--resume", "--confirm"]);
    assert_eq!(doc(&t).field("P1", "status"), "in-progress");
    assert_eq!(doc(&t).field("P1", "worktree"), root);
    assert_eq!(doc(&t).field("P2", "status"), "suspended");
    assert_eq!(doc(&t).plan("P1").unwrap().tasks[0].commit, sha);
    assert_eq!(t.read(&format!("{DIR}/review/round.md")), "sealed review\n");
    assert_eq!(t.read(&format!("{DIR}/artifacts/proof.txt")), "evidence\n");
    assert!(t
        .read(&format!("{DIR}/suspensions/P1.json"))
        .contains("review cap"));
}

#[test]
fn R24_invalid_transitions_and_identity_refuse_without_mutation() {
    let t = fixture();
    let root = t.0.to_str().unwrap();
    refused(
        &t,
        &[
            "plan",
            "edit",
            "P1",
            "--suspend",
            "--reason",
            "x",
            "--worker-stopped",
        ],
        "ready",
    );
    refused(
        &t,
        &["plan", "start", "P1", "--resume", "--confirm"],
        "ready",
    );
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    t.ok(&[
        "plan",
        "edit",
        "P1",
        "--suspend",
        "--reason",
        "review cap",
        "--worker-stopped",
    ]);
    refused(&t, &["plan", "done", "P1"], "suspended");
    refused(
        &t,
        &[
            "plan",
            "edit",
            "P1",
            "--suspend",
            "--reason",
            "again",
            "--worker-stopped",
        ],
        "suspended",
    );
    let mut saved = doc(&t);
    saved.plan_mut("P1").unwrap().worktree = "/does/not/exist".into();
    t.write(&format!("{DIR}/plan.json"), &saved.to_json());
    undecidable(
        &t,
        &["plan", "start", "P1", "--resume", "--confirm"],
        "worktree",
    );
}

#[test]
fn R24_resume_rechecks_dependency_and_capacity() {
    let t = fixture();
    let root = t.0.to_str().unwrap();
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    t.ok(&[
        "plan",
        "edit",
        "P1",
        "--suspend",
        "--reason",
        "review cap",
        "--worker-stopped",
    ]);
    let mut saved = doc(&t);
    saved.plan_mut("P1").unwrap().deps.push("P2".into());
    t.write(&format!("{DIR}/plan.json"), &saved.to_json());
    refused(
        &t,
        &["plan", "start", "P1", "--resume", "--confirm"],
        "unfinished dependencies",
    );
    saved.plan_mut("P1").unwrap().deps.clear();
    t.write(&format!("{DIR}/plan.json"), &saved.to_json());
    let project = t.read(".dstack/project/PROJECT.md");
    t.write(
        ".dstack/project/PROJECT.md",
        &project.replace("max_concurrent: 5", "max_concurrent: 1"),
    );
    t.ok(&["plan", "start", "P2", "--worktree", root]);
    refused(
        &t,
        &["plan", "start", "P1", "--resume", "--confirm"],
        "worker slot",
    );
}

#[test]
fn R24_plan_mutations_do_not_overwrite_a_concurrent_resume() {
    let t = fixture();
    let root = t.0.to_str().unwrap();
    let mut saved = doc(&t);
    saved.plan_mut("P2").unwrap().files = vec!["independent".into()];
    saved.plan_mut("P1").unwrap().tasks.push(Task { id: "T1".into(), slug: "race".into(), ..Task::default() });
    t.write(&format!("{DIR}/plan.json"), &saved.to_json());
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    t.ok(&["plan", "start", "P2", "--worktree", root]);
    t.ok(&["plan", "edit", "P2", "--suspend", "--reason", "wait", "--worker-stopped"]);

    let bin = t.0.join("fake-bin");
    let git = t.write("fake-bin/git", "#!/bin/sh\nif [ \"$1\" = cat-file ]; then\n  : > \"$R24_LOADED\"\n  while [ ! -f \"$R24_RELEASE\" ]; do sleep 0.01; done\nfi\nPATH=\"${PATH#*:}\" exec git \"$@\"\n");
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).unwrap();
    let loaded = t.0.join("loaded");
    let release = t.0.join("release");
    let spawn = |args: &[&str]| Command::new(env!("CARGO_BIN_EXE_dstack"))
        .current_dir(&t.0).env("DSTACK_ROOT", &t.0)
        .env("DSTACK_HOME", std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"))
        .env("DSTACK_DEPS", t.0.join("deps.tsv")).env("CLAUDE_CODE_SESSION_ID", "mode-settings-test")
        .env("PATH", std::env::join_paths(std::iter::once(bin.clone()).chain(std::env::split_paths(&std::env::var_os("PATH").unwrap()))).unwrap())
        .env("R24_LOADED", &loaded).env("R24_RELEASE", &release).args(args)
        .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    let task = spawn(&["task", "done", "T1", "--commit", "HEAD"]);
    for _ in 0..500 { if loaded.is_file() { break; } std::thread::sleep(std::time::Duration::from_millis(10)); }
    assert!(loaded.is_file(), "task done did not reach its post-load Git check");
    let lock_held = t.0.join(".dstack/local/lock").is_dir();
    let mut resume = Some(spawn(&["plan", "start", "P2", "--resume", "--confirm"]));
    let early = if lock_held { None } else { Some(resume.take().unwrap().wait_with_output().unwrap()) };
    t.write("release", "");
    assert!(task.wait_with_output().unwrap().status.success());
    let resumed = early.unwrap_or_else(|| resume.take().unwrap().wait_with_output().unwrap());
    assert!(resumed.status.success(), "{}", String::from_utf8_lossy(&resumed.stderr));
    assert_eq!(doc(&t).field("P2", "status"), "in-progress", "a stale task mutation overwrote the resumed Plan");
}

#[test]
fn R24_next_max_does_not_advertise_beyond_the_admission_cap() {
    let t = fixture();
    let mut saved = doc(&t);
    saved.plan_mut("P2").unwrap().files = vec!["independent".into()];
    t.write(&format!("{DIR}/plan.json"), &saved.to_json());
    let project = t.read(".dstack/project/PROJECT.md");
    t.write(".dstack/project/PROJECT.md", &project.replace("max_concurrent: 5", "max_concurrent: 1"));
    let root = t.0.to_str().unwrap();
    t.ok(&["plan", "start", "P1", "--worktree", root]);
    let next = t.ok(&["next", "--max", "2"]);
    assert!(next.contains("admission cap: 1 (PROJECT.md max_concurrent); schedulable free slots 0"), "{next}");
    assert!(next.contains("schedulable: (none) — 0 of 0 free slot(s)"), "{next}");
    refused(&t, &["plan", "start", "P2", "--worktree", root], "worker slot");
}
