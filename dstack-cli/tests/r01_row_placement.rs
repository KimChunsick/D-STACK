// tests/r01_row_placement.rs
// R01: with the PRD layout `## 요구사항` is no longer the last section, so `req add` has to put the
// first row at the end of that section instead of the end of the file. Documents without the
// heading, and quick requests that end with it, keep the end-of-file placement they always had.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use support::Scratch;

const RUN: &str = "20260930T000000Z_rows";
const REQUEST: &str = ".dstack/runs/20260930T000000Z_rows/request.md";

const FRONT: &str = "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\n\
risk_axes: none\ndesign_review: auto\nreview: on\ncodex_effort: high\ne2e: cli\n\
unit_tests: on\nvisual: none\nkorean_polish: on\n---\n";

/// The new layout up to the last line of `## 요구사항`. The fence in 배경과 문제 holds a
/// `## 요구사항` of its own that is code, not the heading.
const UPPER: &str = "# 행 위치 시험\n\n요약 문단이에요.\n\n## 한눈에 보기\n\n\
<!-- request brief가 채워요. -->\n\n# 1부 요청\n\n## 배경과 문제\n\n예전 문서 모양이에요.\n\n\
```\n# 1부 요청\n## 요구사항\n```\n\n## 목표\n\n1. 첫 행이 요구사항 절에 들어가요.\n\n\
## 요구사항\n\n<!-- dstack req add로 행을 추가해요. 직접 작성하지 않아요. -->\n\
<!-- 진행 중인 요구사항은 12개가 상한이에요(R43). -->\n";

/// Everything after the 요구사항 section: the blank line, 열린 가정 and part 2.
const LOWER: &str = "\n## 열린 가정\n\n없음.\n\n# 2부 설계\n\n## 지금 구조\n\n<!-- 안내 -->\n\n\
## 위험\n\n<!-- 안내 -->\n";

const FIRST: &str = "- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.";

/// A store with one run whose request.md is `text`.
fn scratch(text: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(REQUEST, text);
    t
}

fn add(t: &Scratch, target: &[&str], text: &str, accept: &str) {
    let mut args = vec!["req", "add", text, "--accept", accept];
    args.extend_from_slice(target);
    t.ok(&args);
}

fn add_first(t: &Scratch, target: &[&str]) {
    add(t, target, "첫 요구사항이에요.", "첫 확인이에요.");
}

#[test]
fn R01_row_placement_first_row_goes_under_requirements() {
    let t = scratch(&format!("{FRONT}{UPPER}{LOWER}"));
    add_first(&t, &["--run", RUN]);
    assert_eq!(t.read(REQUEST), format!("{FRONT}{UPPER}{FIRST}\n{LOWER}"));
}

#[test]
fn R01_row_placement_second_row_follows_the_first() {
    let t = scratch(&format!("{FRONT}{UPPER}{LOWER}"));
    add_first(&t, &["--run", RUN]);
    add(&t, &["--run", RUN], "둘째 요구사항이에요.", "둘째 확인이에요.");
    let second = "- [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요.";
    assert_eq!(
        t.read(REQUEST),
        format!("{FRONT}{UPPER}{FIRST}\n{second}\n{LOWER}")
    );
}

#[test]
fn R01_row_placement_legacy_without_heading_appends_at_the_end() {
    // No 요구사항 heading outside the fence: the fenced one must not become the anchor.
    let before = format!(
        "{FRONT}# 옛 요청\n\n요약이에요.\n\n## 메모\n\n```\n## 요구사항\n```\n\n## 기타\n\n내용이에요.\n\n"
    );
    let t = scratch(&before);
    add_first(&t, &["--run", RUN]);
    assert_eq!(t.read(REQUEST), format!("{before}{FIRST}\n"));
}

#[test]
fn R01_row_placement_quick_requirements_last_appends_at_the_end() {
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "q1"]);
    let request = ".dstack/quick/q1/request.md";
    // The section is the last one and a blank line trails it: the row still goes at the end.
    let before = format!(
        "{FRONT}# 빠른 작업: q1\n\n요약 문단이에요.\n\n## 요구사항\n\n<!-- dstack req add로 행을 추가해요. -->\n\n"
    );
    t.write(request, &before);
    add_first(&t, &["--quick", "q1"]);
    assert_eq!(t.read(request), format!("{before}{FIRST}\n"));
}

#[test]
fn R01_row_placement_pending_approval_follows_the_last_row() {
    let t = scratch(&format!("{FRONT}{UPPER}{LOWER}"));
    add_first(&t, &["--run", RUN]);
    // `req add` reads an approved request from the stamp's presence alone.
    t.write(&format!(".dstack/runs/{RUN}/request.approved"), "");
    add(&t, &["--run", RUN], "둘째 요구사항이에요.", "둘째 확인이에요.");
    let pending =
        "- [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요. — status: pending-approval";
    assert_eq!(
        t.read(REQUEST),
        format!("{FRONT}{UPPER}{FIRST}\n{pending}\n{LOWER}")
    );
}
