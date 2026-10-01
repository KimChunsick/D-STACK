// tests/r08_qa_close.rs
// R08 (T17): dstack verify refuses a Goal close while a usage scenario has no QA scenario, a QA
// scenario has no result, a result failed, or a recorded text or artifact changed; skipped and
// blocked pass with their reason in dstack report, whose exit agrees with verify. A request
// without usage scenarios and a quick task keep their output and exit code (D-41, D-42).

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use support::Scratch;

const RUN: &str = "20261003T000000Z_close";
const DIR: &str = ".dstack/runs/20261003T000000Z_close";

/// The two usage scenarios of the Goal request.
const SCENARIOS: &str = "## 사용 시나리오\n\n### S1 요청서를 써요\n\n메인이 요청서를 쓰고 승인받아요.\n\n\
                         ### S2 계획을 확인해요\n\n메인이 계획을 보여주고 확인받아요.\n\n";

/// The QA texts written to qa-1.md, qa-2.md and qa-3.md.
const BODIES: [&str; 3] = [
    "준비: 빈 저장소에서 dstack init을 실행해요.\n단계: 요청서를 만들고 승인해요.\n기대 결과: 승인이 종료 코드 0으로 끝나요.\n",
    "준비: 승인된 요청서가 있어요.\n단계: 계획을 세우고 milestone brief를 봐요.\n기대 결과: 계획이 목록에 나와요.\n",
    "준비: 열린 run이 있어요.\n단계: check request를 실행해요.\n기대 결과: 종료 코드 0이에요.\n",
];

/// One task covering R01, so report's coverage reads ok and only the QA check decides its exit.
const PLAN: &str = r#"{ "v": 2,
  "milestones": [ {"id":"M1","slug":"close","order":1} ],
  "plans": [ {"id":"P1","milestone":"M1","slug":"close","files":["a"],"deps":[],
              "status":"in-progress","worktree":"","started_at":"","done_at":"",
              "tasks":[ {"id":"T1","slug":"close","covers":["R01"],"files":["a"],
                         "deps":[],"commit":"","done_at":""} ] } ] }
"#;

const ROUND: &str = "| R | verdict | evidence in the diff |\n|---|---|---|\n| R01 | covered | r01-run.txt |\n\nVERDICT: approve\n";

fn request(scenarios: &str) -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: skip\nreview: off\ncodex_effort: high\ne2e: cli\nunit_tests: off\n\
         visual: none\nkorean_polish: off\n---\n# QA 마감 시험\n\nGoal을 닫기 전에 QA 결과를 확인해요.\n\n\
         {scenarios}## 요구사항\n\n\
         - [ ] **R01** 명령이 센 수를 출력해요. — accept: 표준 출력에 \"checked N\"이 나와요.\n"
    )
}

/// A started Goal run whose one R row is proven — cli evidence, a sealed covered round and a task —
/// so verify and report exit 0 unless the QA check refuses; the three QA texts sit beside it.
fn goal(scenarios: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(&format!("{DIR}/meta.tsv"), "status\topen\nowner_session\tfixture\nstarted_at\t2026-01-01T00:00:00Z\n");
    t.write(&format!("{DIR}/request.md"), &request(scenarios));
    t.write(&format!("{DIR}/request.approved"), "sha256 -  approved_at 2026-01-01T00:00:00Z\n");
    t.ok(&["cases", "sync"]);
    t.write("r01-run.txt", "$ dstack status\nR01 checked 1\nexit: 0\n");
    let r01 = ["evidence", "add", "--r", "R01", "--case", "c1", "--kind", "cli"];
    t.ok(&[&r01[..], &["--artifact", "r01-run.txt", "--produced-by", "dstack status"]].concat());
    t.write(&format!("{DIR}/plan.json"), PLAN);
    t.write(&format!("{DIR}/review/codex-review-001.md"), ROUND);
    for (n, body) in BODIES.iter().enumerate() {
        t.write(&format!("qa-{}.md", n + 1), body);
    }
    t
}

