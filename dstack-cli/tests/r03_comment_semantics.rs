// tests/r03_comment_semantics.rs
// R03, R13 and R09 read HTML comments as the section outline does: `<!-->` and `<!--->` close
// themselves, so prose after one fills a part-1 section (check request), a part-2 section (the
// design gate of request approve) and a background block (request background), and a row after
// one counts; a section holding only template guidance still fails as empty, and guidance kept
// beside such prose is still caught.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::process::Output;

use support::Scratch;

const RUN: &str = "20261001T000000Z_comment";
const REQUEST: &str = ".dstack/runs/20261001T000000Z_comment/request.md";
const STAMP: &str = ".dstack/runs/20261001T000000Z_comment/request.approved";

/// The prose of the sections these tests rewrite.
const BACKGROUND: &str = "처음 승인한 배경이에요.";
const PROPOSED: &str = "바꿀 구조를 적었어요.";
const ROW: &str = "- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.";

/// A filled Goal request in the PRD layout, part 2 included.
fn goal(design_review: &str) -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: {design_review}\nreview: on\ncodex_effort: high\ne2e: cli\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\
         # 주석 읽기 시험\n\n스스로 닫히는 주석을 시험하는 요청서예요.\n\n\
         ## 한눈에 보기\n\n<!-- 승인 전에 채워요. -->\n\n# 1부 요청\n\n\
         ## 배경과 문제\n\n{BACKGROUND}\n\n\
         ## 목표\n\n1. 주석 뒤의 글을 읽어요.\n\n\
         ## 비목표\n\n1. 주석 문법을 넓히지 않아요.\n\n\
         ## 사용 시나리오\n\n### S1 검사해요\n\n메인이 검사를 돌려요.\n\n\
         ## 요구사항\n\n{ROW}\n\n\
         ## 열린 가정\n\n없음.\n\n# 2부 설계\n\n\
         ## 지금 구조\n\n지금 구조를 적었어요.\n\n\
         ## 바꿀 구조\n\n{PROPOSED}\n\n\
         ## 검토한 대안과 버린 이유\n\n대안을 적었어요.\n\n\
         ## R 행과 모듈의 대응\n\n대응을 적었어요.\n\n\
         ## 위험\n\n위험을 적었어요.\n"
    )
}

/// A store whose one run holds `text` as its request.
fn scratch(text: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
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

/// check request on a Goal request whose background body is `body`.
fn check_background(body: &str) -> Output {
    let t = scratch(&goal("skip").replacen(BACKGROUND, body, 1));
    t.run(&["check", "request", "--run", RUN])
}

#[test]
fn R03_comment_semantics_prose_after_a_self_closing_comment_fills_a_section() {
    for opener in ["<!-->", "<!--->"] {
        let out = check_background(&format!("{opener} 글이에요."));
        assert_eq!(out.status.code(), Some(0), "{opener}: {}", stdout(&out));
    }
}

#[test]
fn R03_comment_semantics_guidance_alone_still_fails_as_empty() {
    for body in [
        "<!-- 배경을 적어요. (키: background) -->",
        "<!--> <!-- 배경을 적어요. (키: background) -->",
    ] {
        let out = check_background(body);
        assert_eq!(out.status.code(), Some(1), "{body}: {}", stdout(&out));
        assert!(
            stdout(&out).contains("section ## 배경과 문제: empty or template guidance only"),
            "{body}: {}",
            stdout(&out)
        );
    }
}

#[test]
fn R03_comment_semantics_guidance_beside_prose_after_a_self_closing_comment_is_caught() {
    let out = check_background("<!--> 글이에요.\n<!-- 배경을 적어요. (키: background) -->");
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section ## 배경과 문제: template guidance still present"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn R03_comment_semantics_a_row_after_a_self_closing_comment_counts() {
    let text = goal("skip").replacen(ROW, &format!("<!-->\n{ROW}"), 1);
    let out = scratch(&text).run(&["check", "request", "--run", RUN]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
}

#[test]
fn R13_comment_semantics_prose_after_a_self_closing_comment_fills_a_part_two_section() {
    let text = goal("required").replacen(PROPOSED, &format!("<!--> {PROPOSED}"), 1);
    let t = scratch(&text);
    let out = t.run(&["request", "approve", "--run", RUN]);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("part 2: 5 of 5 sections filled"),
        "{}",
        stdout(&out)
    );
    assert!(t.0.join(STAMP).exists());
}

#[test]
fn R09_comment_semantics_request_background_takes_prose_after_a_self_closing_comment() {
    let t = scratch(&goal("skip"));
    t.ok(&["request", "approve", "--run", RUN]);
    t.ok(&[
        "req",
        "add",
        "덧붙인 요구사항이에요.",
        "--accept",
        "덧붙인 확인이에요.",
        "--run",
        RUN,
    ]);
    t.write("added.md", "<!--> 추가 배경이에요.\n");
    let out = t.run(&["request", "background", "--run", RUN, "--from", "added.md"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let out = t.run(&["check", "request", "--run", RUN]);
    assert!(
        stdout(&out).contains("\n  background: pending rows 1, pending background blocks 1\n"),
        "{}",
        stdout(&out)
    );
    t.ok(&["request", "approve", "--run", RUN]);
    assert!(t.read(REQUEST).contains("<!--> 추가 배경이에요."));
}
