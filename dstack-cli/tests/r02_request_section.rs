// tests/r02_request_section.rs
// R02: `request section` writes one prose section of an unapproved request and leaves the
// frontmatter, every R row and every other section byte for byte where they were.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::Output;

use dstack_cli::store::request_sections::{body, is_blank, Place};
use support::{tree, Scratch};

const RUN: &str = "20260930T000000Z_sections";
const REQUEST: &str = ".dstack/runs/20260930T000000Z_sections/request.md";

const FRONT: &str = "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\n\
risk_axes: none\ndesign_review: auto\nreview: on\ncodex_effort: high\ne2e: cli\n\
unit_tests: on\nvisual: none\nkorean_polish: on\n---\n";

/// The Goal request as (key, heading line, body) in file order; a key of "" is a heading the
/// writer never targets. Bodies are deliberately uneven so a rewrite of them would show.
fn goal() -> Vec<(&'static str, &'static str, String)> {
    [
        ("summary", "# 절 쓰기 시험", "\n요약 안내문이에요.\n\n"),
        (
            "",
            "## 한눈에 보기",
            "\n<!-- request brief가 채워요. -->\n\n",
        ),
        ("", "# 1부 요청", "\n"),
        ("background", "## 배경과 문제", "\n<!-- 배경 안내 -->\n\n"),
        ("goals", "## 목표", "\n1. 옛 목표예요.\n   \n\n"),
        ("non-goals", "## 비목표", "\n"),
        (
            "scenarios",
            "## 사용 시나리오",
            "\n### S1 옛 시나리오\n\n옛 흐름이에요.\n\n",
        ),
        (
            "",
            "## 요구사항",
            "\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\
             - [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요. — from: Q-01\n\n",
        ),
        ("assumptions", "## 열린 가정", "\n없음.\n\n"),
        ("", "# 2부 설계", "\n"),
        (
            "current",
            "## 지금 구조",
            "\n```\n# 코드 안의 제목\n## 코드 안의 절\n```\n\n",
        ),
        ("proposed", "## 바꿀 구조", "\n<!-- 안내 -->\n\n"),
        (
            "alternatives",
            "## 검토한 대안과 버린 이유",
            "\n| 대안 | 이유 |\n|---|---|\n\n",
        ),
        ("mapping", "## R 행과 모듈의 대응", "\n\n"),
        ("risks", "## 위험", "\n<!-- 안내 -->\n\n"),
        ("", "# 3부 계획과 검증", "\n<!-- 계획 -->\n"),
    ]
    .into_iter()
    .map(|(key, heading, body)| (key, heading, body.to_string()))
    .collect()
}

fn render(parts: &[(&str, &str, String)]) -> String {
    let mut text = FRONT.to_string();
    for (_, heading, body) in parts {
        text.push_str(heading);
        text.push('\n');
        text.push_str(body);
    }
    text
}

/// A store with one run whose request.md is `text`.
fn scratch(text: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(REQUEST, text);
    t
}

/// `request section <key> --from section.md --run <RUN>` over whatever section.md holds.
fn write_from_file(t: &Scratch, key: &str) -> Output {
    t.run(&[
        "request",
        "section",
        key,
        "--from",
        "section.md",
        "--run",
        RUN,
    ])
}

fn section(t: &Scratch, key: &str, content: &str) -> Output {
    t.write("section.md", content);
    write_from_file(t, key)
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A refusal: exit 1, the named reason on stderr, and not one byte of the store changed.
fn refused(t: &Scratch, key: &str, content: &str, reason: &str) {
    t.write("section.md", content);
    let before = tree(&t.0);
    let out = write_from_file(t, key);
    assert_eq!(out.status.code(), Some(1), "{key}: {}", stderr(&out));
    assert!(
        stderr(&out).contains(reason),
        "{key}: want '{reason}' in {}",
        stderr(&out)
    );
    assert_eq!(tree(&t.0), before, "{key}: a refusal wrote something");
}

#[test]
fn R02_request_section_writes_only_named_section() {
    let mut parts = goal();
    let t = scratch(&render(&parts));
    t.write(
        "section.md",
        "\n\n1. 새 목표예요.\n2. 둘째 목표예요.  \n\n\n",
    );
    let mut expected = tree(&t.0);
    let out = write_from_file(&t, "goals");
    assert!(out.status.success(), "{}", stderr(&out));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("## 목표"),
        "the output names the section: {stdout}"
    );

    parts[4].2 = "\n1. 새 목표예요.\n2. 둘째 목표예요.\n\n".to_string();
    let written = render(&parts);
    assert!(written.starts_with(FRONT));
    assert_eq!(t.read(REQUEST), written);
    let file = PathBuf::from(REQUEST);
    expected.iter_mut().find(|(p, _)| *p == file).unwrap().1 = written.into_bytes();
    assert_eq!(tree(&t.0), expected, "only request.md may change");
}

