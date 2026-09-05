// tests/r03_store_plan.rs
// R03: the plan.json layer reads, writes and renders exactly what the shell's jq programs do.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/shell_ref.rs"]
mod shell_ref;

use std::path::{Path, PathBuf};
use std::process::Command;

use dstack_cli::core::roots::Roots;
use dstack_cli::store::plan::{self, PlanDoc};
use dstack_cli::store::plan_graph as graph;
use dstack_cli::store::plan_ids as ids;

/// The fixture is compact on purpose: every pretty form in this file is produced, never typed.
const FIXTURE: &str = r#"{"v":2,"milestones":[{"id":"M1","slug":"core","order":1},{"id":"M2","slug":"wrap","order":2}],"plans":[{"id":"P1","milestone":"M1","slug":"alpha","files":["src/a.rs","docs"],"deps":[],"status":"done","worktree":"","started_at":"2026-01-01T00:00:00Z","done_at":"2026-01-02T00:00:00Z","tasks":[{"id":"T1","slug":"first","covers":["R02","R01"],"files":["src/a.rs"],"deps":[],"commit":"abc1234","done_at":"2026-01-02T00:00:00Z"}]},{"id":"P2","milestone":"M1","slug":"beta","files":["src/b.rs"],"deps":["P1"],"status":"pending","worktree":"","started_at":"","done_at":"","tasks":[]},{"id":"P1.1","milestone":"M1","slug":"gamma","files":["src/c.rs"],"deps":["P2"],"status":"in-progress","worktree":"/tmp/wt","started_at":"2026-01-03T00:00:00Z","done_at":"","tasks":[{"id":"T2","slug":"second","covers":["R03"],"files":["src/c.rs"],"deps":["T1"],"commit":"","done_at":""}]}]}"#;

fn fixture() -> PlanDoc {
    serde_json::from_str(FIXTURE).expect("the fixture parses")
}

