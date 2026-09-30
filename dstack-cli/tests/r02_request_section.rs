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

#[test]
fn R02_request_section_inline_triple_backticks_are_not_a_fence() {
    // An inline code span at the start of a line opens no fence, so the headings after it
    // still end `## 지금 구조`: the R rows above, `## 바꿀 구조`, `## 위험` and the frontmatter
    // stay byte for byte.
    let mut parts = goal();
    parts[10].2 = "\n```old```는 옛 이름이에요.\n\n".to_string();
    let t = scratch(&render(&parts));
    let out = section(&t, "current", "지금 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = "\n지금 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(
        t.read(REQUEST),
        render(&parts),
        "only ## 지금 구조 may change"
    );

    let content = "```new```는 새 이름이에요.";
    let out = section(&t, "current", content);
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = format!("\n{content}\n\n");
    assert_eq!(t.read(REQUEST), render(&parts));
}

#[test]
fn R02_request_section_refuses_unclosed_fence_in_document() {
    // A fence the document never closes hides every heading after it; no body may run past it
    // to the end of the file, so every write names the opening line instead.
    let mut parts = goal();
    parts[10].2 = "\n```\n# 닫히지 않은 코드\n\n".to_string();
    let opener = render(&parts[..10]).lines().count() + 3;
    let t = scratch(&render(&parts));
    for key in ["current", "goals", "risks"] {
        refused(
            &t,
            key,
            "새 내용이에요.",
            &format!("line {opener} opens a code fence that never closes"),
        );
    }

    // A tilde fence may hold backticks in its info string and still hides the `##` line inside.
    let mut tilde = goal();
    tilde[10].2 = "\n~~~ ```x```\n## 코드 안의 절\n~~~\n\n".to_string();
    let text = render(&tilde);
    assert_eq!(
        body(&text, &Place::Section("지금 구조")).unwrap(),
        tilde[10].2
    );
}

#[test]
fn R02_request_section_html_comments_hide_fences_and_headings() {
    // The review fragment: a fence marker inside a comment opens nothing, so `## 바꿀 구조` still
    // ends `## 지금 구조`, and the rows above, every later section and the frontmatter stay.
    let mut parts = goal();
    parts[10].2 = "\n현재 설명이에요.\n<!--\n```\n-->\n\n".to_string();
    parts[11].2 = "\n보존해야 하는 설계예요.\n<!--\n```\n-->\n\n".to_string();
    parts[14].2 =
        "\n<!-- a --> 위험 <!-- b -->\n<!--\n## 주석 속 절\n-->\n위험 설명이에요.\n\n".to_string();
    let t = scratch(&render(&parts));
    let out = section(&t, "current", "지금 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = "\n지금 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(
        t.read(REQUEST),
        render(&parts),
        "only ## 지금 구조 may change"
    );
}

#[test]
fn R02_request_section_refuses_unclosed_comment_in_document() {
    // No later `-->` closes it, so it hides every heading after it; every write names its line.
    let mut parts = goal();
    parts[10].2 = "\n<!-- 닫히지 않은 주석\n\n".to_string();
    for later in [11, 14, 15] {
        parts[later].2 = "\n안내예요.\n\n".to_string();
    }
    let opener = render(&parts[..10]).lines().count() + 3;
    let t = scratch(&render(&parts));
    for key in ["current", "goals", "risks"] {
        refused(
            &t,
            key,
            "새 내용이에요.",
            &format!("line {opener} opens an HTML comment that never closes"),
        );
    }
}

#[test]
fn R02_request_section_refuses_ambiguous_indent_and_inline_comment() {
    // A fence marker indented as a list item's would be may close a list's fence or open one a
    // renderer ends early; either way no boundary after it is certain.
    let mut parts = goal();
    parts[10].2 = "\n- 항목이에요.\n  ```\n  코드예요.\n```\n\n".to_string();
    let marker = render(&parts[..10]).lines().count() + 4;
    let t = scratch(&render(&parts));
    for key in ["current", "risks"] {
        refused(&t, key, "새 내용이에요.", &format!("line {marker} "));
        refused(&t, key, "새 내용이에요.", "ambiguous");
    }

    // A comment opened after text cannot hide a heading from a renderer.
    let mut parts = goal();
    parts[10].2 = "\n설명이에요 <!-- 여기서 열려요\n\n".to_string();
    parts[11].2 = "\n-->\n\n".to_string();
    let heading = render(&parts[..11]).lines().count() + 1;
    let t = scratch(&render(&parts));
    refused(&t, "current", "새 내용이에요.", &format!("line {heading} "));
    refused(&t, "goals", "새 내용이에요.", "ambiguous");
}

#[test]
fn R02_request_section_indented_heading_in_list_is_body() {
    // Only a column-0 `##` line is a section: the list item's heading and the quoted lines are
    // body text of `## 지금 구조`.
    let mut parts = goal();
    parts[10].2 = "\n- 목록 항목이에요.\n  ## 바꿀 구조\n> ## 바꿀 구조\n> ```\n\n".to_string();
    let t = scratch(&render(&parts));
    let out = section(&t, "proposed", "바꿀 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[11].2 = "\n바꿀 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(t.read(REQUEST), render(&parts));

    let out = section(&t, "current", "지금 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = "\n지금 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(t.read(REQUEST), render(&parts));
}

#[test]
fn R02_request_section_content_follows_comment_and_indent_rules() {
    let mut parts = goal();
    let t = scratch(&render(&parts));
    refused(
        &t,
        "current",
        "앞이에요.\n<!-- 닫히지 않아요",
        "line 2 of the content opens an HTML comment that never closes",
    );
    refused(
        &t,
        "current",
        "- 항목이에요.\n  ```\n  코드예요.",
        "ambiguous",
    );
    refused(&t, "current", "설명이에요 <!--\n## 새 절\n-->", "ambiguous");
    refused(&t, "current", "<!-->\n## 새 절\n-->", "is a heading");

    let content = "<!--\n## 안내\n```\n-->\n- 항목이에요.\n  ## 목록 속 제목\n> ```";
    let out = section(&t, "current", content);
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = format!("\n{content}\n\n");
    assert_eq!(t.read(REQUEST), render(&parts));
}

#[test]
fn R02_request_section_refuses_raw_html_blocks() {
    // The review fragment: backticks inside `<pre>` are literal to a renderer, so pairing them as
    // a fence would hide `## 바꿀 구조` and a write to `current` would delete it. Every raw HTML
    // block start refuses instead, naming its line, and the rows above and the store stay.
    for open in ["<pre>", "<details>", "</div>", "<!DOCTYPE html>", "  <pre>"] {
        let mut parts = goal();
        parts[10].2 = format!("\n{open}\n```\n</pre>\n\n");
        parts[11].2 = format!("\n보존해야 하는 설계예요.\n{open}\n```\n</pre>\n\n");
        parts[14].2 = "\n위험 설명이에요.\n\n".to_string();
        let line = render(&parts[..10]).lines().count() + 3;
        let t = scratch(&render(&parts));
        let reason = format!("line {line} starts raw HTML, which request section does not support");
        for key in ["current", "risks"] {
            refused(&t, key, "새 내용이에요.", &reason);
        }
    }
}

#[test]
fn R02_request_section_keeps_html_in_fences_and_comments() {
    // Fence content is literal and a column-0 comment is opaque, so HTML there is text: writing
    // another section keeps both byte for byte, and new content may hold them too.
    let mut parts = goal();
    parts[10].2 = "\n```html\n<pre>\n## 코드 안의 절\n</pre>\n```\n\n".to_string();
    parts[14].2 = "\n<!--\n<pre>\n```\n</pre>\n-->\n위험 설명이에요.\n\n".to_string();
    let t = scratch(&render(&parts));
    let out = section(&t, "proposed", "바꿀 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[11].2 = "\n바꿀 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(
        t.read(REQUEST),
        render(&parts),
        "only ## 바꿀 구조 may change"
    );

    let content = "~~~\n<details>\n```\n</details>\n~~~\n<!-- <pre> -->\n<!--\n</div>\n-->";
    let out = section(&t, "current", content);
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = format!("\n{content}\n\n");
    assert_eq!(t.read(REQUEST), render(&parts));
}

#[test]
fn R02_request_section_refuses_raw_html_in_content() {
    let t = scratch(&render(&goal()));
    for (content, line) in [
        ("<div>\n설명이에요.\n</div>", 1),
        ("설명이에요.\n  <pre>\n```\n</pre>", 2),
    ] {
        let reason = format!("line {line} of the content starts raw HTML, which request section");
        refused(&t, "current", content, &reason);
    }
    // A comment opened after text hides nothing from a renderer, so HTML in it is not safe.
    refused(&t, "current", "설명이에요 <!--\n<pre>\n-->", "ambiguous");
}

#[test]
fn R02_request_section_refuses_tab_before_heading_or_fence() {
    // A tab indents a line past what a top-level heading or fence allows but not past a list
    // item's, so no reading of such a line is certain.
    let mut parts = goal();
    parts[10].2 = "\n- 항목이에요.\n\t## 탭 뒤의 제목\n\n".to_string();
    let line = render(&parts[..10]).lines().count() + 4;
    let t = scratch(&render(&parts));
    refused(
        &t,
        "risks",
        "새 내용이에요.",
        &format!("line {line} indents"),
    );
    refused(&t, "risks", "새 내용이에요.", "with a tab");

    let t = scratch(&render(&goal()));
    for (content, line) in [
        ("\t## 새 절", 1),
        ("설명이에요.\n \t```\n코드예요.\n \t```", 2),
    ] {
        let reason =
            format!("line {line} of the content indents a heading, fence or `<` with a tab");
        refused(&t, "current", content, &reason);
    }
}

#[test]
fn R02_request_section_refuses_setext_heading_in_document() {
    // The review fragment: a renderer shows `바꿀 구조` over its underline as an h2 section, so
    // reading both lines as text would let a write to `current` delete it. Every underline form
    // refuses instead, naming its line, and the rows above and the store stay.
    for underline in ["---", "===", " ---", "  ===", "   --- \t"] {
        let mut parts = goal();
        parts.retain(|part| part.0 != "proposed");
        parts[10].2 =
            format!("\n현재 설명이에요.\n\n바꿀 구조\n{underline}\n\n보존해야 하는 설계예요.\n\n");
        let risks = parts.iter_mut().find(|part| part.0 == "risks").unwrap();
        risks.2 = "\n위험 설명이에요.\n\n".to_string();
        let line = render(&parts[..10]).lines().count() + 6;
        let t = scratch(&render(&parts));
        let reason = format!(
            "line {line} underlines a setext heading, which request section does not support"
        );
        for key in ["current", "risks"] {
            refused(&t, key, "새 내용이에요.", &reason);
        }
    }
}

#[test]
fn R02_request_section_refuses_setext_heading_in_content() {
    // A comment opened after text is inline, so its closing line is still paragraph text.
    let t = scratch(&render(&goal()));
    for (content, line) in [
        ("바꿀 구조\n---\n\n설계예요.", 2),
        ("앞이에요.\n\n설명이에요.\n   ===", 4),
        ("설명이에요 <!-- 주석\n닫혀요 -->\n-", 3),
    ] {
        let reason = format!("line {line} of the content underlines a setext heading");
        refused(&t, "current", content, &reason);
    }
}

#[test]
fn R02_request_section_keeps_thematic_breaks() {
    // After a blank line, an ATX heading or a column-0 comment a `---` or `===` line underlines
    // nothing, so writing another section keeps it byte for byte and new content may hold it.
    // The frontmatter's closing `---` sits under `korean_polish: on` and is never an underline.
    assert!(FRONT.ends_with("korean_polish: on\n---\n"));
    let mut parts = goal();
    parts[6].2 = "\n### S1 시나리오\n---\n흐름이에요.\n\n".to_string();
    parts[10].2 = "\n현재 설명이에요.\n\n---\n\n===\n\n<!--\n안내\n-->\n---\n\n".to_string();
    let t = scratch(&render(&parts));
    let out = section(&t, "proposed", "바꿀 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[11].2 = "\n바꿀 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(
        t.read(REQUEST),
        render(&parts),
        "only ## 바꿀 구조 may change"
    );

    let content = "---\n\n앞이에요.\n\n===\n\n### 소절\n===\n뒤예요.";
    let out = section(&t, "current", content);
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = format!("\n{content}\n\n");
    assert_eq!(t.read(REQUEST), render(&parts));
}

#[test]
fn R02_request_section_nbsp_does_not_close_a_fence() {
    // The review fragment: only spaces or tabs may follow a closing fence, so a fence then U+00A0
    // is code and the next bare fence closes the block. `## 바꿀 구조` still ends `## 지금 구조`,
    // and the rows above, every later section and the frontmatter stay byte for byte.
    for mark in ["~~~", "```"] {
        let mut parts = goal();
        parts[10].2 = format!("\n{mark}\n{mark}\u{a0}\n{mark}\n\n");
        parts[11].2 = format!("\n보존해야 하는 설계예요.\n{mark}\n{mark}\u{a0}\n{mark}\n\n");
        parts[14].2 = "\n위험 설명이에요.\n\n".to_string();
        let t = scratch(&render(&parts));
        let out = section(&t, "current", "지금 구조를 새로 적었어요.");
        assert!(out.status.success(), "{mark}: {}", stderr(&out));
        parts[10].2 = "\n지금 구조를 새로 적었어요.\n\n".to_string();
        assert_eq!(
            t.read(REQUEST),
            render(&parts),
            "{mark}: only ## 지금 구조 may change"
        );
    }
}

#[test]
fn R02_request_section_nbsp_after_hashes_is_not_a_heading() {
    // `##` opens a heading only before a space, a tab or the line end, and a heading's text keeps
    // a trailing U+00A0: the first line is body text of `## 지금 구조`, the second names no `위험`.
    let mut parts = goal();
    parts[10].2 = "\n##\u{a0}바꿀 구조\n\n".to_string();
    parts[14].1 = "## 위험\u{a0}";
    let t = scratch(&render(&parts));
    let out = section(&t, "proposed", "바꿀 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[11].2 = "\n바꿀 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(t.read(REQUEST), render(&parts));
    refused(&t, "risks", "위험이에요.", "no '## 위험' heading");

    let content = "##\u{a0}새 절\n#\u{a0}새 제목";
    let out = section(&t, "current", content);
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = format!("\n{content}\n\n");
    assert_eq!(
        t.read(REQUEST),
        render(&parts),
        "only ## 지금 구조 may change"
    );
}

#[test]
fn R02_request_section_nbsp_line_is_not_blank() {
    // A line of only U+00A0 is paragraph text: a `---` under it underlines a setext heading, a
    // body holding it is filled, and the writer keeps it where new content puts it.
    let mut parts = goal();
    parts[10].2 = "\n현재 설명이에요.\n\n\u{a0}\n---\n\n".to_string();
    let line = render(&parts[..10]).lines().count() + 6;
    let t = scratch(&render(&parts));
    let reason = format!("line {line} underlines a setext heading");
    refused(&t, "risks", "새 내용이에요.", &reason);
    let content = "앞이에요.\n\n\u{a0}\n---";
    let reason = "line 4 of the content underlines a setext heading";
    refused(&scratch(&render(&goal())), "current", content, reason);
    assert!(!is_blank("\n\u{a0}\n"));
    assert!(!is_blank("<!-- 안내 -->\u{a0}\n"));

    let mut parts = goal();
    let t = scratch(&render(&parts));
    let out = section(&t, "current", "\u{a0}\n\n설명이에요.\u{a0}\n\n");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = "\n\u{a0}\n\n설명이에요.\u{a0}\n\n".to_string();
    assert_eq!(t.read(REQUEST), render(&parts));
}

#[test]
fn R02_request_section_spaces_and_tabs_stay_structural() {
    // The ASCII forms read as before: a closing fence may end in spaces or tabs, `##` then a tab
    // opens a heading whose text drops them, and a line of spaces and tabs is blank.
    let mut parts = goal();
    parts[10].2 = "\n~~~\n## 코드 안의 절\n~~~ \t\n\n```\n```\t \n \t\n---\n\n".to_string();
    parts[14].1 = "##\t위험 \t";
    let t = scratch(&render(&parts));
    let out = section(&t, "proposed", "바꿀 구조를 새로 적었어요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[11].2 = "\n바꿀 구조를 새로 적었어요.\n\n".to_string();
    assert_eq!(t.read(REQUEST), render(&parts));
    let out = section(&t, "risks", "위험이에요.");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[14].2 = "\n위험이에요.\n\n".to_string();
    assert_eq!(t.read(REQUEST), render(&parts));

    refused(
        &t,
        "current",
        "앞이에요.\n##\t새 절",
        "line 2 of the content is a heading",
    );
    let out = section(&t, "current", " \t\n\n설명이에요. \t\n \t\n");
    assert!(out.status.success(), "{}", stderr(&out));
    parts[10].2 = "\n설명이에요.\n\n".to_string();
    assert_eq!(t.read(REQUEST), render(&parts));
}

#[test]
fn R02_request_section_refuses_lone_carriage_return_in_document() {
    // The T23 fragment: a renderer ends a line at a carriage return no line feed follows, so the
    // `## 바꿀 구조` after it is a section a write to `current` would delete, the rows above in
    // place. Every write names the line instead, and the store stays byte for byte.
    let mut parts = goal();
    parts.retain(|part| part.0 != "proposed");
    parts[10].2 = "\n현재 설명이에요.\r## 바꿀 구조\n\n보존해야 하는 설계예요.\n\n".to_string();
    let line = render(&parts[..10]).lines().count() + 3;
    let t = scratch(&render(&parts));
    let reason = format!("line {line} holds a carriage return without a line feed");
    for key in ["current", "risks"] {
        refused(&t, key, "새 내용이에요.", &reason);
    }
}

#[test]
fn R02_request_section_refuses_lone_carriage_return_anywhere() {
    // A lone carriage return may move any boundary, so it refuses inside a fence, in a comment,
    // in the frontmatter, before a carriage return and line feed, and at the end of the file.
    let lead = render(&goal()[..10]).lines().count();
    let mut cases = Vec::new();
    for (body, line) in [
        ("\n```\n코드예요.\r```\n\n", lead + 4),
        ("\n<!--\n안내예요.\r-->\n\n", lead + 4),
        ("\n현재 설명이에요.\r\r\n\n", lead + 3),
    ] {
        let mut parts = goal();
        parts[10].2 = body.to_string();
        cases.push((render(&parts), line));
    }
    let text = render(&goal());
    cases.push((text.replacen("on\n---\n", "on\r---\n", 1), 12));
    let end = text.trim_end_matches('\n');
    cases.push((format!("{end}\r"), end.matches('\n').count() + 1));
    for (text, line) in cases {
        let t = scratch(&text);
        let reason = format!("line {line} holds a carriage return without a line feed");
        refused(&t, "current", "새 내용이에요.", &reason);
    }
}

#[test]
fn R02_request_section_refuses_lone_carriage_return_in_content() {
    let t = scratch(&render(&goal()));
    for (content, line) in [
        ("설명이에요.\r## 새 절", 1),
        ("앞이에요.\n설명이에요.\r\r\n뒤예요.", 2),
        ("설명이에요.\r", 1),
    ] {
        let reason =
            format!("line {line} of the content holds a carriage return without a line feed");
        refused(&t, "current", content, &reason);
    }
}

#[test]
fn R02_request_section_keeps_crlf_line_endings() {
    // Every line under the frontmatter ends in CRLF, rows, fences, comments and headings too: a
    // write changes only the named body, CRLF content included, and keeps every other byte.
    let parts = goal();
    let lines = render(&parts)[FRONT.len()..].replace('\n', "\r\n");
    let crlf = format!("{FRONT}{lines}");
    let t = scratch(&crlf);
    let content = "지금 구조를 새로 적었어요.\r\n\r\n둘째 줄이에요.\r\n";
    let out = section(&t, "current", content);
    assert!(out.status.success(), "{}", stderr(&out));
    let old = format!("## 지금 구조\r\n{}", parts[10].2.replace('\n', "\r\n"));
    let new = "## 지금 구조\r\n\n지금 구조를 새로 적었어요.\r\n\r\n둘째 줄이에요.\n\n";
    assert!(crlf.contains(&old));
    assert_eq!(t.read(REQUEST), crlf.replacen(&old, new, 1));
}
