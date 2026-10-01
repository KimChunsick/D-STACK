// tests/r03_comment_semantics.rs
// R03, R13 and R09 read HTML comments as the section outline does: `<!-->` and `<!--->` close
// themselves, so prose after one fills a part-1 section (check request), a part-2 section (the
// design gate of request approve) and a background block (request background), and a row after
// one counts; a section holding only template guidance still fails as empty, and guidance kept
// beside such prose is still caught. R01 and R03 also read a row inside a comment as no row
// (D-33): `req add` puts the next row after the last visible one and never reuses the hidden id,
// and check request, request approve and the row verbs pass it by. A write never changes what a
// comment hides: `req add` and `request approve` refuse where it would, and write nothing.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::process::Output;

use dstack_cli::store::visible;
use support::Scratch;

const RUN: &str = "20261001T000000Z_comment";
const REQUEST: &str = ".dstack/runs/20261001T000000Z_comment/request.md";
const STAMP: &str = ".dstack/runs/20261001T000000Z_comment/request.approved";

/// The prose of the sections these tests rewrite.
const BACKGROUND: &str = "처음 승인한 배경이에요.";
const PROPOSED: &str = "바꿀 구조를 적었어요.";
const ROW: &str = "- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.";
/// A draft row kept back in a comment: read, it would fail check request for its pending accept.
const HIDDEN: &str = "- [ ] **R02** 아직 다듬지 않은 행이에요. — accept: pending: agent to propose";

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

/// The filled Goal request with `hidden` in a multiline comment right after the visible R01.
fn commented(hidden: &str) -> String {
    goal("skip").replacen(ROW, &format!("{ROW}\n<!--\n{hidden}\n-->"), 1)
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

#[test]
fn R01_comment_semantics_add_lands_after_the_visible_row_outside_the_comment() {
    let before = commented(HIDDEN);
    let t = scratch(&before);
    let add = ["req", "add", "새 요구사항이에요.", "--accept", "새 확인이에요."];
    // The commented R02 is spent: an explicit --id R02 is refused and nothing is written.
    let out = t.run(&[&add[..], &["--id", "R02", "--run", RUN]].concat());
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert_eq!(t.read(REQUEST), before);
    t.ok(&[&add[..], &["--run", RUN]].concat());
    let added = "- [ ] **R03** 새 요구사항이에요. — accept: 새 확인이에요.";
    assert_eq!(
        t.read(REQUEST),
        before.replacen(ROW, &format!("{ROW}\n{added}"), 1)
    );
}

#[test]
fn R01_comment_semantics_add_refuses_where_a_comment_opens_on_the_last_row() {
    // R01 opens a comment its next line closes: after R01 is inside it, so nothing is written.
    let before = goal("skip").replacen(ROW, &format!("{ROW} <!--\n-->"), 1);
    let t = scratch(&before);
    let out = t.run(&["req", "add", "새 요구사항이에요.", "--accept", "새 확인이에요.", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("an HTML comment that opens on R01"),
        "{}",
        stderr(&out)
    );
    assert_eq!(t.read(REQUEST), before);
}

#[test]
fn R03_comment_semantics_approve_refuses_a_marker_that_opens_a_comment() {
    // Clearing R01's marker would drop the `<!--` with it and bring the unchecked draft out.
    let t = scratch(&goal("skip"));
    t.ok(&["request", "approve", "--run", RUN]);
    let stamp = t.read(STAMP);
    let marked = format!("{ROW} — status: pending-approval <!--\n{HIDDEN}\n-->");
    let before = goal("skip").replacen(ROW, &marked, 1);
    t.write(REQUEST, &before);
    let out = t.run(&["request", "approve", "--run", RUN]);
    assert_ne!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(stderr(&out).contains("R01: clearing"), "{}", stderr(&out));
    assert_eq!(t.read(REQUEST), before);
    assert_eq!(t.read(STAMP), stamp);
}

#[test]
fn R03_comment_semantics_many_comments_read_line_by_line() {
    // 2,000 one-line comments, then 2,000 that close on the next line: every line still reads
    // as it did when each line was cut against every comment of the text.
    let mut text = String::new();
    for n in 0..2000 {
        text.push_str(&format!("앞 {n} <!-- 주석 {n} --> 뒤\n"));
    }
    let one_line = text.len();
    for n in 0..2000 {
        text.push_str(&format!("앞 {n} <!-- 열어요\n닫아요 --> 뒤 {n}\n"));
    }
    let lines = visible::lines(&text);
    assert_eq!(lines.len(), 6001);
    for (n, line) in lines[..2000].iter().enumerate() {
        assert!(!line.hidden && !line.heading, "line {n}");
        assert_eq!(line.shown, format!("앞 {n}  뒤"));
    }
    for (n, pair) in lines[2000..6000].chunks(2).enumerate() {
        assert!(!pair[0].hidden && pair[1].hidden, "pair {n}");
        assert_eq!(pair[0].shown, format!("앞 {n} "));
        assert_eq!(pair[1].shown, format!(" 뒤 {n}"));
    }
    assert!(!lines[6000].hidden && lines[6000].shown.is_empty());
    let shown: Vec<&str> = lines[..2000].iter().map(|line| line.shown.as_str()).collect();
    assert_eq!(visible::visible(&text[..one_line]), format!("{}\n", shown.join("\n")));
}

#[test]
fn R03_comment_semantics_check_counts_only_the_visible_row() {
    let out = scratch(&commented(HIDDEN)).run(&["check", "request", "--run", RUN]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("\n  rows: 1 (live 1, pending 0, "),
        "{}",
        stdout(&out)
    );
}

#[test]
fn R03_comment_semantics_approve_leaves_a_commented_pending_row_alone() {
    let hidden = "- [ ] **R02** 보류한 행이에요. — accept: 보류한 확인이에요. — status: pending-approval";
    let t = scratch(&commented(hidden));
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
    t.write("added.md", "추가 배경이에요.\n");
    t.ok(&["request", "background", "--run", RUN, "--from", "added.md"]);
    let out = t.ok(&["request", "approve", "--run", RUN]);
    assert!(out.contains(", pending cleared 1, "), "{out}");
    let added = "- [ ] **R03** 덧붙인 요구사항이에요. — accept: 덧붙인 확인이에요.";
    let text = t.read(REQUEST);
    assert!(
        text.contains(&format!("{ROW}\n{added}\n<!--\n{hidden}\n-->\n")),
        "{text}"
    );
}

#[test]
fn R03_comment_semantics_row_verbs_find_no_commented_row() {
    let before = commented(HIDDEN);
    let t = scratch(&before);
    for args in [
        &["req", "accept", "R02", "새 확인이에요.", "--run", RUN][..],
        &["req", "withdraw", "R02", "--why", "필요 없어졌어요.", "--run", RUN],
        &["req", "split", "R02", "--into", "R03,R04", "--run", RUN],
    ] {
        let out = t.run(args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stdout(&out));
        assert!(stderr(&out).contains("no row R02 in "), "{args:?}: {}", stderr(&out));
        assert_eq!(t.read(REQUEST), before, "{args:?}");
    }
}