#[test]
fn R02_request_section_maps_every_key_to_its_heading() {
    let mut parts = goal();
    let t = scratch(&render(&parts));
    for index in 0..parts.len() {
        let key = parts[index].0;
        if key.is_empty() {
            continue;
        }
        let content = match key {
            "scenarios" => {
                "### S1 새 시나리오\n\n흐름이에요.\n\n### S2 둘째 시나리오\n\n흐름이에요."
                    .to_string()
            }
            "proposed" => "```\n# 제목\n## 한눈에 보기\n```".to_string(),
            _ => format!("{key} 새 내용이에요."),
        };
        let out = section(&t, key, &content);
        assert!(out.status.success(), "{key}: {}", stderr(&out));
        let heading = parts[index].1;
        let named = if key == "summary" {
            "# 절 쓰기 시험"
        } else {
            heading
        };
        assert!(
            String::from_utf8_lossy(&out.stdout).contains(named),
            "{key}"
        );
        parts[index].2 = format!("\n{content}\n\n");
        assert_eq!(
            t.read(REQUEST),
            render(&parts),
            "{key} changed more than {heading}"
        );
    }
}

#[test]
fn R02_request_section_writes_quick_summary() {
    let t = Scratch::new();
    t.init();
    t.ok(&["quick", "new", "q1"]);
    let request = ".dstack/quick/q1/request.md";
    let rows = "- [ ] **R01** 빠른 요구사항이에요. — accept: 확인해요.\n";
    t.write(
        request,
        &format!("{FRONT}# 빠른 작업\n\n<!-- 요약 안내 -->\n\n## 요구사항\n\n{rows}"),
    );
    t.write("summary.md", "배경과 목표를 한 문단으로 적었어요.\n");
    let out = t.run(&[
        "request",
        "section",
        "summary",
        "--from",
        "summary.md",
        "--quick",
        "q1",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert_eq!(
        t.read(request),
        format!(
            "{FRONT}# 빠른 작업\n\n배경과 목표를 한 문단으로 적었어요.\n\n## 요구사항\n\n{rows}"
        )
    );
    let before = tree(&t.0);
    let out = t.run(&[
        "request",
        "section",
        "goals",
        "--from",
        "summary.md",
        "--quick",
        "q1",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        stderr(&out).contains("no '## 목표' heading"),
        "{}",
        stderr(&out)
    );
    assert_eq!(tree(&t.0), before);
}

#[test]
fn R02_request_section_refuses_approved_request() {
    let t = scratch(&render(&goal()));
    t.write(
        ".dstack/runs/20260930T000000Z_sections/request.approved",
        "sha256 abc  approved_at 2026-09-30T00:00:00Z\n",
    );
    refused(&t, "goals", "새 목표예요.", "is approved");
}

#[test]
fn R02_request_section_refuses_unknown_key_and_requirements() {
    let t = scratch(&render(&goal()));
    refused(&t, "bogus", "내용이에요.", "unknown section key: bogus");
    for key in ["requirements", "요구사항"] {
        refused(&t, key, "내용이에요.", "dstack req add");
    }
}

#[test]
fn R02_request_section_refuses_rows_and_headings_in_content() {
    let t = scratch(&render(&goal()));
    let row = "- [ ] **R09** 몰래 넣은 행이에요. — accept: 확인해요.";
    refused(
        &t,
        "goals",
        &format!("목표예요.\n{row}"),
        "looks like an R row",
    );
    refused(
        &t,
        "background",
        "앞에서 ] **R01** 을 가리켜요.",
        "looks like an R row",
    );
    refused(&t, "goals", "목표예요.\n## 새 절", "is a heading");
    refused(&t, "risks", "# 새 제목\n위험이에요.", "is a heading");
    refused(&t, "summary", "요약이에요.\n### 소제목", "is a heading");
    refused(&t, "current", "```\n# 닫히지 않은 코드", "never closes");
}

#[test]
fn R02_request_section_refuses_missing_or_repeated_heading() {
    let without_risks: Vec<_> = goal().into_iter().filter(|p| p.0 != "risks").collect();
    let t = scratch(&render(&without_risks));
    refused(&t, "risks", "위험이에요.", "no '## 위험' heading");

    let mut repeated = goal();
    repeated.push(("", "## 목표", "\n또 있어요.\n".to_string()));
    let t = scratch(&render(&repeated));
    refused(&t, "goals", "목표예요.", "appears 2 times");

    let mut rows_inside = goal();
    rows_inside[8].2 = "\n- [ ] **R03** 여기 있는 행이에요. — accept: 확인해요.\n\n".to_string();
    let t = scratch(&render(&rows_inside));
    refused(&t, "assumptions", "없음.", "holds R row R03");
}

#[test]
fn R02_request_section_reads_bodies_and_blank_sections() {
    let text = render(&goal());
    assert_eq!(
        body(&text, &Place::Summary).unwrap(),
        "\n요약 안내문이에요.\n\n"
    );
    let scenarios = body(&text, &Place::Section("사용 시나리오")).unwrap();
    assert_eq!(scenarios, "\n### S1 옛 시나리오\n\n옛 흐름이에요.\n\n");
    let current = body(&text, &Place::Section("지금 구조")).unwrap();
    assert!(
        current.contains("# 코드 안의 제목"),
        "a fenced # line is not a heading"
    );
    for blank in [
        "",
        "\n  \n",
        "\n<!-- 안내 -->\n\n",
        "<!-- a -->\n<!--\n여러 줄\n-->\n",
    ] {
        assert!(is_blank(blank), "{blank:?}");
    }
    for filled in [
        "\n없음.\n",
        "<!-- a --> 글",
        "| 표 |",
        "<!-- 닫힘 -->\n\n### S1",
    ] {
        assert!(!is_blank(filled), "{filled:?}");
    }
    assert!(body(&text, &Place::Section("없는 절")).is_err());
}