/// One call's exit code and stdout.
fn call(t: &Scratch, args: &[&str]) -> (i32, String) {
    let out = t.run(args);
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into_owned())
}

/// verify and report of the current target: report exits as verify does (the QA check is shared).
fn judged(t: &Scratch) -> (i32, String, String) {
    let (code, verify) = call(t, &["verify"]);
    let (reported, report) = call(t, &["report"]);
    assert_eq!(reported, code, "report disagrees with verify:\n{verify}\n{report}");
    (code, verify, report)
}

fn add(t: &Scratch, scenario: &str, from: &str) {
    t.ok(&["qa", "add", "--scenario", scenario, "--from", from]);
}

/// evidence add --qa with an artifact that names the QA id, and the status and note in `more`.
fn record(t: &Scratch, qa: &str, artifact: &str, more: &[&str]) {
    t.write(artifact, &format!("$ dstack status\n{qa} 확인했어요.\nexit: 0\n"));
    let args = ["evidence", "add", "--qa", qa, "--artifact", artifact, "--produced-by", "dstack status"];
    t.ok(&[&args[..], more].concat());
}

/// The QA lines verify prints: one per QA scenario or uncovered usage scenario, then `qa:`.
fn qa_lines(verify: &str) -> Vec<&str> {
    let qa = |line: &&str| {
        let id = line.split(' ').next().unwrap_or_default();
        line.starts_with("qa:")
            || ["QA", "S"].iter().any(|p| id.strip_prefix(p).is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit())))
    };
    verify.lines().filter(qa).collect()
}

/// The QA table report prints, from its header through its `qa:` line; empty when it has none.
fn qa_table(report: &str) -> Vec<&str> {
    let lines: Vec<&str> = report.lines().collect();
    let Some(at) = lines.iter().position(|line| *line == "| QA | scenario | status | reason | artifact |") else {
        return Vec::new();
    };
    let end = lines[at..].iter().position(|line| line.starts_with("qa:")).map_or(lines.len(), |n| at + n + 1);
    lines[at..end].to_vec()
}

const VERIFIED: &str = "verify: checked 1 R, ok 1, failed 0, abstain 0, blocked 0, accepted 0";

#[test]
fn R08_qa_close_a_request_without_usage_scenarios_and_a_quick_task_keep_their_output() {
    // A Goal request written before the usage scenario section: an open QA scenario tied to no
    // usage scenario changes neither verify nor report.
    let t = goal("");
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 0, "{verify}");
    add(&t, "none", "qa-3.md");
    assert!(t.0.join(DIR).join("qa.tsv").is_file());
    assert_eq!(judged(&t), (code, verify.clone(), report.clone()));
    assert!(qa_lines(&verify).is_empty(), "{verify}");
    assert!(qa_table(&report).is_empty(), "{report}");
    assert!(verify.ends_with(&format!("{VERIFIED} → exit 0\n")), "{verify}");

    // A quick task never holds a Goal close, with or without usage scenarios in its request.
    let quick = ".dstack/quick/fix";
    t.write(&format!("{quick}/meta.tsv"), "status\topen\n");
    t.write(&format!("{quick}/request.md"), &request(""));
    let before = (call(&t, &["verify", "--quick", "fix"]), call(&t, &["report", "--quick", "fix"]));
    t.write(&format!("{quick}/request.md"), &request(SCENARIOS));
    let after = (call(&t, &["verify", "--quick", "fix"]), call(&t, &["report", "--quick", "fix"]));
    assert_eq!(after, before);
    assert!(qa_lines(&after.0 .1).is_empty(), "{}", after.0 .1);
    assert!(qa_table(&after.1 .1).is_empty(), "{}", after.1 .1);
}

