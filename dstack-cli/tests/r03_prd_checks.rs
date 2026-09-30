// tests/r03_prd_checks.rs
// R03: check request fails a Goal request whose required part-1 section is missing, holds only
// guidance or keeps its guidance beside prose, a quick request whose summary does (its fallback body
// included), and either one with no R row inside its requirements section; the 60-line cap counts
// that section alone (`## 요구사항` or the legacy `## Requirements`), or the row span when neither
// heading exists; an approved request is not judged again while its text matches the stamp, is
// judged again once changed unless it is a Goal request with none of the PRD layout headings (a
// quick one always is), and then hints an edit and a new approval; and the check-request fixtures
// prove every case.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::rc::Rc;

use dstack_cli::core::context::Context;
use dstack_cli::core::fsx::sha256_file;
use dstack_cli::core::registry::Registry;
use dstack_cli::core::roots::Home;
use dstack_cli::verbs;
use dstack_cli::verbs::doctor::selfrun;
use support::Scratch;

const RUN: &str = "20260930T000000Z_prd";
const REQUEST: &str = ".dstack/runs/20260930T000000Z_prd/request.md";

const FRONT: &str = "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\n\
risk_axes: none\ndesign_review: auto\nreview: on\ncodex_effort: high\ne2e: cli\n\
unit_tests: on\nvisual: none\nkorean_polish: on\n---\n";

/// The five part-1 sections a Goal request fills before approval, as (key, heading).
const REQUIRED: [(&str, &str); 5] = [
    ("background", "배경과 문제"),
    ("goals", "목표"),
    ("non-goals", "비목표"),
    ("scenarios", "사용 시나리오"),
    ("assumptions", "열린 가정"),
];

/// A filled Goal request as (heading line, body) in file order; part 2 keeps its guidance.
fn goal() -> Vec<(&'static str, String)> {
    [
        ("# PRD 검사 시험", "\n검사를 시험하는 요청서예요.\n\n"),
        ("## 한눈에 보기", "\n<!-- 승인 전에 채워요. -->\n\n"),
        ("# 1부 요청", "\n"),
        ("## 배경과 문제", "\n빈 절이 승인된 적이 있어요.\n\n"),
        ("## 목표", "\n1. 빈 절을 검사에서 막아요.\n\n"),
        ("## 비목표", "\n1. 2부는 이번에 검사하지 않아요.\n\n"),
        (
            "## 사용 시나리오",
            "\n### S1 검사해요\n\n메인이 검사를 돌려요.\n\n",
        ),
        (
            "## 요구사항",
            "\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n",
        ),
        ("## 열린 가정", "\n없음.\n\n"),
        ("# 2부 설계", "\n"),
        ("## 지금 구조", "\n<!-- 안내 -->\n\n"),
        ("## 위험", "\n<!-- 안내 -->\n"),
    ]
    .into_iter()
    .map(|(heading, body)| (heading, body.to_string()))
    .collect()
}

fn render(parts: &[(&str, String)]) -> String {
    let mut text = FRONT.to_string();
    for (heading, body) in parts {
        text.push_str(heading);
        text.push('\n');
        text.push_str(body);
    }
    text
}

