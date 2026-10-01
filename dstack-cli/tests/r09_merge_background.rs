// tests/r09_merge_background.rs
// R09: a row merged into an approved Goal request (`req add --run`) needs a background block from
// `request background`: without one `check request` and `request approve` fail naming the verb,
// with one the approval succeeds; every byte before the block is kept, approval leaves the block
// as plain prose, a block without a pending row fails, a legacy-layout request still approves a
// merged row without one, the verb refuses what it may not write, and the fixtures prove the rule.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::Output;
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::selftest::Verdict;
use dstack_cli::store::request_sections::{locate, Place};
use dstack_cli::verbs;
use support::Scratch;

const RUN: &str = "20261001T000000Z_merge";
const REQUEST: &str = ".dstack/runs/20261001T000000Z_merge/request.md";
const STAMP: &str = ".dstack/runs/20261001T000000Z_merge/request.approved";
const BACKGROUND: Place = Place::Section("배경과 문제");
const ADDED: &str = "병합한 요구사항이 필요해진 까닭을 적었어요.\n";
const MISSING: &str = "dstack request background --run";

const FRONT: &str = "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\n\
risk_axes: none\ndesign_review: skip\nreview: on\ncodex_effort: high\ne2e: cli\n\
unit_tests: on\nvisual: none\nkorean_polish: on\n---\n";

