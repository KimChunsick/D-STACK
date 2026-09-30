// tests/r13_design_gate.rs
// R13: `request approve` on a Goal request asks part 2 for what design_review promises — all five
// sections filled when required, filled or a recorded design-skip reason when auto, nothing when
// skip — and prints the reason whenever a skip applies; `request design-skip --why` records that
// reason as a decision row before approval; quick and legacy-layout requests are not gated; and
// `request section` writes one part-2 section, every other byte kept.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::process::Output;

use support::Scratch;

const RUN: &str = "20261001T000000Z_design";
const REQUEST: &str = ".dstack/runs/20261001T000000Z_design/request.md";
const STAMP: &str = ".dstack/runs/20261001T000000Z_design/request.approved";

/// The five part-2 sections as (key, heading), in template order.
const PART_TWO: [(&str, &str); 5] = [
    ("current", "지금 구조"),
    ("proposed", "바꿀 구조"),
    ("alternatives", "검토한 대안과 버린 이유"),
    ("mapping", "R 행과 모듈의 대응"),
    ("risks", "위험"),
];

/// A Goal request with part 1 filled and each part-2 section holding `part_two(key)`.
fn request(design_review: &str, part_two: impl Fn(&str) -> String) -> String {
    let mut text = format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: {design_review}\nreview: on\ncodex_effort: high\ne2e: cli\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\
         # 설계 확인 시험\n\n설계 확인을 시험하는 요청서예요.\n\n\
         ## 한눈에 보기\n\n<!-- 승인 전에 채워요. -->\n\n# 1부 요청\n\n\
         ## 배경과 문제\n\n설계 없이 승인된 적이 있어요.\n\n\
         ## 목표\n\n1. 설계 절을 승인 전에 확인해요.\n\n\
         ## 비목표\n\n1. 설계 내용을 판단하지 않아요.\n\n\
         ## 사용 시나리오\n\n### S1 승인해요\n\n메인이 요청서를 승인해요.\n\n\
         ## 요구사항\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n\
         ## 열린 가정\n\n없음.\n\n# 2부 설계\n\n"
    );
    for (key, heading) in PART_TWO {
        text.push_str(&format!("## {heading}\n\n{}\n", part_two(key)));
    }
    text
}

fn filled(key: &str) -> String {
    format!("{key} 절을 채웠어요.\n")
}

fn guidance(key: &str) -> String {
    format!("<!-- 이 절에 설계를 적어요. (키: {key}) -->\n")
}

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

fn approve(t: &Scratch) -> Output {
    t.run(&["request", "approve", "--run", RUN])
}

/// A refused approval: exit 1 and no stamp.
fn refused(t: &Scratch, out: &Output) {
    assert_eq!(out.status.code(), Some(1), "{}{}", stdout(out), stderr(out));
    assert!(!t.0.join(STAMP).exists(), "a refused approval stamps nothing");
}