#[test]
fn R08_qa_close_a_usage_scenario_without_qa_refuses_and_run_close_refuses() {
    let t = goal(SCENARIOS);
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let why = |s: &str| format!("no QA scenario — add one: dstack qa add --scenario {s} --from <file>");
    let uncovered = |s: &str| format!("{s} FAIL (uncovered): {}", why(s));
    let summary = "qa: usage scenarios 2, QA scenarios 0, met 0, skipped 0, blocked 0, refused 2 → refused";
    assert_eq!(qa_lines(&verify), [uncovered("S1").as_str(), &uncovered("S2"), summary], "{verify}");
    assert!(verify.ends_with(&format!("{VERIFIED} → exit 1\n")), "{verify}");
    let row = |s: &str| format!("| - | {s} | uncovered | {} | - |", why(s));
    let header = ["| QA | scenario | status | reason | artifact |", "|---|---|---|---|---|"];
    assert_eq!(qa_table(&report), [header[0], header[1], &row("S1"), &row("S2"), summary], "{report}");
    assert!(report.contains("requirement coverage rate = MET / (total - WITHDRAWN - DEFERRED) = 1/1 = 100.0%\n\n| QA |"), "{report}");

    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!(closed, 1, "{out}");
    assert!(out.ends_with(&format!("refused: dstack verify failed for {RUN} — fix the lines above, or close with --abandon <why>\n")), "{out}");
    assert!(t.read(&format!("{DIR}/meta.tsv")).lines().any(|line| line == "status\topen"));
}

#[test]
fn R08_qa_close_an_open_or_failed_result_refuses() {
    let t = goal(SCENARIOS);
    add(&t, "S1", "qa-1.md");
    add(&t, "S2", "qa-2.md");
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let open = |qa: &str, s: &str| {
        format!("{qa} ({s}) FAIL (open): no recorded result — record it: dstack evidence add --qa {qa} --artifact <file> --produced-by <command>")
    };
    let summary = "qa: usage scenarios 2, QA scenarios 2, met 0, skipped 0, blocked 0, refused 2 → refused";
    assert_eq!(qa_lines(&verify), [open("QA1", "S1").as_str(), &open("QA2", "S2"), summary], "{verify}");
    assert!(report.contains("| QA1 | S1 | open | no recorded result — record it: dstack evidence add --qa QA1 "), "{report}");

    record(&t, "QA1", "qa1-run.txt", &[]);
    record(&t, "QA2", "qa2-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let failed = "QA2 (S2) FAIL (failed): 목록이 비어 있어요; a failed result does not close the Goal";
    let summary = "qa: usage scenarios 2, QA scenarios 2, met 1, skipped 0, blocked 0, refused 1 → refused";
    assert_eq!(qa_lines(&verify), ["QA1 (S1) met", failed, summary], "{verify}");
    let rows = qa_table(&report);
    assert_eq!(rows[2], "| QA1 | S1 | met | - | qa1-run.txt |", "{report}");
    assert_eq!(rows[3], "| QA2 | S2 | failed | 목록이 비어 있어요; a failed result does not close the Goal | qa2-run.txt |", "{report}");
}

#[test]
fn R08_qa_close_a_changed_artifact_or_text_refuses_naming_the_file() {
    let t = goal(SCENARIOS);
    add(&t, "S1", "qa-1.md");
    add(&t, "S2", "qa-2.md");
    record(&t, "QA1", "qa1-run.txt", &[]);
    record(&t, "QA2", "qa2-run.txt", &[]);
    assert_eq!(judged(&t).0, 0);

    let artifact = t.0.join("qa1-run.txt");
    let kept = t.read("qa1-run.txt");
    t.write("qa1-run.txt", &format!("{kept}나중에 손으로 고쳤어요.\n"));
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let changed = format!("artifact {} changed after it was recorded (sha256 mismatch)", artifact.display());
    assert!(qa_lines(&verify).contains(&format!("QA1 (S1) FAIL (met): {changed}").as_str()), "{verify}");
    assert!(report.contains(&format!("| QA1 | S1 | met | {changed} | qa1-run.txt |")), "{report}");
    std::fs::remove_file(&artifact).expect("drop the artifact");
    let (code, verify, _) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let gone = format!("QA1 (S1) FAIL (met): artifact {} is gone", artifact.display());
    assert!(qa_lines(&verify).contains(&gone.as_str()), "{verify}");
    t.write("qa1-run.txt", &kept);
    assert_eq!(judged(&t).0, 0);

    let body = t.0.join(DIR).join("qa/QA2.md");
    t.write(&format!("{DIR}/qa/QA2.md"), &format!("{}단계: 하나 더 해요.\n", BODIES[1]));
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let changed = format!("text {} changed after it was recorded (sha256 mismatch)", body.display());
    assert!(qa_lines(&verify).contains(&format!("QA2 (S2) FAIL (met): {changed}").as_str()), "{verify}");
    assert!(report.contains(&format!("| QA2 | S2 | met | {changed} | qa2-run.txt |")), "{report}");
    std::fs::remove_file(&body).expect("drop the text");
    let (code, verify, _) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let gone = format!("QA2 (S2) FAIL (met): text {} is gone", body.display());
    assert!(qa_lines(&verify).contains(&gone.as_str()), "{verify}");
}

#[test]
fn R08_qa_close_every_result_met_passes_and_run_close_closes() {
    let t = goal(SCENARIOS);
    add(&t, "S1", "qa-1.md");
    add(&t, "S2", "qa-2.md");
    add(&t, "none", "qa-3.md");
    record(&t, "QA1", "qa1-run.txt", &[]);
    record(&t, "QA2", "qa2-run.txt", &[]);
    let (code, verify, _) = judged(&t);
    assert_eq!(code, 1, "{verify}");
    let open = "QA3 (none) FAIL (open): no recorded result — record it: dstack evidence add --qa QA3 --artifact <file> --produced-by <command>";
    assert!(qa_lines(&verify).contains(&open), "a check tied to no usage scenario needs its result too: {verify}");

    record(&t, "QA3", "qa3-run.txt", &[]);
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 0, "{verify}");
    let summary = "qa: usage scenarios 2, QA scenarios 3, met 3, skipped 0, blocked 0, refused 0 → ok";
    assert_eq!(qa_lines(&verify), ["QA1 (S1) met", "QA2 (S2) met", "QA3 (none) met", summary], "{verify}");
    assert!(verify.contains(&format!("QA3 (none) met\n{summary}\nbranch containment: ")), "{verify}");
    assert!(verify.ends_with(&format!("{VERIFIED} → exit 0\n")), "{verify}");
    assert_eq!(qa_table(&report)[4], "| QA3 | none | met | - | qa3-run.txt |", "{report}");

    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!(closed, 0, "{out}");
    assert!(t.read(&format!("{DIR}/meta.tsv")).lines().any(|line| line == "status\tclosed"));
}

