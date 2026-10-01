// tests/r15_part3_render.rs
// R15 (T11): a Goal request carries part 3 under `# 3부 계획과 검증` with the marker line right
// below the heading; the approval hash covers the bytes above the marker, so part 3 may change
// after approval while one character changed above it fails, and a request without the marker
// keeps the whole-file hash it always had (D-38).

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use dstack_cli::core::fsx::sha256_bytes;
use dstack_cli::core::roots::Roots;
use dstack_cli::core::target::{Target, TargetKind};
use dstack_cli::handoff::snapshot::collect;
use support::Scratch;

const TYPES: [&str; 5] = ["web-ui", "http-api", "cli", "library", "docs-writing"];

const RUN: &str = "20261002T000000Z_part3";
const REQUEST: &str = ".dstack/runs/20261002T000000Z_part3/request.md";
const STAMP: &str = ".dstack/runs/20261002T000000Z_part3/request.approved";

/// The visible heading of part 3 and the marker line right under it, written out here as the
/// oracle the templates and the hash are held to.
const HEADING: &str = "# 3부 계획과 검증";
const MARKER: &str = "<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->";

const HANDOFF_REFUSAL: &str =
    "handoff requires the current request to match its valid approval stamp";

/// A filled Goal request with all three parts; `{marker}` is replaced by the marker line or by
/// nothing for a legacy request that shows the heading but has no boundary.
fn request(marker: bool) -> String {
    let marker = if marker { format!("{MARKER}\n") } else { String::new() };
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: skip\nreview: on\ncodex_effort: high\ne2e: cli\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\
         # 3부 해시 시험\n\n승인 해시가 1부와 2부만 지키는지 봐요.\n\n# 1부 요청\n\n\
         ## 배경과 문제\n\n요청서에 계획을 적을 곳이 없었어요.\n\n\
         ## 목표\n\n1. 계획이 바뀌어도 승인이 유지돼요.\n\n\
         ## 비목표\n\n1. 1부와 2부의 보호는 줄이지 않아요.\n\n\
         ## 사용 시나리오\n\n### S1 계획을 고쳐요\n\n메인이 계획을 고치고 요청서를 검사해요.\n\n\
         ## 요구사항\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n\
         ## 열린 가정\n\n없음.\n\n# 2부 설계\n\n\
         ## 위험\n\n승인 뒤의 수정을 놓칠 수 있어요.\n\n\
         {HEADING}\n{marker}<!-- 계획 대장에서 채워요. -->\n### M1 첫 묶음\n\n- P1 첫 계획이에요.\n"
    )
}

fn scratch(text: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    // The identity handoff checks before it reads the request and its stamp.
    t.write(
        &format!(".dstack/runs/{RUN}/meta.tsv"),
        &format!(
            "id\t{RUN}\nstatus\topen\nowner_session\tfixture\nworktree\t{}\n",
            t.0.display()
        ),
    );
    t.write(
        &format!(".dstack/runs/{RUN}/decisions.md"),
        "| D | Decision | Affects | Status |\n|---|---|---|---|\n",
    );
    t.write(REQUEST, text);
    t
}

fn approved(text: &str) -> Scratch {
    let t = scratch(text);
    t.ok(&["request", "approve"]);
    t
}

fn stamp(t: &Scratch) -> String {
    let text = t.read(STAMP);
    text.split_whitespace().nth(1).expect("the hash field").to_string()
}

/// Every byte above the marker line, the marker line excluded.
fn above_marker(text: &str) -> &str {
    let at = text.find(&format!("\n{MARKER}\n")).expect("the marker line");
    &text[..at + 1]
}

/// check request's exit code and stdout.
fn check(t: &Scratch) -> (i32, String) {
    let out = t.run(&["check", "request"]);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    (out.status.code().unwrap_or(-1), stdout)
}

/// The hash line `request show` prints.
fn shown(t: &Scratch) -> String {
    let out = t.ok(&["request", "show"]);
    out.lines()
        .find(|line| line.starts_with("hash: "))
        .expect("a hash line")
        .to_string()
}

/// Whether handoff refuses the run for its approval stamp. The scratch is no git repository, so
/// a snapshot that gets past the stamp still fails later; only the stamp refusal is asked here.
fn handoff_refuses(t: &Scratch) -> bool {
    let store = t.0.join(".dstack");
    let roots = Roots {
        main_root: t.0.clone(),
        wt_root: t.0.clone(),
        runs: store.join("runs"),
        local: store.join("local"),
        quick: store.join("quick"),
        store: store.clone(),
    };
    let target = Target {
        kind: TargetKind::Run,
        id: RUN.to_string(),
        dir: store.join("runs").join(RUN),
    };
    match collect(&roots, &target) {
        Err(error) => error.message() == HANDOFF_REFUSAL,
        Ok(_) => false,
    }
}

/// `text` with the first `from` replaced by `to`, which must change it.
fn edit(text: &str, from: &str, to: &str) -> String {
    let edited = text.replacen(from, to, 1);
    assert_ne!(edited, text, "{from:?} is in the request");
    edited
}

fn assert_matches(t: &Scratch, what: &str) {
    let (code, stdout) = check(t);
    assert_eq!(code, 0, "{what}: {stdout}");
    assert_eq!(shown(t), "hash: matches the approved file", "{what}");
    assert!(!handoff_refuses(t), "{what}: handoff refused the stamp");
}

