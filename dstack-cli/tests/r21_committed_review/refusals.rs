use crate::support::{Repo, RUN};
use serde_json::json;
use std::fs;

#[test]
fn R21__absent_duplicate_mutable_and_missing_commit_records_are_refused() {
    for case in [
        "empty",
        "no-commit",
        "unfinished",
        "duplicate",
        "duplicate-id",
        "missing",
        "mutable",
    ] {
        let r = Repo::new();
        let head = r.task("task\n");
        r.plan(&[&head]);
        let reason = match case {
            "empty" => {
                r.plan(&[]);
                "at least one completed Task"
            }
            "no-commit" => {
                r.plan(&[""]);
                "completion record and commit"
            }
            "unfinished" => {
                r.edit_plan(|p| p["tasks"][0]["done_at"] = json!(""));
                "completion record and commit"
            }
            "duplicate" => {
                r.plan(&[&head, &head[..12]]);
                "duplicate recorded commit"
            }
            "duplicate-id" => {
                r.plan(&[&r.base, &head]);
                r.edit_plan(|p| p["tasks"][1]["id"] = json!("T1"));
                "duplicate Task id"
            }
            "missing" => {
                r.plan(&["ffffffffffffffffffffffffffffffffffffffff"]);
                "missing or ambiguous"
            }
            _ => {
                r.plan(&["HEAD"]);
                "immutable commit ID"
            }
        };
        r.refused(reason);
    }
}

#[test]
fn R21__gaps_omitted_tasks_unrelated_branches_and_unrelated_head_are_refused() {
    for case in ["gap", "omitted", "branch", "head"] {
        let r = Repo::new();
        let first = r.task("first\n");
        if case == "branch" {
            r.git(&["checkout", "--detach", &r.base]);
        }
        let middle = r.task("middle\n");
        let head = r.task("last\n");
        match case {
            "gap" | "branch" => r.plan(&[&first, &head]),
            "omitted" => {
                r.plan(&[&first, &middle, &head]);
                r.edit_plan(|p| p["tasks"][1]["commit"] = json!(""));
            }
            _ => r.plan(&[&first, &middle]),
        }
        r.refused(match case {
            "omitted" => "completion record and commit",
            "head" => "HEAD is not a recorded Task end",
            _ => "unrecorded or omitted commit",
        });
    }
}

#[test]
fn R21__dirty_index_worktree_untracked_and_outside_source_are_refused() {
    for case in ["tracked", "staged", "untracked", "outside"] {
        let r = Repo::new();
        let head = r.task("task\n");
        r.plan(&[&head]);
        match case {
            "tracked" | "staged" => {
                r.s.write("allowed/a.txt", "dirty\n");
            }
            "untracked" => {
                r.s.write("allowed/new.txt", "dirty\n");
            }
            _ => {
                r.s.write("outside-new.txt", "dirty\n");
            }
        }
        if case == "staged" {
            r.git(&["add", "allowed/a.txt"]);
        }
        r.refused("clean worktree and index");
    }
}

#[test]
fn R21__intermediate_undeclared_paths_cannot_be_hidden_by_reverts_or_renames() {
    for case in ["revert", "delete", "rename", "neighbor"] {
        let r = Repo::new();
        let path = if case == "neighbor" {
            "allowed-neighbor.txt"
        } else {
            "outside.txt"
        };
        r.s.write(path, "undeclared intermediate\n");
        let first = r.commit();
        if case == "rename" {
            r.git(&["mv", path, "allowed/moved.txt"]);
        } else if case == "delete" || case == "neighbor" {
            fs::remove_file(r.s.0.join(path)).unwrap();
        } else {
            r.s.write(path, "initial\n");
        }
        let last = r.task("final\n");
        r.plan(&[&first, &last]);
        r.refused("undeclared path");
    }
}

#[test]
fn R21__root_and_merge_task_commits_are_refused() {
    let root = Repo::new();
    root.plan(&[&root.base]);
    root.refused("root or merge Task commit");

    let r = Repo::new();
    let first = r.task("first\n");
    r.git(&["checkout", "--detach", &r.base]);
    r.s.write("allowed/side.txt", "side\n");
    let side = r.commit();
    r.git(&["checkout", "--detach", &first]);
    r.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.test",
        "-c",
        "commit.gpgsign=false",
        "merge",
        "--no-ff",
        "-m",
        "merge",
        &side,
    ]);
    let head = r.git(&["rev-parse", "HEAD"]);
    r.plan(&[&first, &side, &head]);
    r.refused("root or merge Task commit");
}

#[test]
fn R21__unapproved_request_and_broken_coverage_still_publish_nothing() {
    for case in ["unapproved", "missing-row"] {
        let r = Repo::new();
        let head = r.task("task\n");
        r.plan(&[&head]);
        if case == "unapproved" {
            fs::remove_file(r.s.0.join(format!("{RUN}/request.approved"))).unwrap();
            r.refused("request is not approved");
        } else {
            r.edit_plan(|p| p["tasks"][0]["covers"] = json!(["R21", "R22"]));
            let out = r.review(true);
            assert_eq!(out.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&out.stderr).contains("bundle deleted"));
            assert!(!r.s.0.join("bundle.txt").exists());
        }
    }
}

#[test]
fn R21__replacement_objects_do_not_hide_a_recorded_tasks_real_paths() {
    let r = Repo::new();
    r.s.write("outside.txt", "outside\n");
    let original = r.commit();
    r.git(&["checkout", "--detach", &r.base]);
    let replacement = r.task("allowed\n");
    r.git(&["replace", &original, &replacement]);
    r.git(&["checkout", "--detach", &original]);
    r.plan(&[&original]);
    r.refused("undeclared path");
}

#[test]
fn R21__hidden_index_edits_and_shallow_ancestry_are_refused() {
    for flag in ["--assume-unchanged", "--skip-worktree"] {
        let r = Repo::new();
        let head = r.task("task\n");
        r.plan(&[&head]);
        r.git(&["update-index", flag, "allowed/a.txt"]);
        r.s.write("allowed/a.txt", "hidden dirty edit\n");
        assert!(r.git(&["status", "--porcelain"]).is_empty());
        r.refused("assume-unchanged or skip-worktree");
    }
    let r = Repo::new();
    let head = r.task("task\n");
    r.plan(&[&head]);
    r.s.write(".git/shallow", &format!("{}\n", r.base));
    assert_eq!(r.git(&["rev-parse", "--is-shallow-repository"]), "true");
    r.refused("shallow history");
}
