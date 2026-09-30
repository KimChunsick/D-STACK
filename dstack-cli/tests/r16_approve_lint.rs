// tests/r16_approve_lint.rs
// R16: `request approve` runs `dstack lint-ko` on the whole request.md before it stamps; an S1 hit
// refuses the approval with the hits printed and nothing stamped, a clean request approves; the
// scope table shipped in claude/lint/ko-scope.tsv classifies a run's and a quick task's request.md
// wherever the store sits; and the request-approve fixtures prove the refusal.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::{Command, Output};
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::verbs;
use dstack_cli::verbs::doctor::selfrun;
use support::Scratch;

const RUN: &str = "20261001T000000Z_lint";
const REQUEST: &str = ".dstack/runs/20261001T000000Z_lint/request.md";
const STAMP: &str = ".dstack/runs/20261001T000000Z_lint/request.approved";

/// A filled Goal request whose background paragraph is `background`.
fn request(background: &str) -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: skip\nreview: on\ncodex_effort: high\ne2e: cli\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\
         # 승인 때 한국어 검사\n\n승인 전에 요청서 전체를 검사해요.\n\n# 1부 요청\n\n\
         ## 배경과 문제\n\n{background}\n\n\
         ## 목표\n\n1. 승인 전에 요청서를 검사해요.\n\n\
         ## 비목표\n\n1. 다른 파일은 검사하지 않아요.\n\n\
         ## 사용 시나리오\n\n### S1 승인해요\n\n메인이 요청서를 승인해요.\n\n\
         ## 요구사항\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n\
         ## 열린 가정\n\n없음.\n"
    )
}

/// The table this checkout ships, as the store's own: the machine table under $HOME may be an
/// older install, and the repository table is the first place lint-ko looks.
fn shipped_scope(t: &Scratch) {
    let table = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude/lint/ko-scope.tsv");
    let text = std::fs::read_to_string(table).expect("the shipped scope table");
    t.write(".dstack/project/ko-scope.tsv", &text);
}

fn scratch(text: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    shipped_scope(&t);
    t.run_fixture(RUN, None, true);
    t.write(REQUEST, text);
    t
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn R16_approve_lint_refuses_a_real_violation() {
    let t = scratch(&request("이 파일이 정본이라서 승인한 내용이 바뀌면 안 돼요."));
    let out = t.run(&["request", "approve", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains("K01 (S1) matched '정본'"), "{}", stdout(&out));
    assert!(!t.0.join(STAMP).exists(), "a refused approval stamps nothing");
}

#[test]
fn R16_approve_lint_passes_a_clean_request() {
    let t = scratch(&request("승인한 내용이 바뀌면 안 돼요."));
    let out = t.run(&["request", "approve", "--run", RUN]);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    // Classified and scanned, not passed as unclassified.
    assert!(
        stdout(&out).contains("files 1, hits 0 (S1 0), unclassified 0"),
        "{}",
        stdout(&out)
    );
    assert!(t.0.join(STAMP).exists());
}

#[test]
fn R16_scope_table_classifies_request_documents() {
    let t = Scratch::new();
    shipped_scope(&t);
    t.write(".dstack/runs/r/request.md", "요청서예요.\n");
    t.write(".dstack/quick/q/request.md", "요청서예요.\n");
    // A Plan worktree sees the store of the main checkout by its absolute path.
    let other = Scratch::new();
    let absolute = other.write(".dstack/runs/r/request.md", "요청서예요.\n");
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .current_dir(&t.0)
        .env("DSTACK_ROOT", &t.0)
        .env(
            "DSTACK_HOME",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"),
        )
        .args(["lint-ko", "--report", ".dstack/runs/r/request.md", ".dstack/quick/q/request.md"])
        .arg(&absolute)
        .output()
        .expect("lint-ko");
    let out = stdout(&out);
    assert!(out.contains(".dstack/runs/r/request.md: scope ko-haeyo"), "{out}");
    assert!(out.contains(".dstack/quick/q/request.md: scope ko-haeyo"), "{out}");
    assert!(out.contains(&format!("{}: scope ko-haeyo", absolute.display())), "{out}");
    assert!(out.contains("unclassified 0"), "{out}");
}

#[test]
fn R16_request_approve_fixtures_reject_a_lint_violation() {
    let home = Home::resolve().expect("the repository of this test binary");
    let dir: PathBuf = home.home.join("lint/fixtures/request-approve");
    let fixtures = selfrun::fixtures(&dir);
    assert!(
        fixtures
            .iter()
            .any(|(path, _)| path.ends_with("bad-lint-violation.md")),
        "{fixtures:?}"
    );
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let checkers = verbs::request::selftests();
    let checker = checkers
        .iter()
        .find(|checker| checker.checker() == "request-approve")
        .expect("the request-approve checker");
    for (fixture, expected) in fixtures {
        let actual = checker.run(&mut ctx, &fixture).expect("a verdict");
        assert_eq!(actual, expected, "{}", fixture.display());
    }
}
