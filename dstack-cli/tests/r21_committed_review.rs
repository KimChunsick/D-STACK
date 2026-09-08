#![allow(non_snake_case)]
#[path = "r21_committed_review/completeness.rs"]
mod completeness;
#[path = "r21_committed_review/refusals.rs"]
mod refusals;
#[path = "support/committed_review.rs"]
mod support;
use support::{Repo, ROW};

#[test]
fn R21__multiple_tasks_use_their_complete_range_without_earlier_same_file_changes() {
    let r = Repo::new();
    let base = r.task("EARLIER_PLAN\n");
    let first = r.task("FIRST_TASK\n");
    r.s.write("allowed/b.txt", "SECOND_TASK\n");
    let head = r.commit();
    // Task declaration order need not be commit order; provenance follows immutable ancestry.
    r.plan(&[&head, &first]);
    let bundle = r.checked();
    assert!(bundle.starts_with(&format!("=== REQUEST (frozen) ===\n{ROW}\n")));
    for line in [
        "mode: committed-plan".to_owned(),
        format!("base: {base}"),
        format!("head: {head}"),
        format!("task: T2 commit: {first}"),
        format!("task: T1 commit: {head}"),
    ] {
        assert!(bundle.contains(&line), "missing {line}");
    }
    assert!(bundle.contains("-EARLIER_PLAN\n+FIRST_TASK"));
    assert!(bundle.contains("+SECOND_TASK"));
    assert!(!bundle.contains("-initial"));
    assert!(r.review(false).status.success());
    let default = r.s.read("bundle.txt");
    assert!(default.contains(&format!("base: {}", r.base)));
    assert!(default.contains("-initial"));
    assert!(!default.contains("mode: committed-plan"));
}

#[test]
fn R21__milestone_and_seal_contracts_remain_in_force() {
    let r = Repo::new();
    let head = r.task("task\n");
    r.plan(&[&head]);
    let args = [
        "review",
        "--run",
        "sample",
        "--scope",
        "milestone",
        "--milestone",
        "M1",
        "--out",
        "bundle.txt",
    ];
    assert!(r.run(&args).status.success());
    let ordinary = r.s.read("bundle.txt");
    let mut flagged = args.to_vec();
    flagged.push("--committed");
    let out = r.run(&flagged);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("only with --scope plan"));
    assert_eq!(r.s.read("bundle.txt"), ordinary);
    r.s.write(
        ".dstack/raw.md",
        "| R | verdict | evidence |\n|---|---|---|\n| R21 | absent | none |\nVERDICT: approve\n",
    );
    let out = r.run(&[
        "review",
        "seal",
        "--run",
        "sample",
        "--scope",
        "plan",
        "--id",
        "P1",
        "--from",
        ".dstack/raw.md",
    ]);
    assert_eq!(out.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&out.stdout).contains("this round cannot seal positively"));
    assert!(r
        .s
        .read(".dstack/runs/sample/review/codex-review-001.md")
        .contains("absent"));
}

#[test]
fn R21__fixture_checker_catches_good_and_bad_committed_ranges() {
    use dstack_cli::core::{context::Context, registry::Registry, roots::Home};
    use dstack_cli::selftest::Verdict;
    use std::{path::PathBuf, rc::Rc};
    let home = Home::resolve().unwrap();
    let fixtures = home.repo.join("claude/lint/fixtures/review-bundle");
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(dstack_cli::verbs::all_verbs())),
    );
    let checker = dstack_cli::verbs::all_selftests()
        .into_iter()
        .find(|checker| checker.checker() == "review-bundle")
        .expect("R21 registered fixture checker");
    let mut counted = [0, 0];
    for entry in std::fs::read_dir(fixtures).unwrap() {
        let path = entry.unwrap().path();
        let good = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("good-");
        let wanted = if good { Verdict::Pass } else { Verdict::Reject };
        assert_eq!(
            checker.run(&mut ctx, &path).unwrap(),
            wanted,
            "{}",
            path.display()
        );
        counted[usize::from(good)] += 1;
        eprintln!("R21 fixture {}: {wanted:?}", path.display());
    }
    assert!(counted.iter().all(|n| *n > 0));
}
