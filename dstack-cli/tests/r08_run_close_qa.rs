// tests/r08_run_close_qa.rs
// R08 (T61): dstack run close holds the Goal-close QA check whatever Plans are left (verify
// --at-close, D-47): while a Plan is in progress plain verify only informs about an open result
// and an uncovered usage scenario, but run close refuses them and leaves the run open. Once every
// result is recorded, or retired and recorded again, it closes; a request without usage scenarios
// and run close --abandon close exactly as before.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use support::Scratch;

const RUN: &str = "20261003T000000Z_run-close-qa";
const DIR: &str = ".dstack/runs/20261003T000000Z_run-close-qa";
const META: &str = ".dstack/runs/20261003T000000Z_run-close-qa/meta.tsv";

/// The two usage scenarios of the Goal request.
const SCENARIOS: &str = "## 사용 시나리오\n\n### S1 요청서를 써요\n\n메인이 요청서를 쓰고 승인받아요.\n\n\
                         ### S2 계획을 확인해요\n\n메인이 계획을 보여주고 확인받아요.\n\n";

/// The QA texts written to qa-1.md, qa-2.md and qa-3.md.
const BODIES: [&str; 3] = [
    "준비: 빈 저장소에서 dstack init을 실행해요.\n단계: 요청서를 만들고 승인해요.\n기대 결과: 승인이 종료 코드 0으로 끝나요.\n",
    "준비: 승인된 요청서가 있어요.\n단계: 계획을 세우고 milestone brief를 봐요.\n기대 결과: 계획이 목록에 나와요.\n",
    "준비: 열린 run이 있어요.\n단계: check request를 실행해요.\n기대 결과: 종료 코드 0이에요.\n",
];

/// One Plan in progress with one task covering R01, so only the QA check decides verify.
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

/// A started Goal run whose one Plan is still in progress and whose one R row is proven, so verify
/// exits 0 unless the QA check refuses; the three QA texts sit beside it.
fn goal(scenarios: &str) -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(META, "status\topen\nowner_session\tfixture\nstarted_at\t2026-01-01T00:00:00Z\n");
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

fn add(t: &Scratch, scenario: &str, from: &str) {
    t.ok(&["qa", "add", "--scenario", scenario, "--from", from]);
}

/// evidence add --qa with an artifact that names the QA id, and the status and note in `more`.
fn record(t: &Scratch, qa: &str, artifact: &str, more: &[&str]) {
    t.write(artifact, &format!("$ dstack status\n{qa} 확인했어요.\nexit: 0\n"));
    let args = ["evidence", "add", "--qa", qa, "--artifact", artifact, "--produced-by", "dstack status"];
    t.ok(&[&args[..], more].concat());
}

fn status(t: &Scratch) -> String {
    let meta = t.read(META);
    meta.lines().find_map(|line| line.strip_prefix("status\t")).unwrap_or_default().to_string()
}

const REFUSED: &str = "refused: dstack verify failed for 20261003T000000Z_run-close-qa — fix the lines above, or close with --abandon <why>\n";

#[test]
fn R08_run_close_qa_refuses_open_and_uncovered_while_a_plan_is_in_progress() {
    let t = goal(SCENARIOS);
    add(&t, "S1", "qa-1.md");
    let (code, verify) = call(&t, &["verify"]);
    assert_eq!(code, 0, "plain verify only informs while a Plan is in progress: {verify}");
    assert!(verify.contains("before Goal close → ok"), "{verify}");

    let meta = t.read(META);
    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!(closed, 1, "{out}");
    let open = "QA1 (S1) FAIL (open): no recorded result — record it: dstack evidence add --qa QA1 --artifact <file> --produced-by <command>\n";
    let uncovered = "S2 FAIL (uncovered): no QA scenario — add one: dstack qa add --scenario S2 --from <file>\n";
    let summary = "qa: usage scenarios 2, QA scenarios 1, met 0, skipped 0, blocked 0, open 1, uncovered 1, refused 2, at Goal close → refused\n";
    assert!(out.contains(&format!("{open}{uncovered}{summary}")), "{out}");
    assert!(out.ends_with(REFUSED), "{out}");
    assert_eq!(t.read(META), meta, "a refused close leaves the run as it was");
    assert_eq!(t.read(".dstack/local/CURRENT"), format!("{RUN}\n"));
}

#[test]
fn R08_run_close_qa_closes_once_the_results_are_recorded_again() {
    let t = goal(SCENARIOS);
    add(&t, "S1", "qa-1.md");
    add(&t, "S2", "qa-2.md");
    record(&t, "QA2", "qa2-run.txt", &[]);
    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!((closed, status(&t).as_str()), (1, "open"), "QA1 is open: {out}");
    assert!(out.contains("QA1 (S1) FAIL (open): "), "{out}");

    record(&t, "QA1", "qa1-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!((closed, status(&t).as_str()), (1, "open"), "QA1 failed: {out}");
    assert!(out.contains("QA1 (S1) FAIL (failed): 목록이 비어 있어요"), "{out}");

    t.ok(&["evidence", "retire", "--qa", "QA1", "--why", "목록을 고쳤어요"]);
    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!((closed, status(&t).as_str()), (1, "open"), "the retired QA1 is open again: {out}");
    assert!(out.contains("QA1 (S1) FAIL (open): "), "{out}");

    record(&t, "QA1", "qa1-rerun.txt", &[]);
    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!(closed, 0, "{out}");
    let summary = "qa: usage scenarios 2, QA scenarios 2, met 2, skipped 0, blocked 0, open 0, uncovered 0, refused 0, at Goal close → ok\n";
    assert!(out.contains(&format!("QA1 (S1) met\nQA2 (S2) met\n{summary}")), "{out}");
    assert!(out.lines().last().is_some_and(|line| line.starts_with(&format!("closed {RUN} (closed) at "))), "{out}");
    assert_eq!(status(&t), "closed");
    assert_eq!(t.read(".dstack/local/CURRENT"), "");
}

#[test]
fn R08_run_close_qa_a_request_without_usage_scenarios_closes_as_before() {
    let t = goal("");
    add(&t, "none", "qa-3.md");
    let (code, verify) = call(&t, &["verify", "--run", RUN]);
    assert_eq!(code, 0, "{verify}");
    assert!(!verify.contains("qa:"), "{verify}");

    let (closed, out) = call(&t, &["run", "close", RUN]);
    assert_eq!(closed, 0, "{out}");
    let rest = out.strip_prefix(verify.as_str()).unwrap_or_else(|| panic!("run close printed another verify:\n{verify}\n{out}"));
    assert!(rest.starts_with(&format!("closed {RUN} (closed) at ")) && rest.lines().count() == 1, "{out}");
    assert_eq!(status(&t), "closed");
}

#[test]
fn R08_run_close_qa_abandon_closes_as_before() {
    let t = goal(SCENARIOS);
    add(&t, "S1", "qa-1.md");
    let (closed, out) = call(&t, &["run", "close", RUN, "--abandon", "범위를 바꿨어요"]);
    assert_eq!(closed, 0, "{out}");
    assert!(out.starts_with(&format!("closed {RUN} (abandoned) at ")) && out.lines().count() == 1, "abandon runs no verify: {out}");
    let meta = t.read(META);
    assert_eq!(status(&t), "abandoned");
    assert!(meta.lines().any(|line| line == "closed_reason\t범위를 바꿨어요"), "{meta}");
    assert_eq!(t.read(".dstack/local/CURRENT"), "");
}