#[test]
fn R13_design_gate_required_names_every_unfilled_section() {
    // Blank, guidance only, and guidance kept beside prose all count as unfilled.
    let t = scratch(&request("required", |key| match key {
        "current" => filled(key),
        "proposed" => String::new(),
        "alternatives" => guidance(key),
        "mapping" => format!("{}{}", filled(key), guidance(key)),
        _ => guidance(key),
    }));
    let out = approve(&t);
    refused(&t, &out);
    for heading in ["바꿀 구조", "검토한 대안과 버린 이유", "R 행과 모듈의 대응", "위험"] {
        assert!(
            stdout(&out).contains(&format!("section ## {heading}:")),
            "{heading}: {}",
            stdout(&out)
        );
    }
    assert!(!stdout(&out).contains("section ## 지금 구조:"), "{}", stdout(&out));
    // A recorded skip is refused up front, so required can only be met by writing part 2.
    let skip = t.run(&["request", "design-skip", "--why", "작은 변경이에요.", "--run", RUN]);
    assert_eq!(skip.status.code(), Some(1), "{}", stdout(&skip));
    assert!(stderr(&skip).contains("design_review: required"), "{}", stderr(&skip));

    for (key, _) in PART_TWO {
        t.write("section.md", &filled(key));
        t.ok(&["request", "section", key, "--from", "section.md", "--run", RUN]);
    }
    let out = approve(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    assert!(t.0.join(STAMP).exists());
}

#[test]
fn R13_design_gate_auto_needs_part_two_or_a_skip_reason() {
    let t = scratch(&request("auto", guidance));
    let out = approve(&t);
    refused(&t, &out);
    assert!(stdout(&out).contains("section ## 지금 구조:"), "{}", stdout(&out));
    assert!(
        stderr(&out).contains("dstack request design-skip --why"),
        "the refusal names the skip command: {}",
        stderr(&out)
    );

    let why = "명령 하나를 고치는 작은 변경이에요.";
    t.ok(&["request", "design-skip", "--why", why, "--run", RUN]);
    let decisions = t.read(".dstack/runs/20261001T000000Z_design/decisions.md");
    assert!(
        decisions.contains(&format!("| design skipped: {why} | R01 |")),
        "{decisions}"
    );
    let out = approve(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains(why), "the reason is printed: {}", stdout(&out));
}

#[test]
fn R13_design_gate_auto_passes_a_filled_part_two() {
    let t = scratch(&request("auto", filled));
    let out = approve(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    assert!(t.0.join(STAMP).exists());
}

#[test]
fn R13_design_gate_skip_prints_the_frontmatter_skip() {
    let t = scratch(&request("skip", guidance));
    let out = approve(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("skipped by the frontmatter (design_review: skip)"),
        "{}",
        stdout(&out)
    );
}

#[test]
fn R13_design_skip_refuses_what_it_cannot_record() {
    let t = scratch(&request("auto", guidance));
    for why in ["", "   "] {
        let out = t.run(&["request", "design-skip", "--why", why, "--run", RUN]);
        assert_eq!(out.status.code(), Some(1), "empty reason {why:?}");
    }
    let out = t.run(&["request", "design-skip", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "no --why");
    assert!(!t.0.join(".dstack/runs/20261001T000000Z_design/decisions.md").exists());

    t.ok(&["request", "design-skip", "--why", "작은 변경이에요.", "--run", RUN]);
    t.ok(&["request", "approve", "--run", RUN]);
    let out = t.run(&["request", "design-skip", "--why", "다시 남겨요.", "--run", RUN]);
    assert_eq!(out.status.code(), Some(1), "approved");
    assert!(stderr(&out).contains("approved"), "{}", stderr(&out));

    t.ok(&["quick", "new", "qq"]);
    let out = t.run(&["request", "design-skip", "--why", "작은 변경이에요.", "--quick", "qq"]);
    assert_eq!(out.status.code(), Some(1), "quick");
    assert!(stderr(&out).contains("quick"), "{}", stderr(&out));
}

#[test]
fn R13_design_gate_leaves_quick_and_legacy_requests_alone() {
    // A quick request has no part 2, whatever its frontmatter says.
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "qq"]);
    let quick = ".dstack/quick/qq/request.md";
    let text = t.read(quick).replace("design_review: skip", "design_review: required");
    t.write(quick, &text);
    t.write("section.md", "요약 문단을 채웠어요.\n");
    t.ok(&["request", "section", "summary", "--from", "section.md", "--quick", "qq"]);
    t.ok(&["req", "add", "첫 요구사항이에요.", "--accept", "첫 확인이에요.", "--quick", "qq"]);
    t.ok(&["request", "approve", "--quick", "qq"]);

    // A legacy request approved before the PRD layout re-approves after a merged row.
    let legacy = request("required", filled);
    let legacy = format!(
        "{}# 옛 요청서예요\n\n한 문단이에요.\n\n## 요구사항\n\n\
         - [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n",
        &legacy[..legacy.find("# 설계 확인 시험").expect("the title")]
    );
    let t = scratch(&legacy);
    let run = t.0.join(".dstack/runs").join(RUN);
    let hash = dstack_cli::core::fsx::sha256_file(&run.join("request.md")).expect("the request");
    std::fs::write(
        run.join("request.approved"),
        format!("sha256 {hash}  approved_at 2026-09-13T20:31:44Z\n"),
    )
    .expect("the stamp");
    t.ok(&["req", "add", "덧붙인 요구사항이에요.", "--accept", "덧붙인 확인이에요.", "--run", RUN]);
    let out = approve(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
}

#[test]
fn R13_request_section_writes_only_the_named_part_two_section() {
    let t = scratch(&request("auto", guidance));
    let before = t.read(REQUEST);
    t.write("section.md", "바꾼 구조를 적었어요.\n");
    t.ok(&["request", "section", "proposed", "--from", "section.md", "--run", RUN]);
    let expected = before.replacen(&guidance("proposed"), "바꾼 구조를 적었어요.\n", 1);
    assert_eq!(t.read(REQUEST), expected);
}

#[test]
fn R13_design_gate_refuses_a_skip_decision_without_a_reason() {
    // `decision add` can write the skip prefix alone; that row is no reason, so auto still asks
    // for part 2 and names every unfilled section.
    let t = scratch(&request("auto", guidance));
    for text in ["design skipped:", "design skipped:   "] {
        t.ok(&["decision", "add", text, "--affects", "R01", "--run", RUN]);
        let out = approve(&t);
        refused(&t, &out);
        for (_, heading) in PART_TWO {
            assert!(
                stdout(&out).contains(&format!("section ## {heading}:")),
                "{text:?} {heading}: {}",
                stdout(&out)
            );
        }
    }
}

#[test]
fn R13_design_skip_refuses_a_reason_with_a_line_break() {
    // A line break would split the decision row, and approval could not read the reason back.
    let t = scratch(&request("auto", guidance));
    let why = "명령 하나를 고치는 작은 변경이에요.";
    t.ok(&["request", "design-skip", "--why", why, "--run", RUN]);
    let decisions = ".dstack/runs/20261001T000000Z_design/decisions.md";
    let before = t.read(decisions);
    for broken in ["첫 줄\n둘째 줄", "첫 줄\r둘째 줄", "첫 줄\r\n둘째 줄"] {
        let out = t.run(&["request", "design-skip", "--why", broken, "--run", RUN]);
        assert_eq!(out.status.code(), Some(1), "{broken:?}: {}", stdout(&out));
        assert!(stderr(&out).contains("line break"), "{}", stderr(&out));
        assert_eq!(t.read(decisions), before, "{broken:?} wrote nothing");
    }
    let out = approve(&t);
    assert_eq!(out.status.code(), Some(0), "{}{}", stdout(&out), stderr(&out));
    assert!(stdout(&out).contains(why), "the reason is printed: {}", stdout(&out));
}
