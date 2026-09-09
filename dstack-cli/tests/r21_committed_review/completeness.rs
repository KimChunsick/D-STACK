use crate::support::{Repo, RUN};
use serde_json::json;
use std::fs;

#[test]
fn R21__large_text_tail_and_binary_diff_are_complete_despite_git_driver_settings() {
    let r = Repo::new();
    r.s.write(
        "allowed/large.txt",
        &format!("FIRST\n{}\nLAST\n", "x".repeat(70000)),
    );
    fs::write(r.s.0.join("allowed/binary.dat"), [0, 1, 255, 0, 14]).unwrap();
    let head = r.commit();
    r.plan(&[&head[..12]]);
    r.git(&["config", "diff.external", "false"]);
    r.git(&["config", "diff.relative", "true"]);
    r.git(&["config", "color.ui", "always"]);
    let bundle = r.checked();
    let diff = r.git(&[
        "diff",
        "--binary",
        "--full-index",
        "--no-ext-diff",
        "--no-textconv",
        "--no-renames",
        "--no-color",
        "--no-relative",
        "--unified=3",
        "--ignore-submodules=none",
        "--submodule=short",
        "--src-prefix=a/",
        "--dst-prefix=b/",
        &r.base,
        &head,
        "--",
    ]);
    assert!(diff.len() > 65536);
    assert!(bundle.contains(&diff));
    assert!(bundle.contains("+FIRST\n"));
    assert!(bundle.contains("+LAST\n"));
    assert!(bundle.contains("GIT binary patch"));
    assert!(bundle.len() <= 1024000);
}

#[test]
fn R21__oversize_refuses_both_default_and_explicit_publication_without_truncating() {
    let r = Repo::new();
    let head = r.task(&format!("FIRST\n{}\nLAST\n", "x".repeat(1024000)));
    r.plan(&[&head]);
    r.refused("bundle exceeds 1024KB");
    let out = r.run(&[
        "review",
        "--run",
        "sample",
        "--scope",
        "plan",
        "--plan",
        "P1",
        "--committed",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("(ceiling 1024000)"));
    assert!(String::from_utf8_lossy(&out.stderr).contains("bundle exceeds 1024KB"));
    assert!(!r.s.0.join(format!("{RUN}/review")).exists());
    r.s.write("bundle.txt", "existing output\n");
    assert_eq!(r.review(true).status.code(), Some(1));
    assert_eq!(r.s.read("bundle.txt"), "existing output\n");
}

#[test]
fn R21__literal_paths_with_spaces_and_rename_deletions_are_complete() {
    let r = Repo::new();
    r.git(&["mv", "allowed/a.txt", "allowed/space name.txt"]);
    r.s.write("allowed/space name.txt", "renamed\n");
    let head = r.commit();
    r.plan(&[&head]);
    r.edit_plan(|p| p["files"] = json!(["./allowed/a.txt", "allowed/space name.txt"]));
    let bundle = r.checked();
    assert!(bundle.contains("deleted file mode"));
    assert!(bundle.contains("+renamed"));
    assert!(bundle.contains("--- file: allowed/space name.txt"));
}

#[test]
fn R21__reverted_declared_changes_keep_both_task_provenance_records() {
    let r = Repo::new();
    let first = r.task("temporary\n");
    let head = r.task("initial\n");
    r.plan(&[&first, &head]);
    let bundle = r.checked();
    assert!(bundle.contains(&format!("task: T1 commit: {first}")));
    assert!(bundle.contains(&format!("task: T2 commit: {head}")));
    assert!(bundle.contains("(no changes against the base)"));
}

#[test]
fn R21__hex_named_refs_cannot_replace_recorded_commit_identity() {
    let r = Repo::new();
    let first = r.task("first\n");
    let head = r.task("last\n");
    r.git(&["branch", &first[..12], &head]);
    r.plan(&[&first[..12], &head]);
    let bundle = r.checked();
    assert!(bundle.contains(&format!("task: T1 commit: {first}")));
    assert!(bundle.contains(&format!("base: {}", r.base)));
}