/// A filled Goal request in the PRD layout.
fn goal() -> String {
    format!(
        "{FRONT}# 배경 추가 시험\n\n병합 경로를 시험하는 요청서예요.\n\n\
         ## 한눈에 보기\n\n<!-- 승인 전에 채워요. -->\n\n# 1부 요청\n\n\
         ## 배경과 문제\n\n처음 승인한 배경이에요.\n\n\
         ## 목표\n\n1. 병합한 행에 배경을 남겨요.\n\n\
         ## 비목표\n\n1. 처음 배경은 고치지 않아요.\n\n\
         ## 사용 시나리오\n\n### S1 행을 병합해요\n\n메인이 행을 병합해요.\n\n\
         ## 요구사항\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n\
         ## 열린 가정\n\n없음.\n\n# 2부 설계\n\n## 지금 구조\n\n<!-- 안내 -->\n"
    )
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A store whose one run holds `text` as its request.
fn scratch(text: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(REQUEST, text);
    t
}

/// A store whose filled Goal request `request approve` has stamped.
fn approved_goal() -> Scratch {
    let t = scratch(&goal());
    t.ok(&["request", "approve", "--run", RUN]);
    t
}

fn merge_row(t: &Scratch) {
    t.ok(&[
        "req",
        "add",
        "덧붙인 요구사항이에요.",
        "--accept",
        "덧붙인 확인이에요.",
        "--run",
        RUN,
    ]);
}

fn background(t: &Scratch, content: &str) -> Output {
    t.write("added.md", content);
    t.run(&["request", "background", "--run", RUN, "--from", "added.md"])
}

#[test]
fn R09_merge_background_a_merged_row_without_background_fails() {
    let t = approved_goal();
    merge_row(&t);
    let stamped = t.read(STAMP);
    let out = t.run(&["check", "request", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stdout(&out).contains(MISSING), "{}", stdout(&out));
    let out = t.run(&["request", "approve", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stdout(&out).contains(MISSING), "{}", stdout(&out));
    assert_eq!(t.read(STAMP), stamped, "a refused approval keeps the stamp");
}

#[test]
fn R09_merge_background_lets_the_merged_row_approve_and_keeps_every_earlier_byte() {
    let t = approved_goal();
    let before = t.read(REQUEST);
    let end = locate(&before, &BACKGROUND).expect("the background").end;
    merge_row(&t);
    let out = background(&t, ADDED);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let pending = t.read(REQUEST);
    assert_eq!(
        &pending[..end],
        &before[..end],
        "every earlier byte is kept"
    );
    assert!(pending[end..].contains("pending-approval"), "{pending}");
    // The rule holds now; the pending row and the stale hash are what approve clears.
    let out = t.run(&["check", "request", "--run", RUN]);
    assert!(!stdout(&out).contains(MISSING), "{}", stdout(&out));

    t.ok(&["request", "approve", "--run", RUN]);
    t.ok(&["check", "request", "--run", RUN]);
    let after = t.read(REQUEST);
    assert_eq!(
        &after[..end],
        &before[..end],
        "the old background is unchanged"
    );
    assert!(!after.contains("pending-approval"), "{after}");
    let added = &after[end..locate(&after, &BACKGROUND).expect("the background").end];
    assert!(added.contains("### 추가 배경"), "{added}");
    assert!(added.contains(ADDED.trim_end()), "{added}");
    // A second merge piles a second block after the first, which stays as approved.
    let first = after.clone();
    let end = locate(&first, &BACKGROUND).expect("the background").end;
    merge_row(&t);
    assert_eq!(
        background(&t, "두 번째 배경이에요.\n").status.code(),
        Some(0)
    );
    t.ok(&["request", "approve", "--run", RUN]);
    assert_eq!(&t.read(REQUEST)[..end], &first[..end]);
}

#[test]
fn R09_merge_background_a_block_without_a_pending_row_fails() {
    let t = approved_goal();
    let out = background(&t, ADDED);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let out = t.run(&["check", "request", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stdout(&out).contains("no pending row"), "{}", stdout(&out));
    let out = t.run(&["request", "approve", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stdout(&out).contains("no pending row"), "{}", stdout(&out));
}

/// The refusal leaves request.md as it was and says why on stderr.
fn refused(t: &Scratch, request: &str, out: Output, why: &str) {
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stderr(&out).contains(why), "{}", stderr(&out));
    assert_eq!(t.read(request), t.read(&format!("{request}.before")));
}

fn keep(t: &Scratch, request: &str) {
    t.write(&format!("{request}.before"), &t.read(request));
}

#[test]
fn R09_merge_background_refuses_what_it_may_not_write() {
    // Unapproved: request section writes the section then.
    let t = scratch(&goal());
    keep(&t, REQUEST);
    let out = background(&t, ADDED);
    refused(&t, REQUEST, out, "dstack request section background --from");

    // Empty, or comments only.
    let t = approved_goal();
    merge_row(&t);
    keep(&t, REQUEST);
    for content in ["", "\n  \n", "<!-- 아직 비었어요. -->\n"] {
        let out = background(&t, content);
        refused(&t, REQUEST, out, "is empty");
    }
    // Prose rules of request section, and a line shaped like a pending block.
    for content in [
        "- [ ] **R09** 행이에요. — accept: 확인이에요.\n",
        "## 목표\n",
        "### 추가 배경 (2026-10-01) — status: pending-approval\n",
    ] {
        let out = background(&t, content);
        refused(&t, REQUEST, out, "line 1 of the content");
    }

    // A legacy-layout request has no section to append to.
    let legacy = format!(
        "{FRONT}# 옛 요청서예요\n\n옛 요청서의 요약 문단이에요.\n\n## 요구사항\n\n\
         - [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n"
    );
    let t = scratch(&legacy);
    stamp(&t);
    merge_row(&t);
    keep(&t, REQUEST);
    let out = background(&t, ADDED);
    refused(&t, REQUEST, out, "no '## 배경과 문제' heading");

    // A quick task has no Goal background.
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "qq"]);
    let request = ".dstack/quick/qq/request.md";
    keep(&t, request);
    t.write("added.md", ADDED);
    let out = t.run(&[
        "request",
        "background",
        "--quick",
        "qq",
        "--from",
        "added.md",
    ]);
    refused(&t, request, out, "quick");
}

/// The stamp an approval made before the PRD checks left behind.
fn stamp(t: &Scratch) {
    let hash = dstack_cli::core::fsx::sha256_file(&t.0.join(REQUEST)).expect("the request");
    t.write(
        STAMP,
        &format!("sha256 {hash}  approved_at 2026-09-13T20:31:44Z\n"),
    );
}

#[test]
fn R09_merge_background_a_legacy_layout_request_approves_a_merged_row_without_it() {
    let legacy = format!(
        "{FRONT}# 옛 요청서예요\n\n옛 요청서의 요약 문단이에요.\n\n## 요구사항\n\n\
         - [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n"
    );
    let t = scratch(&legacy);
    stamp(&t);
    merge_row(&t);
    let out = t.run(&["check", "request", "--run", RUN]);
    assert!(!stdout(&out).contains(MISSING), "{}", stdout(&out));
    t.ok(&["request", "approve", "--run", RUN]);
    t.ok(&["check", "request", "--run", RUN]);
}

fn fixture(name: &str) -> PathBuf {
    let home = Home::resolve().expect("the repository of this test binary");
    home.home.join("lint/fixtures/check-request").join(name)
}

#[test]
fn R09_merge_background_fixtures_reject_both_halves_of_the_rule() {
    let mut ctx = Context::new(
        Home::resolve().expect("the repository of this test binary"),
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let checkers = verbs::request::selftests();
    let checker = checkers
        .iter()
        .find(|checker| checker.checker() == "check-request")
        .expect("the check-request checker");
    for name in [
        "bad-pending-row-without-background.md",
        "bad-background-without-pending-row.md",
    ] {
        let verdict = checker.run(&mut ctx, &fixture(name)).expect("a verdict");
        assert_eq!(verdict, Verdict::Reject, "{name}");
    }
    // The runner checks in full mode, where a pending row fails on its own; the line this rule
    // prints is what proves the first fixture is rejected for the missing background.
    let text = std::fs::read_to_string(fixture("bad-pending-row-without-background.md"))
        .expect("the fixture");
    let t = scratch(&text);
    stamp(&t);
    let out = t.run(&["check", "request", "--run", RUN]);
    assert!(stdout(&out).contains(MISSING), "{}", stdout(&out));
}
