// tests/r14_request_brief.rs
// R14: `request brief` fills the `## 한눈에 보기` section of an unapproved Goal request with four
// groups — assumptions, design choices, non-goals and affected files — each showing 없음 when
// empty; a second run is byte-identical and every byte outside the section is kept; a quick
// target, an approved request and a request without the heading are refused, nothing written.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::process::Output;

use dstack_cli::store::request_sections::{locate, Place};
use support::Scratch;

const RUN: &str = "20261001T000000Z_brief";
const REQUEST: &str = ".dstack/runs/20261001T000000Z_brief/request.md";
const RECON: &str = ".dstack/runs/20261001T000000Z_brief/recon.md";
const BRIEF: Place = Place::Section("한눈에 보기");

/// A Goal request with every section but 비목표 and the mapping filled.
fn request(non_goals: &str, mapping: &str) -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: auto\nreview: on\ncodex_effort: high\ne2e: cli\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\
         # 한눈에 보기 시험\n\n한눈에 보기를 만드는 요청서예요.\n\n\
         ## 한눈에 보기\n\n<!-- 승인하기 전에 CLI가 채워요. -->\n\n# 1부 요청\n\n\
         ## 배경과 문제\n\n승인 전에 볼 내용이 흩어져 있어요.\n\n\
         ## 목표\n\n1. 판단할 내용을 맨 위에 모아요.\n\n\
         ## 비목표\n\n{non_goals}\n\
         ## 사용 시나리오\n\n### S1 요약을 만들어요\n\n메인이 요약을 만들어요.\n\n\
         ## 요구사항\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n\
         ## 열린 가정\n\n없음.\n\n# 2부 설계\n\n\
         ## 지금 구조\n\n지금 구조를 적었어요.\n\n\
         ## 바꿀 구조\n\n바꿀 구조를 적었어요.\n\n\
         ## 검토한 대안과 버린 이유\n\n버린 대안을 적었어요.\n\n\
         ## R 행과 모듈의 대응\n\n{mapping}\n\
         ## 위험\n\n위험을 적었어요.\n"
    )
}

const NON_GOALS: &str = "1. 이미 승인된 요청서는 다시 쓰지 않아요.\n\
                         <!-- 안내문은 한눈에 보기에 옮기지 않아요. -->\n\n\
                         2. 화면 캡처는 추가하지 않아요.\n";

const MAPPING: &str = "| R | 파일 |\n|---|---|\n\
                       | R01 | `dstack-cli/src/verbs/request/brief.rs`, `claude/templates/request/cli.md` |\n\
                       | R02 | `--run` 옵션과 `R01` 행이에요 |\n";

const RECON_TEXT: &str = "# Recon\n\n## Blast radius\n\n| R | Likely touch points |\n|---|---|\n\
                          | R01 | `claude/templates/request/cli.md`; `quick/new.rs:141-215`; `README.md`; `cargo test` |\n\n\
                          ## Open design forks\n\n- `not/in/the/table.rs`\n";

/// The section body every group of the full fixture fills.
const FULL: &str = "<!-- dstack request brief가 decisions.md, recon.md, 비목표 절, R 행과 모듈의 대응 절에서 모아 만든 절이에요. 고칠 때는 원본을 바꾼 뒤 다시 만들어요. -->\n\n\
                    **대신 정한 가정**\n\n\
                    - D-01 (Q-01): adopted default: 표 형식을 써요. (from Q-01)\n\
                    - D-02 (Q-01): lists are accepted too\n\n\
                    **설계 선택지와 버린 대안**\n\n\
                    - D-DESIGN-01: design round 1: document layout — one request file\n\
                    - D-03 설계를 건너뛴 사유: 작은 변경이에요.\n\n\
                    **비목표**\n\n\
                    1. 이미 승인된 요청서는 다시 쓰지 않아요.\n\
                    2. 화면 캡처는 추가하지 않아요.\n\n\
                    **영향 파일**\n\n\
                    - `claude/templates/request/cli.md`\n\
                    - `quick/new.rs:141-215`\n\
                    - `README.md`\n\
                    - `dstack-cli/src/verbs/request/brief.rs`\n";

const EMPTY: &str = "<!-- dstack request brief가 decisions.md, recon.md, 비목표 절, R 행과 모듈의 대응 절에서 모아 만든 절이에요. 고칠 때는 원본을 바꾼 뒤 다시 만들어요. -->\n\n\
                     **대신 정한 가정**\n\n없음\n\n\
                     **설계 선택지와 버린 대안**\n\n없음\n\n\
                     **비목표**\n\n없음\n\n\
                     **영향 파일**\n\n없음\n";

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn scratch(text: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(REQUEST, text);
    t
}