/// `parts` with the body under `heading` replaced.
fn with_body(heading: &str, body: &str) -> Vec<(&'static str, String)> {
    let mut parts = goal();
    let at = parts
        .iter()
        .position(|(h, _)| *h == heading)
        .expect("a heading");
    parts[at].1 = body.to_string();
    parts
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

fn check(t: &Scratch) -> Output {
    t.run(&["check", "request", "--run", RUN])
}

#[test]
fn R03_prd_checks_refuse_a_fresh_goal_request_until_part_one_is_filled() {
    let t = Scratch::new();
    t.init();
    t.ok(&["run", "new", "prd", "--type", "cli"]);
    t.ok(&[
        "request",
        "new",
        "--type",
        "cli",
        "--title",
        "PRD 검사 시험",
    ]);
    t.ok(&[
        "req",
        "add",
        "첫 요구사항이에요.",
        "--accept",
        "첫 확인이에요.",
    ]);
    let out = t.run(&["check", "request"]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    for (_, heading) in REQUIRED {
        assert!(
            stdout(&out).contains(&format!("section ## {heading}: empty")),
            "{heading}: {}",
            stdout(&out)
        );
    }
    // approve runs the same check, so an unfilled request cannot be stamped either.
    let out = t.run(&["request", "approve"]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    let id = t.read(".dstack/local/CURRENT").trim_end().to_string();
    assert!(!t
        .0
        .join(format!(".dstack/runs/{id}/request.approved"))
        .exists());

    for (index, (key, heading)) in REQUIRED.iter().enumerate() {
        t.write("section.md", &format!("{heading} 절을 채웠어요.\n"));
        t.ok(&["request", "section", key, "--from", "section.md"]);
        let code = t.run(&["check", "request"]).status.code();
        let want = if index + 1 < REQUIRED.len() { 1 } else { 0 };
        assert_eq!(code, Some(want), "after {key}");
    }
    t.ok(&["request", "approve"]);
    t.ok(&["check", "request"]);
}

#[test]
fn R03_prd_checks_count_comment_only_blank_and_missing_sections_as_empty() {
    let out = check(&scratch(&render(&goal())));
    assert_eq!(
        out.status.code(),
        Some(0),
        "the filled request: {}",
        stdout(&out)
    );
    for (_, heading) in REQUIRED {
        let line = format!("## {heading}");
        for body in [
            "\n<!-- 안내만 남았어요. -->\n<!-- 둘째 줄이에요. -->\n\n",
            "\n\n",
        ] {
            let out = check(&scratch(&render(&with_body(&line, body))));
            assert_eq!(out.status.code(), Some(1), "{heading} {body:?}");
            assert!(
                stdout(&out).contains(&format!("section {line}: empty")),
                "{heading}: {}",
                stdout(&out)
            );
        }
        let mut parts = goal();
        parts.retain(|(h, _)| *h != line);
        let out = check(&scratch(&render(&parts)));
        assert_eq!(out.status.code(), Some(1), "{heading} missing");
        assert!(
            stdout(&out).contains(&format!("no '{line}' heading")),
            "{heading}: {}",
            stdout(&out)
        );
    }
}

#[test]
fn R03_prd_checks_quick_needs_only_its_summary() {
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "qq"]);
    t.ok(&[
        "req",
        "add",
        "첫 요구사항이에요.",
        "--accept",
        "첫 확인이에요.",
        "--quick",
        "qq",
    ]);
    let quick = ["--quick", "qq"];
    let out = t.run(&["check", "request", quick[0], quick[1]]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stdout(&out).contains("section the paragraph under # 빠른 작업: qq: empty"));
    assert_eq!(
        t.run(&["request", "approve", quick[0], quick[1]])
            .status
            .code(),
        Some(1)
    );
    // Blank is as empty as guidance.
    let request = ".dstack/quick/qq/request.md";
    let text = t.read(request);
    let guidance = text
        .lines()
        .find(|l| l.starts_with("<!-- "))
        .expect("guidance");
    t.write(request, &text.replacen(&format!("{guidance}\n"), "", 1));
    assert_eq!(
        t.run(&["check", "request", quick[0], quick[1]])
            .status
            .code(),
        Some(1)
    );

    t.write("section.md", "요약 문단을 채웠어요.\n");
    t.ok(&[
        "request",
        "section",
        "summary",
        "--from",
        "section.md",
        quick[0],
        quick[1],
    ]);
    t.ok(&["check", "request", quick[0], quick[1]]);
    t.ok(&["request", "approve", quick[0], quick[1]]);
}

#[test]
fn R03_prd_checks_count_only_requirement_lines() {
    let long: String = (1..=80)
        .map(|n| format!("배경 {n}번째 줄이에요.\n"))
        .collect();
    let out = check(&scratch(&render(&with_body(
        "## 배경과 문제",
        &format!("\n{long}\n"),
    ))));
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(
        !stderr(&out).contains("> 60"),
        "long prose warned: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("requirement lines 3 (max 60)"),
        "{}",
        stdout(&out)
    );

    let rows: String = (1..=61).map(|n| format!("<!-- 안내 {n} -->\n")).collect();
    let body = format!("\n{rows}- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n");
    let out = check(&scratch(&render(&with_body("## 요구사항", &body))));
    assert_eq!(
        out.status.code(),
        Some(0),
        "the cap only warns: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("requirement lines 64 > 60"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn R03_prd_checks_pass_an_approved_legacy_request() {
    // The template every request was written from before the PRD layout: a guidance paragraph
    // and the rows, nothing else.
    let legacy = format!(
        "{FRONT}# 옛 요청서예요\n\n어떤 명령을 누가 실행하는지 한 문단으로 적어요.\n\n## 요구사항\n\n\
         <!-- 행을 추가해요. -->\n\
         - [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n"
    );
    let t = scratch(&legacy);
    assert_eq!(check(&t).status.code(), Some(1), "unapproved, it is judged");
    let run = t.0.join(".dstack/runs").join(RUN);
    stamp(&run);
    let out = check(&t);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    // A row merged into it later is re-approved without the sections being asked for.
    t.ok(&[
        "req",
        "add",
        "덧붙인 요구사항이에요.",
        "--accept",
        "덧붙인 확인이에요.",
        "--run",
        RUN,
    ]);
    let out = check(&t);
    assert_eq!(out.status.code(), Some(1), "the pending row");
    // Changed since its stamp, it is still exempt: it holds none of the PRD layout headings.
    assert!(stdout(&out).contains(LEGACY_LINE), "{}", stdout(&out));
    let approved = t.ok(&["request", "approve", "--run", RUN]);
    assert!(approved.contains(LEGACY_LINE), "{approved}");
    t.ok(&["check", "request", "--run", RUN]);
}

const LEGACY_LINE: &str = "sections: not judged (approved without the PRD layout";

/// A store whose filled Goal request `request approve` has stamped, and the run directory.
fn approved_goal() -> (Scratch, PathBuf) {
    let t = scratch(&render(&goal()));
    t.ok(&["request", "approve", "--run", RUN]);
    let run = t.0.join(".dstack/runs").join(RUN);
    (t, run)
}

#[test]
fn R03_prd_checks_leave_an_unchanged_approved_request_unjudged() {
    let (t, _) = approved_goal();
    let out = check(&t);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("sections: not judged (approved and unchanged since"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn R03_prd_checks_judge_again_a_request_emptied_after_approval() {
    let (t, run) = approved_goal();
    let stamped = std::fs::read(run.join("request.approved")).expect("the stamp");
    t.write(REQUEST, &render(&with_body("## 목표", "\n")));
    let out = t.run(&["request", "approve", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section ## 목표: empty"),
        "{}",
        stdout(&out)
    );
    // request section refuses an approved request, so the hint names the edit and approve.
    assert!(stdout(&out).contains(EDIT_HINT), "{}", stdout(&out));
    assert!(
        !stdout(&out).contains("request section"),
        "{}",
        stdout(&out)
    );
    let now = std::fs::read(run.join("request.approved")).expect("the stamp");
    assert_eq!(now, stamped, "a refused approval leaves the stamp alone");
    let out = check(&t);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(stdout(&out).contains("section ## 목표: empty"));
}

const EDIT_HINT: &str = "(edit it in request.md, then dstack request approve again)";

#[test]
fn R03_prd_checks_judge_again_an_edited_approved_quick_request() {
    let t = Scratch::new();
    t.init();
    let quick = ["--quick", "qq"];
    t.ok(&["quick", "new", "qq"]);
    t.ok(&[
        "req",
        "add",
        "첫 요구사항이에요.",
        "--accept",
        "첫 확인이에요.",
        quick[0],
        quick[1],
    ]);
    let prose = "요약 문단을 채웠어요.\n";
    t.write("section.md", prose);
    t.ok(&[
        "request",
        "section",
        "summary",
        "--from",
        "section.md",
        quick[0],
        quick[1],
    ]);
    t.ok(&["request", "approve", quick[0], quick[1]]);
    let stamp = t.0.join(".dstack/quick/qq/request.approved");
    let stamped = std::fs::read(&stamp).expect("the stamp");
    let out = t.run(&["check", "request", quick[0], quick[1]]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("sections: not judged (approved and unchanged since"),
        "{}",
        stdout(&out)
    );

    // A quick request never has the PRD layout headings, yet an edit is judged all the same.
    let request = ".dstack/quick/qq/request.md";
    t.write(request, &t.read(request).replacen(prose, "", 1));
    let out = t.run(&["request", "approve", quick[0], quick[1]]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section the paragraph under # 빠른 작업: qq: empty"),
        "{}",
        stdout(&out)
    );
    assert!(stdout(&out).contains(EDIT_HINT), "{}", stdout(&out));
    let now = std::fs::read(&stamp).expect("the stamp");
    assert_eq!(now, stamped, "a refused approval leaves the stamp alone");
}

#[test]
fn R03_prd_checks_reapprove_a_filled_request_after_a_merged_row() {
    let (t, _) = approved_goal();
    t.ok(&[
        "req",
        "add",
        "덧붙인 요구사항이에요.",
        "--accept",
        "덧붙인 확인이에요.",
        "--run",
        RUN,
    ]);
    // Changed since its stamp, it is judged again and passes because every section is filled.
    let approved = t.ok(&["request", "approve", "--run", RUN]);
    assert!(
        approved.contains("sections: required 6, unfilled 0"),
        "{approved}"
    );
    t.ok(&["check", "request", "--run", RUN]);
}

const ROW: &str = "- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n";

/// check request on a quick task whose request is `summary` under its title and `rows` under
/// 요구사항.
fn quick(summary: &str, rows: &str) -> Output {
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "qq"]);
    let front = FRONT.replace("route: new-goal", "route: quick");
    t.write(
        ".dstack/quick/qq/request.md",
        &format!("{front}# 빠른 작업: qq\n\n{summary}\n## 요구사항\n\n{rows}"),
    );
    t.run(&["check", "request", "--quick", "qq"])
}

#[test]
fn R03_prd_checks_require_a_row() {
    // The template's standing guidance under 요구사항 is an instruction, not a row.
    let guidance =
        "\n<!-- dstack req add \"<한국어 요구사항>\" --accept \"<한국어 완료 기준>\"으로 행을 추가해요. -->\n";
    let out = check(&scratch(&render(&with_body(
        "## 요구사항",
        &format!("{guidance}\n"),
    ))));
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section ## 요구사항: no R row (dstack req add"),
        "{}",
        stdout(&out)
    );
    let out = check(&scratch(&render(&with_body(
        "## 요구사항",
        &format!("{guidance}{ROW}\n"),
    ))));
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));

    let out = quick("요약 문단을 채웠어요.\n", "");
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section ## 요구사항: no R row"),
        "{}",
        stdout(&out)
    );
    let out = quick("요약 문단을 채웠어요.\n", ROW);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
}

