// tests/r01_prd_templates.rs
// R01: every work_type template gives `request new` the PRD layout with its guidance in comments,
// the row, approval and hash verbs answer as before, and a quick task keeps only its summary and
// `## 요구사항`.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use dstack_cli::store::request_sections::{body, is_blank, Place, REQUIREMENTS, SECTION_KEYS};
use support::Scratch;

const TYPES: [&str; 5] = ["web-ui", "http-api", "cli", "library", "docs-writing"];

const TITLE: &str = "PRD 틀 시험";

/// The headings of a fresh Goal request after its title, in file order.
const LAYOUT: [&str; 14] = [
    "## 한눈에 보기",
    "# 1부 요청",
    "## 배경과 문제",
    "## 목표",
    "## 비목표",
    "## 사용 시나리오",
    "## 요구사항",
    "## 열린 가정",
    "# 2부 설계",
    "## 지금 구조",
    "## 바꿀 구조",
    "## 검토한 대안과 버린 이유",
    "## R 행과 모듈의 대응",
    "## 위험",
];

/// Prose for every part-1 key, so the request is one a reader could approve.
const PART_ONE: [(&str, &str); 6] = [
    (
        "summary",
        "시험용 요청서예요. 새 틀로 만든 요청서가 예전처럼 승인되는지 봐요.\n",
    ),
    ("background", "요청서에 배경을 적을 곳이 없었어요.\n"),
    ("goals", "1. 요청서 한 파일에서 배경과 목표를 읽어요.\n"),
    ("non-goals", "1. 승인된 요청서는 다시 쓰지 않아요.\n"),
    (
        "scenarios",
        "### S1 요청서를 승인해요\n\n메인이 절을 채우고 사용자가 승인해요.\n",
    ),
    ("assumptions", "없음.\n"),
];

/// A store with one open run of `work_type` and the request `request new` made for it, with the
/// request's path relative to the scratch directory.
fn goal(work_type: &str) -> (Scratch, String) {
    let t = Scratch::new();
    t.init();
    t.ok(&["run", "new", "prd", "--type", work_type]);
    t.ok(&["request", "new", "--type", work_type, "--title", TITLE]);
    let id = t.read(".dstack/local/CURRENT").trim_end().to_string();
    (t, format!(".dstack/runs/{id}/request.md"))
}

/// Every heading line below the frontmatter.
fn headings(text: &str) -> Vec<String> {
    let below = text.splitn(3, "---\n").nth(2).unwrap_or(text);
    below
        .lines()
        .filter(|line| line.starts_with('#'))
        .map(String::from)
        .collect()
}

/// The summary and the ten section keys, each with the place it names.
fn places() -> Vec<(&'static str, Place)> {
    let mut places = vec![("summary", Place::Summary)];
    places.extend(
        SECTION_KEYS
            .iter()
            .map(|&(key, heading)| (key, Place::Section(heading))),
    );
    places
}

fn code(t: &Scratch, args: &[&str]) -> i32 {
    let out = t.run(args);
    out.status.code().unwrap_or(-1)
}

#[test]
fn R01_prd_templates_show_every_section_heading() {
    for work_type in TYPES {
        let (t, request) = goal(work_type);
        let text = t.read(&request);
        let mut want = vec![format!("# {TITLE}")];
        want.extend(LAYOUT.iter().map(|heading| heading.to_string()));
        assert_eq!(headings(&text), want, "{work_type}");
        // Guidance lives only in comments: each prose place says what goes in it and is still
        // blank to the checker, and the rows keep their own guidance.
        for (key, place) in places() {
            let prose = body(&text, &place).expect("every place of the layout");
            assert!(
                is_blank(prose) && prose.contains("<!--"),
                "{work_type} {key}: want guidance comments only, got {prose:?}"
            );
        }
        let rows = body(&text, &Place::Section(REQUIREMENTS)).expect("the rows section");
        assert!(
            is_blank(rows) && rows.contains("<!--"),
            "{work_type}: {rows:?}"
        );
    }
}

#[test]
fn R01_prd_templates_round_trip_every_section() {
    for work_type in TYPES {
        let (t, request) = goal(work_type);
        let before = t.read(&request);
        for (key, place) in places() {
            let prose = body(&before, &place).expect("every place of the layout");
            t.write("section.md", prose);
            t.ok(&["request", "section", key, "--from", "section.md"]);
            assert_eq!(t.read(&request), before, "{work_type} {key}");
        }
    }
}

#[test]
fn R01_prd_templates_keep_row_approval_and_hash_exit_codes() {
    for work_type in TYPES {
        let (t, request) = goal(work_type);
        for (key, prose) in PART_ONE {
            t.write("section.md", prose);
            assert_eq!(
                code(&t, &["request", "section", key, "--from", "section.md"]),
                0,
                "{work_type} {key}"
            );
        }
        let add = [
            "req",
            "add",
            "첫 요구사항이에요.",
            "--accept",
            "첫 확인이에요.",
        ];
        assert_eq!(code(&t, &add), 0, "{work_type}: req add");
        assert_eq!(code(&t, &["check", "request"]), 0, "{work_type}: check");
        assert_eq!(code(&t, &["request", "approve"]), 0, "{work_type}: approve");
        assert_eq!(code(&t, &["check", "request"]), 0, "{work_type}: approved");
        let edited = format!("{}승인 뒤에 덧붙인 줄이에요.\n", t.read(&request));
        t.write(&request, &edited);
        let out = t.run(&["check", "request"]);
        assert_eq!(out.status.code(), Some(1), "{work_type}: edited");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains("hash mismatch"),
            "{work_type}: the edit is caught by the hash"
        );
    }
}

#[test]
fn R01_prd_templates_quick_keeps_summary_and_requirements() {
    let t = Scratch::new();
    t.init();
    let home = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../claude");
    for work_type in TYPES {
        let slug = format!("q-{work_type}");
        t.ok(&["quick", "new", &slug, "--type", work_type]);
        let text = t.read(&format!(".dstack/quick/{slug}/request.md"));
        let want = [format!("# 빠른 작업: {slug}"), format!("## {REQUIREMENTS}")];
        assert_eq!(headings(&text), want, "{work_type}");
        let template =
            std::fs::read_to_string(home.join(format!("templates/request/{work_type}.md")))
                .expect("the work_type template");
        let summary = body(&text, &Place::Summary).expect("the summary");
        assert!(
            is_blank(summary) && summary.contains("<!--"),
            "{work_type}: {summary:?}"
        );
        assert_eq!(
            summary,
            body(&template, &Place::Summary).expect("template summary")
        );
        let rows = |text: &str| {
            let rows = body(text, &Place::Section(REQUIREMENTS)).expect("the rows section");
            rows.trim_end().to_string()
        };
        assert_eq!(rows(&text), rows(&template), "{work_type}");
    }
}
