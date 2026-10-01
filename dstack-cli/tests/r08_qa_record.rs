// tests/r08_qa_record.rs
// R08 (T16): dstack qa add records a Goal QA scenario for one usage scenario of the request (the
// `### S<n>` headings under `## 사용 시나리오`, comments and fences not counted) or for none,
// keeps its text in qa/QA<n>.md and its row in qa.tsv, refuses without writing anything, and
// part 3 and milestone brief show the QA scenarios.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::Output;

use dstack_cli::core::fsx::sha256_bytes;
use support::{tree, Scratch};

const RUN: &str = "20261003T000000Z_qa";
const DIR: &str = ".dstack/runs/20261003T000000Z_qa";
const REQUEST: &str = ".dstack/runs/20261003T000000Z_qa/request.md";
const MARKER: &str = "<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->";
const GUIDANCE: &str = "<!-- 이 부분은 직접 쓰지 않아요. 계획 대장(plan.json)이 바뀔 때마다 CLI가 Milestone, Plan, Task 분해를 채워요. -->";
const HEADER: &str = "qa\tscenario\tstatus\tbody\tbody_sha256\tartifact\tartifact_sha256\tproduced_by\trecorded_at\tnote";

/// The QA texts of S1, S2 and the check tied to no usage scenario, each with preparation, steps
/// and an expected result, written to qa-1.md, qa-2.md and qa-3.md.
const BODIES: [&str; 3] = [
    "준비: 빈 저장소에서 dstack init을 실행해요.\n단계: 요청서를 만들고 승인해요.\n기대 결과: 승인이 종료 코드 0으로 끝나요.\n",
    "준비: 승인된 요청서가 있어요.\n단계: 계획을 세우고 milestone brief를 봐요.\n기대 결과: QA 시나리오가 목록에 나와요.\n",
    "<!-- 사용 시나리오와 묶이지 않은 확인이에요. -->\n준비: 열린 run이 있어요.\n단계: check request를 실행해요.\n기대 결과: 종료 코드 0이에요.\n",
];

/// A filled Goal request: S1 and S2 are usage scenarios, S3 sits in a comment and S4 in a code
/// fence, so neither is one.
fn request() -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: skip\nreview: on\ncodex_effort: high\ne2e: cli\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\
         # QA 기록 시험\n\n사용 시나리오마다 QA 시나리오를 남기는지 봐요.\n\n# 1부 요청\n\n\
         ## 배경과 문제\n\nGoal을 닫기 전에 사용 흐름을 다시 확인할 곳이 없었어요.\n\n\
         ## 목표\n\n1. 사용 시나리오마다 QA 시나리오를 남겨요.\n\n\
         ## 비목표\n\n1. 결과 기록은 이 시험에서 다루지 않아요.\n\n\
         ## 사용 시나리오\n\n### S1 요청서를 써요\n\n메인이 요청서를 쓰고 승인받아요.\n\n\
         ### S2 계획을 확인해요\n\n메인이 계획을 보여주고 확인받아요.\n\n\
         <!--\n### S3 숨은 시나리오\n-->\n\n```text\n### S4 코드 블록 안의 제목\n```\n\n\
         ## 요구사항\n\n- [ ] **R01** 첫 요구사항이에요. — accept: 첫 확인이에요.\n\n\
         ## 열린 가정\n\n없음.\n\n# 2부 설계\n\n\
         ## 위험\n\nQA 시나리오를 빠뜨릴 수 있어요.\n\n\
         # 3부 계획과 검증\n{MARKER}\n<!-- 계획 대장에서 채워요. -->\n"
    )
}

/// A store whose open run holds the request, with the scope table this checkout ships so lint-ko
/// classifies request.md as approve does, and the three QA texts beside it.
fn scratch() -> Scratch {
    let t = Scratch::new();
    t.init();
    let table = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude/lint/ko-scope.tsv");
    let table = std::fs::read_to_string(table).expect("the shipped scope table");
    t.write(".dstack/project/ko-scope.tsv", &table);
    t.run_fixture(RUN, None, true);
    t.write(REQUEST, &request());
    for (n, body) in BODIES.iter().enumerate() {
        t.write(&format!("qa-{}.md", n + 1), body);
    }
    t
}