fn assert_mismatch(t: &Scratch, what: &str) {
    let (code, stdout) = check(t);
    assert_eq!(code, 1, "{what}: {stdout}");
    assert!(stdout.contains("hash mismatch"), "{what}: {stdout}");
    assert!(shown(t).starts_with("hash: MISMATCH"), "{what}");
    assert!(handoff_refuses(t), "{what}: handoff accepted the stamp");
}

#[test]
fn R15_part3_hash_templates_put_the_marker_under_the_part3_heading() {
    for work_type in TYPES {
        let t = Scratch::new();
        t.init();
        t.ok(&["run", "new", "part3", "--type", work_type]);
        t.ok(&["request", "new", "--type", work_type, "--title", "3부 틀 시험"]);
        let id = t.read(".dstack/local/CURRENT").trim_end().to_string();
        let text = t.read(&format!(".dstack/runs/{id}/request.md"));
        let lines: Vec<&str> = text.lines().collect();
        let at = lines
            .iter()
            .position(|line| *line == HEADING)
            .unwrap_or_else(|| panic!("{work_type}: no '{HEADING}'"));
        assert_eq!(lines.get(at + 1), Some(&MARKER), "{work_type}");
        assert_eq!(text.matches(MARKER).count(), 1, "{work_type}");
        // Part 3 is the last part and not a request section: nothing heads it but the heading.
        let risks = lines.iter().position(|line| *line == "## 위험").expect("## 위험");
        assert!(risks < at, "{work_type}: part 3 follows ## 위험");
        let below = &lines[at + 1..];
        assert!(
            below.iter().all(|line| !line.starts_with('#') && !line.contains("(키:")),
            "{work_type}: {below:?}"
        );
        assert!(
            below.len() > 1 && below[1].starts_with("<!--") && below[1].ends_with("-->"),
            "{work_type}: one guidance comment line under the marker: {below:?}"
        );
    }
}

#[test]
fn R15_part3_hash_stamps_the_bytes_above_the_marker() {
    let text = request(true);
    let t = approved(&text);
    assert_eq!(stamp(&t), sha256_bytes(above_marker(&text).as_bytes()));
    assert_matches(&t, "approved");
    for (what, edited) in [
        ("a line at the end", format!("{text}- P2 새 계획이에요.\n")),
        (
            "the guidance replaced",
            edit(&text, "<!-- 계획 대장에서 채워요. -->\n", "### M2 새 묶음\n"),
        ),
        (
            "a line right under the marker",
            edit(&text, &format!("{MARKER}\n"), &format!("{MARKER}\n새 줄이에요.\n")),
        ),
        ("part 3 emptied", format!("{}{MARKER}\n", above_marker(&text))),
    ] {
        t.write(REQUEST, &edited);
        assert_matches(&t, what);
    }
}

#[test]
fn R15_part3_hash_catches_one_character_above_the_marker() {
    let text = request(true);
    let t = approved(&text);
    for (what, edited) in [
        ("part 1", edit(&text, "없었어요.", "없었어요!")),
        ("part 2", edit(&text, "놓칠 수 있어요.", "놓칠 수 있어요!")),
        ("the part 3 heading", edit(&text, HEADING, "# 3부 계획과 검정")),
        ("the marker removed", edit(&text, &format!("{MARKER}\n"), "")),
        ("the marker line edited", edit(&text, &format!("{MARKER}\n"), &format!("{MARKER}x\n"))),
        (
            "a second marker above",
            edit(&text, "# 1부 요청\n", &format!("{MARKER}\n# 1부 요청\n")),
        ),
    ] {
        t.write(REQUEST, &edited);
        assert_mismatch(&t, what);
        t.write(REQUEST, &text);
        assert_matches(&t, what);
    }
}

#[test]
fn R15_part3_hash_keeps_the_whole_file_without_the_marker() {
    let text = request(false);
    assert!(!text.contains(MARKER) && text.contains(HEADING));
    let t = approved(&text);
    assert_eq!(stamp(&t), sha256_bytes(text.as_bytes()));
    assert_matches(&t, "legacy approved");
    t.write(REQUEST, &format!("{text}- P2 새 계획이에요.\n"));
    assert_mismatch(&t, "legacy, a line at the end");
}

#[test]
fn R15_part3_hash_request_section_risks_keeps_the_heading_and_marker() {
    let t = Scratch::new();
    t.init();
    t.ok(&["run", "new", "part3", "--type", "cli"]);
    t.ok(&["request", "new", "--type", "cli", "--title", "3부 쓰기 시험"]);
    let id = t.read(".dstack/local/CURRENT").trim_end().to_string();
    let path = format!(".dstack/runs/{id}/request.md");
    let before = t.read(&path);
    let part3 = |text: &str| -> String {
        let at = text.find(&format!("\n{HEADING}\n{MARKER}\n")).expect("part 3");
        text[at..].to_string()
    };
    t.write("section.md", "승인 뒤의 수정을 놓칠 수 있어요.\n");
    t.ok(&["request", "section", "risks", "--from", "section.md"]);
    let after = t.read(&path);
    assert_ne!(after, before, "the risks section was written");
    assert!(after.contains("## 위험\n\n승인 뒤의 수정을 놓칠 수 있어요.\n"), "{after}");
    assert_eq!(part3(&after), part3(&before));
}