#[test]
fn R03_prd_checks_refuse_guidance_kept_beside_prose() {
    let goal =
        "<!-- 이번 작업이 끝나면 누가 무엇을 할 수 있게 되는지 번호 목록으로 적어요. (키: goals) -->\n";
    let prose = "1. 빈 절을 검사에서 막아요.\n";
    let out = check(&scratch(&render(&with_body(
        "## 목표",
        &format!("\n{goal}{prose}\n"),
    ))));
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section ## 목표: template guidance still present"),
        "{}",
        stdout(&out)
    );
    let out = check(&scratch(&render(&with_body(
        "## 목표",
        &format!("\n{prose}\n"),
    ))));
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));

    let summary = "<!-- 무엇을 왜 바꾸는지 한 문단으로 적어요. (키: summary) -->\n요약 문단을 채웠어요.\n";
    let out = quick(summary, ROW);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out)
            .contains("section the paragraph under # 빠른 작업: qq: template guidance still present"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn R03_prd_checks_count_the_whole_english_requirements_section() {
    let long: String = (1..=80)
        .map(|n| format!("배경 {n}번째 줄이에요.\n"))
        .collect();
    let text = render(&with_body("## 배경과 문제", &format!("\n{long}\n")))
        .replace("## 요구사항\n", "## Requirements\n");
    let out = check(&scratch(&text));
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(
        !stderr(&out).contains("requirement lines"),
        "long prose warned: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("requirement lines 3 (max 60)"),
        "{}",
        stdout(&out)
    );

    // Guidance above the first row is part of the section, as under 요구사항.
    let guidance: String = (1..=61).map(|n| format!("<!-- 안내 {n} -->\n")).collect();
    let rows = format!(
        "\n{guidance}{ROW}- [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요.\n\
         - [ ] **R03** 셋째 요구사항이에요. — accept: 셋째 확인이에요.\n\n"
    );
    let text =
        render(&with_body("## 요구사항", &rows)).replace("## 요구사항\n", "## Requirements\n");
    let out = check(&scratch(&text));
    assert_eq!(
        out.status.code(),
        Some(0),
        "the cap only warns: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("requirement lines 66 > 60"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn R03_prd_checks_count_the_row_span_without_a_requirements_heading() {
    let long: String = (1..=80)
        .map(|n| format!("배경 {n}번째 줄이에요.\n"))
        .collect();
    let text = render(&with_body("## 배경과 문제", &format!("\n{long}\n")))
        .replace("## 요구사항\n", "## R rows\n");
    let out = check(&scratch(&text));
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    assert!(
        !stderr(&out).contains("requirement lines"),
        "long prose warned: {}",
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("requirement lines 1 (max 60)"),
        "{}",
        stdout(&out)
    );

    let gap: String = (1..=59).map(|n| format!("<!-- 안내 {n} -->\n")).collect();
    let rows =
        format!("\n{ROW}{gap}- [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요.\n\n");
    let text = render(&with_body("## 요구사항", &rows)).replace("## 요구사항\n", "## R rows\n");
    let out = check(&scratch(&text));
    assert_eq!(
        out.status.code(),
        Some(0),
        "the cap only warns: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("requirement lines 61 > 60"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn R03_prd_checks_require_the_row_inside_the_requirements_section() {
    let guidance = "\n<!-- 행을 추가해요. -->\n\n";
    let outside = with_body("## 요구사항", guidance);
    let mut parts = outside.clone();
    let at = parts.iter().position(|(h, _)| *h == "## 열린 가정").unwrap();
    parts[at].1 = format!("\n없음.\n\n{ROW}\n");
    let out = check(&scratch(&render(&parts)));
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section ## 요구사항: no R row (dstack req add"),
        "{}",
        stdout(&out)
    );
    // The same row moved under 요구사항.
    let out = check(&scratch(&render(&with_body(
        "## 요구사항",
        &format!("{guidance}{ROW}\n"),
    ))));
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));

    let text = render(&parts).replace("## 요구사항\n", "## Requirements\n");
    let out = check(&scratch(&text));
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("section ## Requirements: no R row (dstack req add"),
        "{}",
        stdout(&out)
    );

    let mut both = goal();
    both.push((
        "## Requirements",
        "\n- [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요.\n".to_string(),
    ));
    let out = check(&scratch(&render(&both)));
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out).contains("'## 요구사항' and '## Requirements'"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn R03_prd_checks_quick_fallback_summary_keeps_its_marker() {
    let t = Scratch::new();
    t.init();
    // A home without templates/request, so quick new writes its fallback body.
    let home = t.0.join("home-without-templates");
    std::fs::create_dir_all(&home).expect("the empty home");
    let out = Command::new(env!("CARGO_BIN_EXE_dstack"))
        .current_dir(&t.0)
        .env("DSTACK_ROOT", &t.0)
        .env("DSTACK_HOME", &home)
        .env("DSTACK_DEPS", t.0.join("deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "mode-settings-test")
        .args(["quick", "new", "qq"])
        .output()
        .expect("quick new");
    assert!(out.status.success(), "{}", stderr(&out));
    t.ok(&[
        "req",
        "add",
        "첫 요구사항이에요.",
        "--accept",
        "첫 확인이에요.",
        "--quick",
        "qq",
    ]);
    let request = ".dstack/quick/qq/request.md";
    let text = t.read(request);
    let guidance = text
        .lines()
        .find(|l| l.starts_with("<!-- "))
        .expect("the fallback guidance")
        .to_string();
    let prose = "요약 문단을 채웠어요.";
    t.write(
        request,
        &text.replacen(&guidance, &format!("{guidance}\n{prose}"), 1),
    );
    let out = t.run(&["check", "request", "--quick", "qq"]);
    assert_eq!(out.status.code(), Some(1), "{}", stdout(&out));
    assert!(
        stdout(&out)
            .contains("section the paragraph under # 빠른 작업: qq: template guidance still present"),
        "{}",
        stdout(&out)
    );
    t.write(request, &text.replacen(&guidance, prose, 1));
    let out = t.run(&["check", "request", "--quick", "qq"]);
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
}

/// The stamp an approval made before these checks left behind.
fn stamp(run: &Path) {
    let hash = sha256_file(&run.join("request.md")).expect("the request");
    std::fs::write(
        run.join("request.approved"),
        format!("sha256 {hash}  approved_at 2026-09-13T20:31:44Z\n"),
    )
    .expect("the stamp");
}

#[test]
fn R03_prd_checks_fixtures_cover_every_case() {
    let home = Home::resolve().expect("the repository of this test binary");
    let dir: PathBuf = home.home.join("lint/fixtures/check-request");
    let names: Vec<String> = selfrun::fixtures(&dir)
        .iter()
        .map(|(path, _)| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    for name in [
        "bad-both-requirements-headings.md",
        "bad-guidance-only-section.md",
        "bad-missing-section.md",
        "bad-quick-guidance-summary.md",
        "bad-quick-retained-summary.md",
        "bad-retained-guidance.md",
        "bad-row-outside-english-section.md",
        "bad-row-outside-section.md",
        "bad-zero-rows.md",
        "good-legacy-approved.md",
        "good-long-prose.md",
        "good-minimal.md",
        "good-no-requirements-heading.md",
        "good-quick-summary.md",
        "good-standing-guidance.md",
    ] {
        assert!(names.iter().any(|n| n == name), "{name} in {names:?}");
    }
    let mut ctx = Context::new(
        home,
        PathBuf::from(env!("CARGO_BIN_EXE_dstack")),
        Rc::new(Registry::new(verbs::all_verbs())),
    );
    let checkers = verbs::request::selftests();
    let checker = checkers
        .iter()
        .find(|checker| checker.checker() == "check-request")
        .expect("the check-request checker");
    for (fixture, expected) in selfrun::fixtures(&dir) {
        let actual = checker.run(&mut ctx, &fixture).expect("a verdict");
        assert_eq!(actual, expected, "{}", fixture.display());
    }
}
