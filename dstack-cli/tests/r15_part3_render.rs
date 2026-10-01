// tests/r15_part3_render.rs
// R15 (T11): a Goal request carries part 3 under `# 3부 계획과 검증` with the marker line right
// below the heading; the approval hash covers the bytes through the marker line, so part 3 may
// change after approval while one character changed above it fails, deleting the marker with
// part 3 fails too, and a request without the marker keeps its whole-file hash (D-38, D-39).
// R15 (T50): every plan verb regenerates part 3 from plan.json in hierarchy order, check request
// names an empty Milestone goal or Plan purpose, and a legacy request is never touched.
// R15 (T51): R rows exist only above the marker, so a row written below it is no row and fails
// check request by its line; planning prose is escaped so it opens no comment and forms no row.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;

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
         ## 요구사항\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\
         - [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요.\n\n\
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

/// Every byte through the marker line, its line break included: what the approval covers.
fn through_marker(text: &str) -> &str {
    let at = text.find(&format!("\n{MARKER}\n")).expect("the marker line");
    &text[..at + MARKER.len() + 2]
}

/// Everything below the marker line: what plan verbs regenerate.
fn part3(text: &str) -> &str {
    &text[through_marker(text).len()..]
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
fn R15_part3_hash_stamps_the_bytes_through_the_marker() {
    let text = request(true);
    let t = approved(&text);
    assert_eq!(stamp(&t), sha256_bytes(through_marker(&text).as_bytes()));
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
        ("part 3 emptied", through_marker(&text).to_string()),
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

#[test]
fn R15_part3_hash_fails_when_the_marker_and_part3_are_deleted() {
    let text = request(true);
    let t = approved(&text);
    let at = text.find(&format!("{MARKER}\n")).expect("the marker line");
    t.write(REQUEST, &text[..at]);
    assert_mismatch(&t, "the marker and part 3 deleted");
    t.write(REQUEST, &text);
    assert_matches(&t, "restored");
}

/// The guidance every render starts with, as the templates write it under the marker.
const GUIDANCE: &str = "<!-- 이 부분은 직접 쓰지 않아요. 계획 대장(plan.json)이 바뀔 때마다 CLI가 Milestone, Plan, Task 분해를 채워요. -->";

/// An approved request in a store with the scope table this checkout ships, so lint-ko and
/// milestone confirm read request.md as the approve-time tests do.
fn scoped(text: &str) -> Scratch {
    let t = scratch(text);
    let table = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude/lint/ko-scope.tsv");
    let table = std::fs::read_to_string(table).expect("the shipped scope table");
    t.write(".dstack/project/ko-scope.tsv", &table);
    t.ok(&["request", "approve"]);
    t
}

/// The planning steps of the fixture, each with a line its part 3 shows afterwards: two
/// Milestones with goals; M1 holds P1 (task T1) and P2 (after P1, tasks T2 and T3, T3 after T2
/// and without a purpose); M2 holds P3 without tasks.
const PLANNING: [(&[&str], &str); 8] = [
    (&["milestone", "add", "core", "--goal", "설치를 한 번에 끝내요"], "## M1 core\n\n- 목표: 설치를 한 번에 끝내요\n"),
    (&["milestone", "add", "wrap", "--goal", "보고서를 정리해요"], "## M2 wrap\n\n- 목표: 보고서를 정리해요\n"),
    (
        &["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh", "--purpose", "설치 스크립트를 정리해요", "--e2e-focus", "설치 출력이 그대로예요"],
        "### P1 first\n\n- 목적: 설치 스크립트를 정리해요\n",
    ),
    (
        &["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh,c/e.sh", "--deps", "P1", "--purpose", "설치를 연결해요", "--e2e-focus", "설치 기록을 봐요"],
        "### P2 second\n\n- 목적: 설치를 연결해요\n",
    ),
    (
        &["plan", "add", "third", "--milestone", "M2", "--files", "e/f.sh", "--purpose", "보고서 틀을 만들어요", "--e2e-focus", "보고서 출력을 봐요"],
        "### P3 third\n\n- 목적: 보고서 틀을 만들어요\n",
    ),
    (
        &["task", "add", "write-lib", "--plan", "P1", "--covers", "R01", "--files", "a/b.sh", "--purpose", "라이브러리를 써요"],
        "- Task T1 write-lib\n  - 목적: 라이브러리를 써요\n",
    ),
    (
        &["task", "add", "wire", "--plan", "P2", "--covers", "R01,R02", "--files", "c/d.sh", "--purpose", "스크립트를 이어요"],
        "- Task T2 wire\n  - 목적: 스크립트를 이어요\n",
    ),
    (&["task", "add", "docs", "--plan", "P2", "--covers", "R02", "--files", "c/e.sh", "--deps", "T2"], "- Task T3 docs\n  - 목적: (비어 있어요)\n"),
];

/// Part 3 once every planning step ran.
fn planned_part3() -> String {
    format!(
        "{GUIDANCE}\n\nGoal `{RUN}`: 3부 해시 시험\n\n\
         ## M1 core\n\n- 목표: 설치를 한 번에 끝내요\n- 확인한 Plan: (비어 있어요)\n\n\
         ### P1 first\n\n- 목적: 설치 스크립트를 정리해요\n- E2E 초점: 설치 출력이 그대로예요\n\
         - 다루는 R 행: R01\n- 선언 파일: `a/b.sh`\n- 선행 Plan: (비어 있어요)\n- 상태: `ready`\n\
         - Task T1 write-lib\n  - 목적: 라이브러리를 써요\n  - 다루는 R 행: R01\n\
         \x20 - 선언 파일: `a/b.sh`\n  - 선행 Task: (비어 있어요)\n\
         \x20 - 상태: 아직 커밋하지 않았어요\n\n\
         ### P2 second\n\n- 목적: 설치를 연결해요\n- E2E 초점: 설치 기록을 봐요\n\
         - 다루는 R 행: R01, R02\n- 선언 파일: `c/d.sh`, `c/e.sh`\n- 선행 Plan: P1\n- 상태: `pending`\n\
         - Task T2 wire\n  - 목적: 스크립트를 이어요\n  - 다루는 R 행: R01, R02\n\
         \x20 - 선언 파일: `c/d.sh`\n  - 선행 Task: (비어 있어요)\n\
         \x20 - 상태: 아직 커밋하지 않았어요\n\
         - Task T3 docs\n  - 목적: (비어 있어요)\n  - 다루는 R 행: R02\n\
         \x20 - 선언 파일: `c/e.sh`\n  - 선행 Task: T2\n\
         \x20 - 상태: 아직 커밋하지 않았어요\n\n\
         ## M2 wrap\n\n- 목표: 보고서를 정리해요\n- 확인한 Plan: (비어 있어요)\n\n\
         ### P3 third\n\n- 목적: 보고서 틀을 만들어요\n- E2E 초점: 보고서 출력을 봐요\n\
         - 다루는 R 행: (비어 있어요)\n- 선언 파일: `e/f.sh`\n- 선행 Plan: (비어 있어요)\n- 상태: `ready`\n\
         - Task: (비어 있어요)\n"
    )
}

fn planned(text: &str) -> Scratch {
    let t = scoped(text);
    for (args, _) in PLANNING {
        t.ok(args);
    }
    t
}

#[test]
fn R15_part3_render_shows_every_unit_in_hierarchy_order() {
    let text = request(true);
    let t = planned(&text);
    let after = t.read(REQUEST);
    assert_eq!(through_marker(&after), through_marker(&text));
    assert_eq!(part3(&after), planned_part3());
    assert_matches(&t, "planned");
}

#[test]
fn R15_part3_render_follows_every_plan_verb() {
    let text = request(true);
    let t = scoped(&text);
    let lifecycle: [(&[&str], &str); 5] = [
        (&["plan", "edit", "P3", "--purpose", "보고서 틀을 고쳐요"], "- 목적: 보고서 틀을 고쳐요\n"),
        (&["milestone", "confirm", "M1"], "- 확인한 Plan: P1, P2\n"),
        (&["plan", "start", "P1"], "- 상태: `in-progress`\n"),
        (&["plan", "done", "P1"], "- 상태: `done`\n"),
        (&["milestone", "confirm", "M2"], "- 확인한 Plan: P3\n"),
    ];
    let mut before = t.read(REQUEST);
    for (args, shown) in PLANNING.into_iter().chain(lifecycle) {
        let what = args.join(" ");
        t.ok(args);
        let after = t.read(REQUEST);
        assert_ne!(part3(&after), part3(&before), "{what}: part 3 regenerated");
        assert!(part3(&after).contains(shown), "{what}: {shown:?} in {}", part3(&after));
        assert_eq!(through_marker(&after), through_marker(&text), "{what}");
        assert_matches(&t, &what);
        before = after;
    }
    // P2 waited on P1, so plan done made it ready, and part 3 shows that too.
    assert!(part3(&before).contains("- 선행 Plan: P1\n- 상태: `ready`\n"), "{before}");
    // A hand edit below the marker keeps the stamp, and plan render writes the plan back.
    t.write(REQUEST, &format!("{}손으로 고친 3부예요.\n", through_marker(&before)));
    assert_matches(&t, "part 3 edited by hand");
    t.ok(&["plan", "render"]);
    assert_eq!(t.read(REQUEST), before, "plan render regenerates part 3");
}

#[test]
fn R15_part3_render_keeps_parts_1_and_2_hashed() {
    let text = request(true);
    let t = planned(&text);
    let rendered = t.read(REQUEST);
    assert_eq!(part3(&rendered), planned_part3());
    for (what, from, to) in [("part 1", "없었어요.", "없었어요!"), ("part 2", "놓칠 수 있어요.", "놓칠 수 있어요!")] {
        t.write(REQUEST, &edit(&rendered, from, to));
        assert_mismatch(&t, what);
        t.write(REQUEST, &rendered);
        assert_matches(&t, what);
    }
}

#[test]
fn R15_part3_render_check_names_an_empty_goal_and_purpose() {
    let t = planned(&request(true));
    t.ok(&["milestone", "add", "later"]);
    t.ok(&["plan", "add", "fourth", "--milestone", "M3", "--files", "g/h.sh"]);
    let part = part3(&t.read(REQUEST)).to_string();
    assert!(part.contains("## M3 later\n\n- 목표: (비어 있어요)\n"), "{part}");
    assert!(part.contains("### P4 fourth\n\n- 목적: (비어 있어요)\n"), "{part}");
    let (code, stdout) = check(&t);
    assert_eq!(code, 1, "{stdout}");
    let goal = "  part 3: milestone M3 has no goal (dstack milestone edit M3 --goal <text>)\n";
    let purpose = "  part 3: plan P4 has no purpose (dstack plan edit P4 --purpose <text>)\n";
    assert!(stdout.contains(goal) && stdout.contains(purpose), "{stdout}");
    assert!(stdout.contains(", failures 2\n"), "{stdout}");
    t.ok(&["milestone", "edit", "M3", "--goal", "나중에 정리해요"]);
    t.ok(&["plan", "edit", "P4", "--purpose", "남은 일을 끝내요"]);
    assert_matches(&t, "goal and purpose filled");

    // Without the marker nothing new is checked.
    let legacy = scoped(&request(false));
    legacy.ok(&["milestone", "add", "later"]);
    legacy.ok(&["plan", "add", "fourth", "--milestone", "M1", "--files", "g/h.sh"]);
    let (code, stdout) = check(&legacy);
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("part 3:"), "{stdout}");
}

#[test]
fn R15_part3_render_leaves_a_legacy_request_byte_identical() {
    let text = request(false);
    let t = planned(&text);
    t.ok(&["milestone", "confirm", "M1"]);
    t.ok(&["plan", "start", "P1"]);
    t.ok(&["plan", "done", "P1"]);
    t.ok(&["plan", "render"]);
    assert_eq!(t.read(REQUEST), text);
    assert_matches(&t, "legacy after plan verbs");
}

#[test]
fn R15_part3_render_passes_lint_ko_and_adds_no_rows() {
    let text = request(true);
    let rows = |t: &Scratch| -> String {
        let (_, stdout) = check(t);
        stdout.lines().find(|line| line.starts_with("  rows: ")).expect("a rows line").to_string()
    };
    let t = scoped(&text);
    let unplanned = rows(&t);
    for (args, _) in PLANNING {
        t.ok(args);
    }
    assert_eq!(rows(&t), unplanned);
    assert!(unplanned.starts_with("  rows: 2 (live 2,"), "{unplanned}");
    let after = t.read(REQUEST);
    assert_eq!(part3(&after), planned_part3());
    for line in part3(&after).lines() {
        let bare = line.trim_start();
        assert!(!bare.starts_with("- [") && !bare.contains("**R"), "row-shaped: {line}");
    }
    let out = t.ok(&["lint-ko", REQUEST]);
    assert!(out.contains("files 1, hits 0 (S1 0), unclassified 0"), "{out}");
}

#[test]
fn R15_part3_rows_below_the_marker_fail_check_and_count_nothing() {
    let text = request(true);
    let t = approved(&text);
    let row = "- [ ] **R03** 새 요구사항이에요. — accept: 새 검사를 통과해요.";
    let added = format!("{text}{row}\n");
    t.write(REQUEST, &added);
    let lineno = added.lines().count();
    let (code, stdout) = check(&t);
    assert_eq!(code, 1, "{stdout}");
    let named = format!(
        "  line {lineno}: R rows belong in ## 요구사항 above part 3; part 3 is generated: {row}\n"
    );
    assert!(stdout.contains(&named), "{stdout}");
    assert!(stdout.contains("  rows: 2 (live 2,"), "{stdout}");
    assert!(!stdout.contains("hash mismatch") && stdout.contains(", failures 1\n"), "{stdout}");

    // req add puts the next row right after R02, above part 3, and leaves part 3 as it was.
    t.ok(&["req", "add", "넷째 요구사항이에요.", "--accept", "넷째 확인이에요."]);
    let after = t.read(REQUEST);
    assert_eq!(part3(&after), part3(&added));
    let r02 = "- [ ] **R02** 둘째 요구사항이에요. — accept: 둘째 확인이에요.\n";
    let at = after.find(r02).expect("the R02 row") + r02.len();
    let next = after[at..].lines().next().unwrap_or_default();
    assert!(next.starts_with("- [ ] **R") && next.contains(" 넷째 요구사항이에요."), "{after}");
}

#[test]
fn R15_part3_render_escapes_planning_prose() {
    let t = scratch(&request(true));
    let (code, stdout) = check(&t);
    assert_eq!(code, 0, "unplanned: {stdout}");
    t.ok(&["milestone", "add", "core", "--goal", "설치를 끝내요"]);
    t.ok(&["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh", "--purpose", "**R01** 요구사항을 구현해요"]);
    t.ok(&["plan", "add", "second", "--milestone", "M1", "--files", "c/d.sh", "--purpose", "<!-- 숨겨요"]);
    let part = part3(&t.read(REQUEST)).to_string();
    assert!(part.contains("### P1 first\n\n- 목적: **R01** 요구사항을 구현해요\n"), "{part}");
    assert!(part.contains("### P2 second\n\n- 목적: &lt;!-- 숨겨요\n"), "{part}");
    let generated = part.strip_prefix(GUIDANCE).expect("the guidance first");
    assert!(!generated.contains('<'), "{part}");
    let (code, stdout) = check(&t);
    assert_eq!(code, 0, "planned: {stdout}");
    assert!(stdout.contains("  rows: 2 (live 2,"), "{stdout}");

    // The outline still reads: the risks section is rewritten and part 3 stays as rendered.
    t.write("section.md", "계획 문장이 요청서 문법을 깨뜨릴 수 있어요.\n");
    t.ok(&["request", "section", "risks", "--from", "section.md"]);
    let after = t.read(REQUEST);
    assert!(after.contains("## 위험\n\n계획 문장이 요청서 문법을 깨뜨릴 수 있어요.\n"), "{after}");
    assert_eq!(part3(&after), part);
}

#[test]
fn R15_part3_rows_of_a_legacy_request_are_read_to_the_end() {
    let text = request(false);
    let t = scratch(&format!("{text}- [ ] **R03** 셋째 요구사항이에요. — accept: 셋째 확인이에요.\n"));
    let (_, stdout) = check(&t);
    assert!(stdout.contains("  rows: 3 (live 3,"), "{stdout}");
    assert!(!stdout.contains("part 3 is generated"), "{stdout}");
}