#[test]
fn R08_qa_close_skipped_and_blocked_pass_with_their_reason_in_report() {
    let t = goal(SCENARIOS);
    add(&t, "S1", "qa-1.md");
    add(&t, "S2", "qa-2.md");
    record(&t, "QA1", "qa1-run.txt", &["--status", "skipped", "--note", "실행할 환경이 없어서 건너뛰어요"]);
    record(&t, "QA2", "qa2-run.txt", &["--status", "blocked", "--note", "권한 | 승인을 기다려요"]);
    let (code, verify, report) = judged(&t);
    assert_eq!(code, 0, "{verify}");
    let summary = "qa: usage scenarios 2, QA scenarios 2, met 0, skipped 1, blocked 1, refused 0 → ok";
    let lines = ["QA1 (S1) skipped: 실행할 환경이 없어서 건너뛰어요", "QA2 (S2) blocked: 권한 | 승인을 기다려요", summary];
    assert_eq!(qa_lines(&verify), lines, "{verify}");
    let rows = qa_table(&report);
    assert_eq!(rows[2], "| QA1 | S1 | skipped | 실행할 환경이 없어서 건너뛰어요 | qa1-run.txt |", "{report}");
    assert_eq!(rows[3], "| QA2 | S2 | blocked | 권한 \\| 승인을 기다려요 | qa2-run.txt |", "{report}");
    assert_eq!(rows[4], summary, "{report}");
}