fn empty() -> PlanDoc {
    serde_json::from_str(plan::SEED).expect("the seed parses")
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("dstack-r03-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// The closed v2 run of this repository, when the machine-local store is there.
fn live_run() -> Option<PathBuf> {
    let dir = Roots::resolve()
        .ok()?
        .runs
        .join("20260902T052531Z_dstack-v2");
    if dir.join("plan.json").is_file() {
        Some(dir)
    } else {
        None
    }
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).expect("read")
}

#[test]
fn r03__pretty_json_matches_jq() {
    let doc = fixture();
    assert_eq!(
        PlanDoc {
            v: 2,
            milestones: doc.milestones[..1].to_vec(),
            plans: vec![],
        }
        .to_json(),
        "{\n  \"v\": 2,\n  \"milestones\": [\n    {\n      \"id\": \"M1\",\n      \"slug\": \"core\",\n      \"order\": 1\n    }\n  ],\n  \"plans\": []\n}\n"
    );
    if !have("jq") {
        eprintln!("skipped the jq comparison: jq is not on PATH");
        return;
    }
    let dir = scratch("jq");
    let compact = dir.join("compact.json");
    std::fs::write(&compact, FIXTURE).expect("write");
    let out = Command::new("jq")
        .arg(".")
        .arg(&compact)
        .output()
        .expect("run jq");
    let expected = String::from_utf8(out.stdout).expect("utf-8");
    assert_eq!(doc.to_json(), expected, "to_json is not jq's pretty print");
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r03__the_live_store_round_trips_byte_for_byte() {
    let runs = match Roots::resolve() {
        Ok(roots) if roots.runs.is_dir() => roots.runs,
        _ => {
            eprintln!("skipped: no .dstack store on this machine");
            return;
        }
    };
    let mut seen = 0;
    for entry in std::fs::read_dir(&runs).expect("list runs") {
        let dir = entry.expect("entry").path();
        if !plan::exists(&dir) {
            continue;
        }
        seen += 1;
        let raw = read(&plan::path(&dir));
        let doc = plan::load(&dir).expect("load plan.json");
        assert_eq!(doc.to_json(), raw, "{} does not round-trip", dir.display());
    }
    assert!(seen > 0, "no run in {} holds a plan.json", runs.display());
}

#[test]
fn r03__ensure_seeds_and_commit_regenerates_both_documents() {
    let dir = scratch("commit");
    assert!(!plan::exists(&dir));
    plan::ensure(&dir).expect("seed");
    assert_eq!(read(&plan::path(&dir)), "{\"v\":2,\"milestones\":[],\"plans\":[]}\n");
    assert!(plan::exists(&dir));
    let seeded = plan::load(&dir).expect("load the seed");
    assert_eq!(seeded.v, 2);
    assert!(seeded.milestones.is_empty() && seeded.plans.is_empty());

    std::fs::write(plan::path(&dir), "{\"v\":2,\"milestones\":[],\"plans\":[]}\n").expect("write");
    plan::ensure(&dir).expect("second ensure");
    assert_eq!(read(&plan::path(&dir)), "{\"v\":2,\"milestones\":[],\"plans\":[]}\n");

    let doc = fixture();
    let repo = repo();
    doc.commit(&dir, "20260101T000000Z_run", &repo).expect("commit");
    let mut refreshed = doc.clone();
    graph::refresh(&mut refreshed);
    assert_eq!(read(&plan::path(&dir)), refreshed.to_json());
    assert_eq!(
        read(&dir.join("ROADMAP.md")),
        graph::render_roadmap(&refreshed, "20260101T000000Z_run")
    );
    let head = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(&repo)
        .output()
        .expect("git");
    let head = String::from_utf8(head.stdout).expect("utf-8").trim().to_string();
    let state = read(&dir.join("STATE.md"));
    assert!(state.starts_with("# State — 20260101T000000Z_run\n\nGenerated by dstack from plan.json. Never hand-edit.\n\n"));
    assert!(
        state.contains(&format!("\nlast_commit: {head}\n")),
        "STATE.md does not carry the worktree HEAD:\n{state}"
    );
    // A worktree git cannot read is the shell's `|| echo none`.
    doc.commit(&dir, "20260101T000000Z_run", Path::new("/nonexistent"))
        .expect("commit");
    assert!(read(&dir.join("STATE.md")).contains("\nlast_commit: none\n"));
    std::fs::remove_dir_all(&dir).expect("clean up");
}

#[test]
fn r03__refresh_only_recomputes_pending_and_ready() {
    let mut doc = fixture();
    graph::refresh(&mut doc);
    assert_eq!(doc.field("P1", "status"), "done");
    assert_eq!(doc.field("P2", "status"), "ready");
    assert_eq!(doc.field("P1.1", "status"), "in-progress");

    // A dependency that is not done — and one that does not exist at all — keeps a plan pending.
    let mut doc = fixture();
    doc.plan_mut("P2").expect("P2").deps = vec!["P1".into(), "P9".into()];
    doc.plan_mut("P1.1").expect("P1.1").status = "ready".into();
    graph::refresh(&mut doc);
    assert_eq!(doc.field("P2", "status"), "pending");
    assert_eq!(doc.field("P1.1", "status"), "pending");
}

#[test]
fn r03__acyclic_follows_the_peel_rule() {
    let doc = fixture();
    assert!(graph::acyclic_plans(&doc));
    assert!(graph::acyclic_tasks(&doc));
    assert!(ids::assert_acyclic_plans(&doc).is_ok());

    let mut doc = fixture();
    doc.plan_mut("P1").expect("P1").deps = vec!["P1.1".into()];
    assert!(!graph::acyclic_plans(&doc));
    let err = ids::assert_acyclic_plans(&doc).expect_err("cycle");
    assert_eq!(err.code(), 1);
    assert_eq!(
        err.message(),
        "the resulting plans dependency graph has a cycle (a plan cannot depend on itself or on anything that waits for it)"
    );

    let mut doc = fixture();
    doc.plan_mut("P1").expect("P1").tasks[0].deps = vec!["T2".into()];
    assert!(!graph::acyclic_tasks(&doc));
    assert_eq!(
        ids::assert_acyclic_tasks(&doc).expect_err("cycle").message(),
        "the resulting tasks dependency graph has a cycle (a plan cannot depend on itself or on anything that waits for it)"
    );

    // Deps that name unknown ids are ignored, not treated as a cycle.
    let mut doc = fixture();
    doc.plan_mut("P1").expect("P1").deps = vec!["nope".into()];
    assert!(graph::acyclic_plans(&doc));
}

#[test]
fn r03__dependents_and_subtree_busy_match_jq() {
    let doc = fixture();
    assert_eq!(graph::dependents(&doc, "P1"), vec!["P1", "P1.1", "P2"]);
    assert_eq!(graph::dependents(&doc, "P1.1"), vec!["P1.1"]);
    assert_eq!(graph::dependents(&doc, "nope"), vec!["nope"]);
    assert_eq!(graph::subtree_busy(&doc, "P1"), "P1.1");
    assert_eq!(graph::subtree_busy(&doc, "P2"), "P1.1");
    assert_eq!(graph::subtree_busy(&doc, "P1.1"), "P1.1");
    let mut doc = fixture();
    doc.plan_mut("P1.1").expect("P1.1").status = "done".into();
    assert_eq!(graph::subtree_busy(&doc, "P1"), "");
}

#[test]
fn r03__ids_are_minted_like_the_shell() {
    let doc = fixture();
    assert_eq!(ids::next_int_id(&doc, "M"), "M3");
    assert_eq!(ids::next_int_id(&doc, "P"), "P3");
    assert_eq!(ids::next_int_id(&empty(), "M"), "M1");
    assert_eq!(ids::next_int_id(&empty(), "P"), "P1");

    let taken: Vec<String> = vec!["P1".into(), "P1.1".into(), "P1.3".into()];
    assert_eq!(ids::next_decimal_id("P1", &taken).expect("free"), "P1.2");
    let full: Vec<String> = (1..=99).map(|k| format!("P1.{k}")).collect();
    let err = ids::next_decimal_id("P1", &full).expect_err("full");
    assert_eq!(err.code(), 2);
    assert_eq!(err.message(), "no free decimal id under P1 (99 taken)");

    assert_eq!(doc.plan_ids(), vec!["P1", "P2", "P1.1"]);
    assert_eq!(doc.task_ids(), vec!["T1", "T2"]);
    assert_eq!(doc.field("P1.1", "milestone"), "M1");
    assert_eq!(doc.field("P1.1", "worktree"), "/tmp/wt");
    assert_eq!(doc.field("P2", "worktree"), "");
    assert_eq!(doc.field("nope", "status"), "");
    assert_eq!(doc.field("P2", "nosuchfield"), "");
}

#[test]
fn r03__validation_messages_are_character_for_character() {
    assert_eq!(ids::csv_list("a, b ,,c\n"), vec!["a", "b", "c"]);
    assert!(ids::csv_list("  ").is_empty());

    assert!(ids::path_within("a/b", "a"));
    assert!(ids::path_within("a", "a"));
    assert!(!ids::path_within("a", "a/b"));
    assert!(!ids::path_within("ab/c", "a"));

    assert_eq!(
        ids::validate_files("src/a.rs, docs").expect("valid"),
        vec!["src/a.rs", "docs"]
    );
    let err = ids::validate_files("src/a.rs, ../etc").expect_err("dotdot");
    assert_eq!(err.code(), 1);
    assert_eq!(
        err.message(),
        "invalid file path: '../etc' (must be repo-relative: no leading /, no .. segment, no * ? [ , not empty)"
    );
    for bad in ["/abs", "a/*", "a?b", "a[0]", "..", "a/..", "a/../b"] {
        assert!(
            ids::validate_files(bad).is_err(),
            "{bad} should not validate"
        );
    }
    assert_eq!(
        ids::validate_files(" , ").expect_err("empty").message(),
        "--files must list at least one repo-relative path (comma separated)"
    );

    let known: Vec<String> = vec!["P1".into(), "P2".into()];
    let shown: Vec<String> = vec!["P1".into()];
    assert_eq!(
        ids::validate_deps("P1,P2", &known, "plan", &known).expect("known"),
        vec!["P1", "P2"]
    );
    assert!(ids::validate_deps("", &known, "plan", &known).expect("empty").is_empty());
    assert_eq!(
        ids::validate_deps("P3", &known, "plan", &shown)
            .expect_err("unknown")
            .message(),
        "plan dependency does not exist: P3 (known: P1)"
    );
    assert_eq!(
        ids::validate_deps("T9", &[], "task", &[])
            .expect_err("unknown")
            .message(),
        "task dependency does not exist: T9 (known: none)"
    );
}

#[test]
fn r03__covers_helpers_are_sorted_and_unique() {
    let doc = fixture();
    assert_eq!(graph::plan_covers(&doc, "P1"), vec!["R01", "R02"]);
    assert_eq!(graph::plan_covers(&doc, "P2"), Vec::<String>::new());
    assert_eq!(graph::milestone_covers(&doc, "M1"), vec!["R01", "R02", "R03"]);
    assert_eq!(graph::milestone_covers(&doc, "M2"), Vec::<String>::new());
    assert_eq!(graph::tasks_covering(&doc, "R01"), vec!["P1/T1"]);
    assert_eq!(graph::tasks_covering(&doc, "R03"), vec!["P1.1/T2"]);
    assert!(graph::tasks_covering(&doc, "R99").is_empty());
}

#[test]
#[cfg_attr(
    not(feature = "shell-parity"),
    ignore = "skipped: historical shell comparison is opt-in (--features shell-parity)"
)]
fn r03__renders_match_the_shell_renderers() {
    let doc = fixture();
    assert_eq!(
        graph::counts_line(&doc),
        "milestones 2, plans 3 (pending 1, ready 0, in-progress 1, done 1), tasks 2 (committed 1)"
    );
    assert!(graph::render_table(&doc).contains("| — | (no plans yet) | — | — | — | — |"));
    assert_eq!(
        graph::render_state(&doc, "run-1", "abc1234", "2026-01-04T00:00:00Z"),
        "# State — run-1\n\nGenerated by dstack from plan.json. Never hand-edit.\n\n\
         current_plan: P1.1\nready: \nin_progress: P1.1\nblocked: P2\ndone: P1\n\
         last_commit: abc1234\nupdated_at: 2026-01-04T00:00:00Z\n"
    );

    if have("jq") && have("bash") {
        let dir = scratch("render");
        std::fs::write(dir.join("plan.json"), doc.to_json()).expect("write");
        let out = Command::new("bash")
            .arg("-c")
            .arg(". \"$1/roadmap.sh\"; _plan_table \"$2\"; _plan_counts \"$2\"")
            .arg("bash")
            .arg(shell_ref::lib())
            .arg(&dir)
            .output()
            .expect("run the shell renderers");
        assert_eq!(
            String::from_utf8(out.stdout).expect("utf-8"),
            format!("{}{}\n", graph::render_table(&doc), graph::counts_line(&doc))
        );
        std::fs::remove_dir_all(&dir).expect("clean up");
    } else {
        eprintln!("skipped the shell comparison: jq or bash is not on PATH");
    }

    let run = match live_run() {
        Some(dir) => dir,
        None => {
            eprintln!("skipped the live comparison: no closed v2 run on this machine");
            return;
        }
    };
    let run_id = run
        .file_name()
        .expect("run id")
        .to_string_lossy()
        .into_owned();
    let live = plan::load(&run).expect("load");
    assert_eq!(read(&run.join("ROADMAP.md")), graph::render_roadmap(&live, &run_id));
    let state = read(&run.join("STATE.md"));
    let value = |key: &str| -> String {
        state
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{key}: ")))
            .unwrap_or("")
            .to_string()
    };
    assert_eq!(
        state,
        graph::render_state(&live, &run_id, &value("last_commit"), &value("updated_at"))
    );
}