/// The full fixture: two assumed decisions (one naming its Q, one reaching it through the R row
/// `ask assume` added), a design round, a design skip, an answered decision the brief leaves out,
/// a filled 비목표, a mapping table and a recon.md with a Blast radius table.
fn full() -> Scratch {
    let t = scratch(&request(NON_GOALS, MAPPING));
    t.write(RECON, RECON_TEXT);
    let run = |args: &[&str]| {
        let mut args = args.to_vec();
        args.extend(["--run", RUN]);
        t.ok(&args);
    };
    run(&["ask", "add", "Which layout?", "--affects", "R01"]);
    run(&["ask", "assume", "Q-01", "표 형식을 써요.", "--accept", "표가 아니면 보여요."]);
    run(&["decision", "add", "lists are accepted too", "--affects", "R02", "--assumed"]);
    run(&["decision", "add", "one request file", "--affects", "design", "--design", "document layout"]);
    run(&["request", "design-skip", "--why", "작은 변경이에요."]);
    run(&["decision", "add", "an answered decision stays out", "--affects", "R01"]);
    t
}

fn brief(t: &Scratch) -> Output {
    t.run(&["request", "brief", "--run", RUN])
}

fn section(text: &str) -> &str {
    &text[locate(text, &BRIEF).expect("the brief heading")]
}

/// A refusal: exit 1, a reason on stderr naming `why`, and the request byte-identical.
fn refused(t: &Scratch, request: &str, out: &Output, before: &str, why: &str) {
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(out), stderr(out));
    assert!(stderr(out).contains(why), "{why}: {}", stderr(out));
    assert_eq!(t.read(request), before, "a refused brief writes nothing");
}

#[test]
fn R14_request_brief_collects_the_four_groups() {
    let t = full();
    let out = brief(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    let text = t.read(REQUEST);
    assert_eq!(section(&text), format!("\n{FULL}\n"));
    for line in FULL.lines().filter(|line| !line.is_empty()) {
        assert!(stdout(&out).contains(line), "{line}: {}", stdout(&out));
    }
    assert!(!stdout(&out).contains("an answered decision"), "{}", stdout(&out));
}

#[test]
fn R14_request_brief_prints_none_for_every_empty_group() {
    let t = scratch(&request(
        "<!-- 이번 작업에서 하지 않는 일을 적어요. (키: non-goals) -->\n",
        "<!-- R 행마다 `dstack-cli/src/example.rs`처럼 파일을 적어요. (키: mapping) -->\n",
    ));
    let out = brief(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    assert_eq!(section(&t.read(REQUEST)), format!("\n{EMPTY}\n"));
    assert_eq!(stdout(&out).matches("\n없음\n").count(), 4, "{}", stdout(&out));
}

#[test]
fn R14_request_brief_is_idempotent_and_keeps_every_other_byte() {
    let t = full();
    let before = t.read(REQUEST);
    let old = locate(&before, &BRIEF).unwrap();
    assert_eq!(brief(&t).status.code(), Some(0));
    let first = t.read(REQUEST);
    let new = locate(&first, &BRIEF).unwrap();
    assert_eq!(first[..new.start], before[..old.start]);
    assert_eq!(first[new.end..], before[old.end..]);
    assert_eq!(brief(&t).status.code(), Some(0));
    assert_eq!(t.read(REQUEST), first, "a second run is byte-identical");
}

#[test]
fn R14_request_brief_refuses_a_quick_target() {
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "qq"]);
    let quick = ".dstack/quick/qq/request.md";
    let before = t.read(quick);
    let out = t.run(&["request", "brief", "--quick", "qq"]);
    refused(&t, quick, &out, &before, "quick");
}

#[test]
fn R14_request_brief_refuses_an_approved_request() {
    let t = scratch(&request(NON_GOALS, MAPPING));
    t.ok(&["request", "approve", "--run", RUN]);
    let before = t.read(REQUEST);
    let out = brief(&t);
    refused(&t, REQUEST, &out, &before, "approved");
}

#[test]
fn R14_request_brief_refuses_a_request_without_the_heading() {
    let text = request(NON_GOALS, MAPPING).replace("## 한눈에 보기\n\n<!-- 승인하기 전에 CLI가 채워요. -->\n\n", "");
    let t = scratch(&text);
    let out = brief(&t);
    refused(&t, REQUEST, &out, &text, "## 한눈에 보기");
}