fn add(t: &Scratch, scenario: &str, from: &str) -> String {
    t.ok(&["qa", "add", "--scenario", scenario, "--from", from])
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Everything below the marker line: what the CLI regenerates.
fn part3(t: &Scratch) -> String {
    let text = t.read(REQUEST);
    let at = text.find(&format!("{MARKER}\n")).expect("the marker line");
    text[at + MARKER.len() + 1..].to_string()
}

/// check request's exit code and stdout.
fn check(t: &Scratch) -> (i32, String) {
    let out = t.run(&["check", "request"]);
    let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
    (out.status.code().unwrap_or(-1), stdout)
}

#[test]
fn R08_qa_record_mints_ids_body_files_and_index_rows() {
    let t = scratch();
    for (n, scenario) in ["S1", "S2", "none"].into_iter().enumerate() {
        let out = add(&t, scenario, &format!("qa-{}.md", n + 1));
        let id = format!("QA{}", n + 1);
        let first = out.lines().next().unwrap_or_default();
        assert_eq!(first, format!("qa add: run {RUN} — {id} (scenario {scenario})"), "{out}");
        assert_eq!(t.read(&format!("{DIR}/qa/{id}.md")), BODIES[n], "{id} keeps the text as written");
    }
    let index = t.read(&format!("{DIR}/qa.tsv"));
    let mut expected = vec![HEADER.to_string()];
    for (n, scenario) in ["S1", "S2", "none"].into_iter().enumerate() {
        let sha = sha256_bytes(BODIES[n].as_bytes());
        let id = n + 1;
        expected.push(format!("QA{id}\t{scenario}\topen\tqa/QA{id}.md\t{sha}\t-\t-\t-\t-\t-"));
    }
    assert_eq!(index, format!("{}\n", expected.join("\n")));
}

#[test]
fn R08_qa_record_refusals_write_nothing() {
    let t = scratch();
    add(&t, "S1", "qa-1.md");
    t.write("empty.md", "");
    t.write("comment.md", "<!-- 준비, 단계, 기대 결과를 적어요. -->\n\n");
    t.write(".dstack/quick/fix/meta.tsv", "status\topen\n");
    let usage = "dstack: usage: dstack qa add --scenario S<n>|none --from <file> [--run <id>]\n";
    let empty = |file: &str| {
        format!("dstack: --from file {file} is empty (or comments only); a QA scenario holds its preparation, steps and expected result\n")
    };
    let cases: Vec<(Vec<&str>, String)> = vec![
        (vec!["--scenario", "S9", "--from", "qa-2.md"], "dstack: unknown usage scenario: S9 (known: S1 S2)\n".into()),
        (vec!["--scenario", "s2", "--from", "qa-2.md"], "dstack: unknown usage scenario: s2 (known: S1 S2)\n".into()),
        (vec!["--scenario", "S2", "--from", "missing.md"], "dstack: --from file not found: missing.md\n".into()),
        (vec!["--scenario", "S2", "--from", "empty.md"], empty("empty.md")),
        (vec!["--scenario", "none", "--from", "comment.md"], empty("comment.md")),
        (
            vec!["--quick", "fix", "--scenario", "none", "--from", "qa-3.md"],
            "dstack: quick tasks have no Goal QA scenarios; qa add records them for a run\n".into(),
        ),
        (vec!["--scenario", "S2"], usage.into()),
        (vec!["--from", "qa-2.md"], usage.into()),
        (vec!["--scenario", "S2", "--from", "qa-2.md", "extra"], "dstack: unexpected argument: extra\n".into()),
    ];
    for (args, message) in cases {
        let (run, quick) = (tree(&t.0.join(DIR)), tree(&t.0.join(".dstack/quick")));
        let out = t.run(&[&["qa", "add"][..], &args].concat());
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stderr(&out));
        assert_eq!(stderr(&out), message, "{args:?}");
        assert_eq!(tree(&t.0.join(DIR)), run, "{args:?} wrote into the run");
        assert_eq!(tree(&t.0.join(".dstack/quick")), quick, "{args:?} wrote into the quick task");
    }
    assert!(!t.0.join(".dstack/quick/fix/qa").exists());

    // A run without request.md has no usage scenarios to tie a QA scenario to.
    let bare = "20261003T000000Z_bare";
    t.run_fixture(bare, None, false);
    let before = tree(&t.0.join(".dstack/runs").join(bare));
    let out = t.run(&["qa", "add", "--run", bare, "--scenario", "none", "--from", "qa-3.md"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let message = stderr(&out);
    assert!(message.starts_with("dstack: no request.md in "), "{message}");
    assert!(message.ends_with(&format!("{bare} (dstack request new --type <work_type>)\n")), "{message}");
    assert_eq!(tree(&t.0.join(".dstack/runs").join(bare)), before);
    assert!(!t.0.join(".dstack/runs").join(bare).join("qa").exists());
}

#[test]
fn R08_qa_record_a_usage_scenario_heading_inside_a_comment_is_no_id() {
    let t = scratch();
    for hidden in ["S3", "S4"] {
        let out = t.run(&["qa", "add", "--scenario", hidden, "--from", "qa-1.md"]);
        assert_eq!(out.status.code(), Some(1), "{hidden}");
        let message = format!("dstack: unknown usage scenario: {hidden} (known: S1 S2)\n");
        assert_eq!(stderr(&out), message, "{hidden}");
    }
    assert!(!t.0.join(DIR).join("qa.tsv").exists());
    add(&t, "S2", "qa-2.md");
}

#[test]
fn R08_qa_record_part3_shows_the_qa_section_and_keeps_it_after_a_plan_write() {
    let t = scratch();
    t.ok(&["request", "approve"]);
    add(&t, "S1", "qa-1.md");
    let first = "\n## QA 시나리오\n\n- QA1: 사용 시나리오 S1\n  - 상태: `open`\n\
                 \x20 - 내용 첫 줄: 준비: 빈 저장소에서 dstack init을 실행해요.\n";
    // A run without plan.json renders part 3 with no plan and the QA section under it.
    let expected = format!("{GUIDANCE}\n\nGoal `{RUN}`: QA 기록 시험\n\n- Milestone: (비어 있어요)\n{first}");
    assert_eq!(part3(&t), expected);
    let (code, stdout) = check(&t);
    assert_eq!(code, 0, "{stdout}");

    t.ok(&["milestone", "add", "core", "--goal", "QA를 기록해요"]);
    let after = part3(&t);
    assert!(after.contains("## M1 core\n\n- 목표: QA를 기록해요\n"), "{after}");
    assert!(after.ends_with(first), "a plan write keeps the QA section: {after}");

    add(&t, "none", "qa-3.md");
    let after = part3(&t);
    let second = "- QA2: 사용 시나리오 없음\n  - 상태: `open`\n  - 내용 첫 줄: 준비: 열린 run이 있어요.\n";
    assert!(after.contains("## M1 core\n"), "{after}");
    assert!(after.ends_with(&format!("{first}{second}")), "{after}");
    let (code, stdout) = check(&t);
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("hash mismatch"), "{stdout}");
}

#[test]
fn R08_qa_record_milestone_brief_lists_the_qa_scenarios() {
    let t = scratch();
    t.ok(&["milestone", "add", "core", "--goal", "QA를 기록해요"]);
    let plan = ["plan", "add", "first", "--milestone", "M1", "--files", "a/b.sh"];
    t.ok(&[&plan[..], &["--purpose", "기록을 남겨요", "--e2e-focus", "qa add 출력을 봐요"]].concat());
    let brief = t.ok(&["milestone", "brief", "M1"]);
    assert!(brief.contains("\nQA scenarios: (none yet)\nconfirmed: no\n"), "{brief}");
    add(&t, "S1", "qa-1.md");
    add(&t, "S2", "qa-2.md");
    add(&t, "none", "qa-3.md");
    let brief = t.ok(&["milestone", "brief", "M1"]);
    let listed = "\nQA scenarios:\n  QA1: scenario S1, status open\n  QA2: scenario S2, status open\n\
                  \x20 QA3: scenario none, status open\nconfirmed: no\n";
    assert!(brief.contains(listed), "{brief}");
}

#[test]
fn R08_qa_record_lint_ko_reports_no_hits_on_the_rendered_request() {
    let t = scratch();
    add(&t, "S1", "qa-1.md");
    add(&t, "S2", "qa-2.md");
    add(&t, "none", "qa-3.md");
    t.ok(&["milestone", "add", "core", "--goal", "QA를 기록해요"]);
    assert!(part3(&t).contains("## QA 시나리오\n"));
    let out = t.ok(&["lint-ko", REQUEST]);
    assert!(out.contains("files 1, hits 0 (S1 0), unclassified 0"), "{out}");
}

// T52: evidence add --qa records the result of one QA scenario, and e2e brief --goal hands every
// QA scenario to the runner.

/// The scratch store with a run start, so evidence add can date an artifact against it.
fn started() -> Scratch {
    let t = scratch();
    let meta = "status\topen\nowner_session\tfixture\nstarted_at\t2026-01-01T00:00:00Z\n";
    t.write(&format!("{DIR}/meta.tsv"), meta);
    t
}

/// The started store with QA1 (S1), QA2 (S2) and QA3 (no usage scenario).
fn recording() -> Scratch {
    let t = started();
    for (n, scenario) in ["S1", "S2", "none"].into_iter().enumerate() {
        add(&t, scenario, &format!("qa-{}.md", n + 1));
    }
    t
}

/// evidence add --qa with the artifact, the command that produced it and any further options.
fn record(t: &Scratch, qa: &str, artifact: &str, more: &[&str]) -> Output {
    let args = ["evidence", "add", "--qa", qa, "--artifact", artifact, "--produced-by", "dstack status"];
    t.run(&[&args[..], more].concat())
}

/// The qa.tsv cells of one QA scenario.
fn qa_row(t: &Scratch, qa: &str) -> Vec<String> {
    let index = t.read(&format!("{DIR}/qa.tsv"));
    let line = index.lines().skip(1).find(|line| line.split('\t').next() == Some(qa));
    line.expect("the QA row").split('\t').map(String::from).collect()
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[test]
fn R08_qa_record_evidence_add_qa_records_met_and_a_failure_with_its_reason() {
    let t = recording();
    let capture = "$ dstack request approve\nQA1 승인이 종료 코드 0으로 끝났어요.\nexit: 0\n";
    t.write("qa1-run.txt", capture);
    let out = record(&t, "QA1", "qa1-run.txt", &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let printed = stdout(&out);
    let sha = sha256_bytes(capture.as_bytes());
    assert_eq!(printed.lines().next(), Some(format!("evidence add: run {RUN} — QA1 met").as_str()), "{printed}");
    let summary = format!("  artifact qa1-run.txt ({} bytes, sha256 {}…)", capture.len(), &sha[..8]);
    assert!(printed.contains(&summary), "{printed}");
    let row = qa_row(&t, "QA1");
    assert_eq!((row[2].as_str(), row[5].as_str(), row[6].as_str()), ("met", "qa1-run.txt", sha.as_str()));
    assert_eq!((row[7].as_str(), row[9].as_str()), ("dstack status", "-"));
    assert_ne!(row[8], "-", "recorded_at is stamped");
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `met`\n"), "{}", part3(&t));

    let capture = "$ dstack milestone brief M1\nQA2 목록에 QA 시나리오가 없어요.\nexit: 0\n";
    t.write("qa2-run.txt", capture);
    let out = record(&t, "QA2", "qa2-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out).lines().next(), Some(format!("evidence add: run {RUN} — QA2 failed").as_str()));
    let row = qa_row(&t, "QA2");
    assert_eq!((row[2].as_str(), row[9].as_str()), ("failed", "목록이 비어 있어요"));
    let after = part3(&t);
    assert!(after.contains("- QA2: 사용 시나리오 S2\n  - 상태: `failed`\n"), "{after}");
    assert!(after.contains("- QA3: 사용 시나리오 없음\n  - 상태: `open`\n"), "{after}");
    assert_eq!(qa_row(&t, "QA3")[2], "open");
}

#[test]
fn R08_qa_record_evidence_add_qa_refusals_leave_the_ledger_as_it_was() {
    let t = recording();
    t.write("named.txt", "$ dstack status\nQA1 확인했어요.\nexit: 0\n");
    t.write("other.txt", "$ dstack status\nQA10 QA1_ok 확인했어요.\nexit: 0\n");
    let combined = "dstack: --qa does not combine with --r, --case or --kind — usage: dstack evidence add ";
    let cases: [(&str, &str, &[&str], &str); 7] = [
        ("QA1", "named.txt", &["--status", "skipped"], "dstack: a skipped QA result needs a reason (--note)\n"),
        ("QA1", "named.txt", &["--status", "blocked", "--note", " "], "dstack: a blocked QA result needs a reason (--note)\n"),
        ("QA1", "other.txt", &[], "dstack: the artifact must name QA1 as a whole word; other.txt does not — record the run that mentions it\n"),
        ("QA1", "named.txt", &["--r", "R01"], combined),
        ("QA1", "named.txt", &["--case", "c1"], combined),
        ("QA1", "named.txt", &["--kind", "cli"], combined),
        ("QA9", "named.txt", &[], "dstack: unknown QA scenario: QA9 (known: QA1 QA2 QA3)\n"),
    ];
    for (qa, artifact, more, message) in cases {
        let before = tree(&t.0.join(DIR));
        let out = record(&t, qa, artifact, more);
        assert_eq!(out.status.code(), Some(1), "{more:?}: {}", stderr(&out));
        assert!(stderr(&out).starts_with(message), "{more:?}: {}", stderr(&out));
        assert_eq!(tree(&t.0.join(DIR)), before, "{more:?} wrote into the run");
    }
    assert_eq!(qa_row(&t, "QA1")[2], "open");

    let out = record(&t, "QA1", "named.txt", &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let before = tree(&t.0.join(DIR));
    let out = record(&t, "QA1", "named.txt", &["--status", "skipped", "--note", "다시 기록해요"]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    let message = "dstack: QA1 is already recorded (status met); a recorded QA result is never overwritten\n";
    assert_eq!(stderr(&out), message);
    assert_eq!(tree(&t.0.join(DIR)), before, "a second record wrote into the run");
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `met`\n"), "{}", part3(&t));
}

#[test]
fn R08_qa_record_e2e_brief_goal_prints_every_body_and_lists_the_open_ones() {
    let t = started();
    let header = format!("e2e brief: run {RUN} — Goal QA — e2e: cli\n");
    let before = tree(&t.0.join(".dstack"));
    let out = t.ok(&["e2e", "brief", "--goal"]);
    assert_eq!(out, format!("{header}(no QA scenarios — run dstack qa add)\n"));
    assert_eq!(tree(&t.0.join(".dstack")), before, "e2e brief wrote to the store");

    for (n, scenario) in ["S1", "S2", "none"].into_iter().enumerate() {
        add(&t, scenario, &format!("qa-{}.md", n + 1));
    }
    t.write("qa2-run.txt", "$ dstack milestone brief M1\nQA2 목록에 나와요.\nexit: 0\n");
    let out = record(&t, "QA2", "qa2-run.txt", &[]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));

    let before = tree(&t.0.join(".dstack"));
    let out = t.ok(&["e2e", "brief", "--goal"]);
    assert_eq!(tree(&t.0.join(".dstack")), before, "e2e brief wrote to the store");
    let expected = format!(
        "{header}\n## QA scenarios\n\
         \n### QA1 — usage scenario: S1 요청서를 써요 — status: open\n{}\
         \n### QA2 — usage scenario: S2 계획을 확인해요 — status: met\n{}\
         \n### QA3 — usage scenario: (none) — status: open\n{}\
         \n## Open QA scenarios\n| QA | usage scenario |\n|---|---|\n\
         | QA1 | S1 요청서를 써요 |\n| QA3 | (none) |\n",
        BODIES[0], BODIES[1], BODIES[2]
    );
    assert_eq!(out, expected);
}
