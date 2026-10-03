// tests/r08_qa_rerun.rs
// R08 (T57): dstack evidence retire --qa reopens a recorded Goal QA result with a reason, keeps the
// retired result and the reason in the append-only qa-history.tsv, refuses without writing
// anything, and lets evidence add --qa record the scenario again (D-46). R08 (T58): verify at Goal
// close refuses the reopened scenario until it is recorded again. R08 (T60): the same retire run
// again after a failed qa.tsv rewrite finishes it without a second history row (D-47). R08 (T64):
// every history row carries its attempt and a retire stays pending in qa-retire.pending until its
// last step, so the same retire or a record finishes an interrupted one, and identical recordings
// within one second keep a history row and a reason each (D-53). R08 (T67): while a retire is
// pending, verify — before Goal close or at it — and report refuse, naming the QA (D-56). R08
// (T68): a history ending without a newline is refused and left unchanged (D-57).

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::time::Duration;

use dstack_cli::core::fsx::sha256_bytes;
use support::{tree, Scratch};

const RUN: &str = "20261003T000000Z_rerun";
const DIR: &str = ".dstack/runs/20261003T000000Z_rerun";
const MARKER: &str = "<!-- dstack 3부 경계: 이 줄 아래는 CLI가 계획 대장에서 다시 만들고, 승인 해시는 이 줄 위까지만 봐요. 이 줄은 지우거나 고치지 않아요. -->";
const HISTORY: &str = "qa\tretired_at\twas_status\tartifact\tartifact_sha256\tproduced_by\trecorded_at\tresult_note\twhy\tattempt";
const PENDING: &str = ".dstack/runs/20261003T000000Z_rerun/qa-retire.pending";

/// The QA texts of S1 and S2, written to qa-1.md and qa-2.md.
const BODIES: [&str; 2] = [
    "준비: 빈 저장소에서 dstack init을 실행해요.\n단계: 요청서를 만들고 승인해요.\n기대 결과: 승인이 종료 코드 0으로 끝나요.\n",
    "준비: 승인된 요청서가 있어요.\n단계: 계획을 세우고 milestone brief를 봐요.\n기대 결과: 계획이 목록에 나와요.\n",
];

const ROUND: &str = "| R | verdict | evidence in the diff |\n|---|---|---|\n| R01 | covered | r01-run.txt |\n\nVERDICT: approve\n";

/// A Goal request with the usage scenarios S1 and S2 and the part 3 marker, so part 3 lists the QA.
fn request() -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: skip\nreview: off\ncodex_effort: high\ne2e: cli\nunit_tests: off\n\
         visual: none\nkorean_polish: off\n---\n# QA 재실행 시험\n\n실패한 QA 결과를 거두고 다시 기록해요.\n\n\
         ## 사용 시나리오\n\n### S1 요청서를 써요\n\n메인이 요청서를 쓰고 승인받아요.\n\n\
         ### S2 계획을 확인해요\n\n메인이 계획을 보여주고 확인받아요.\n\n\
         ## 요구사항\n\n\
         - [ ] **R01** 명령이 센 수를 출력해요. — accept: 표준 출력에 \"checked N\"이 나와요.\n\n\
         # 3부 계획과 검증\n{MARKER}\n<!-- 계획 대장에서 채워요. -->\n"
    )
}

/// A started Goal run without plan.json whose R01 is proven — cli evidence and a sealed covered
/// round — with QA1 (S1) and QA2 (S2) open.
fn goal() -> Scratch {
    let t = Scratch::new();
    t.init();
    t.run_fixture(RUN, None, true);
    t.write(&format!("{DIR}/meta.tsv"), "status\topen\nowner_session\tfixture\nstarted_at\t2026-01-01T00:00:00Z\n");
    t.write(&format!("{DIR}/request.md"), &request());
    t.write(&format!("{DIR}/request.approved"), "sha256 -  approved_at 2026-01-01T00:00:00Z\n");
    t.ok(&["cases", "sync"]);
    t.write("r01-run.txt", "$ dstack status\nR01 checked 1\nexit: 0\n");
    let r01 = ["evidence", "add", "--r", "R01", "--case", "c1", "--kind", "cli"];
    t.ok(&[&r01[..], &["--artifact", "r01-run.txt", "--produced-by", "dstack status"]].concat());
    t.write(&format!("{DIR}/review/codex-review-001.md"), ROUND);
    for (n, body) in BODIES.iter().enumerate() {
        let from = format!("qa-{}.md", n + 1);
        t.write(&from, body);
        t.ok(&["qa", "add", "--scenario", &format!("S{}", n + 1), "--from", &from]);
    }
    t
}

/// evidence add --qa with an artifact that names the QA id, and the status and note in `more`.
fn record(t: &Scratch, qa: &str, artifact: &str, more: &[&str]) {
    t.write(artifact, &format!("$ dstack status\n{qa} 확인했어요.\nexit: 0\n"));
    let args = ["evidence", "add", "--qa", qa, "--artifact", artifact, "--produced-by", "dstack status"];
    t.ok(&[&args[..], more].concat());
}

fn retire(t: &Scratch, args: &[&str]) -> Output {
    t.run(&[&["evidence", "retire"][..], args].concat())
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The qa.tsv cells of one QA scenario.
fn qa_row(t: &Scratch, qa: &str) -> Vec<String> {
    let index = t.read(&format!("{DIR}/qa.tsv"));
    let line = index.lines().skip(1).find(|line| line.split('\t').next() == Some(qa));
    line.expect("the QA row").split('\t').map(String::from).collect()
}

/// The rows of qa-history.tsv after its header, which has to be the first line.
fn history(t: &Scratch) -> Vec<Vec<String>> {
    let text = t.read(&format!("{DIR}/qa-history.tsv"));
    assert_eq!(text.lines().next(), Some(HISTORY), "{text}");
    text.lines().skip(1).map(|line| line.split('\t').map(String::from).collect()).collect()
}

fn history_file(t: &Scratch) -> PathBuf {
    t.0.join(DIR).join("qa-history.tsv")
}

/// Everything below the marker line: what the CLI regenerates.
fn part3(t: &Scratch) -> String {
    let text = t.read(&format!("{DIR}/request.md"));
    let at = text.find(&format!("{MARKER}\n")).expect("the marker line");
    text[at + MARKER.len() + 1..].to_string()
}

/// The cells a fresh qa add row carries for QA<n> of S<n>.
fn fresh(n: usize) -> Vec<String> {
    let sha = sha256_bytes(BODIES[n - 1].as_bytes());
    let cells = [format!("QA{n}"), format!("S{n}"), "open".into(), format!("qa/QA{n}.md"), sha];
    cells.into_iter().chain(std::iter::repeat_n("-".to_string(), 5)).collect()
}

#[test]
fn R08_qa_rerun_retire_reopens_a_failed_qa_and_keeps_its_result_in_history() {
    let t = goal();
    record(&t, "QA1", "qa1-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    record(&t, "QA2", "qa2-run.txt", &[]);
    let held = qa_row(&t, "QA1");
    let other = qa_row(&t, "QA2");
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `failed`\n"), "{}", part3(&t));

    let out = retire(&t, &["--qa", "QA1", "--why", "목록을 고쳤어요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let printed = stdout(&out);
    let lines: Vec<&str> = printed.lines().collect();
    assert_eq!(lines[0], "retired QA1 (was failed): 목록을 고쳤어요", "{printed}");
    assert!(lines.iter().any(|line| line.trim_start().starts_with("dstack evidence add --qa QA1 ")), "{printed}");

    assert_eq!(qa_row(&t, "QA1"), fresh(1), "the reopened row is what qa add wrote");
    assert_eq!(qa_row(&t, "QA2"), other, "the other row stays as it was");
    let rows = history(&t);
    assert_eq!(rows.len(), 1, "{rows:?}");
    let row = &rows[0];
    assert_eq!(row[0], "QA1");
    assert!(row[1].ends_with('Z') && row[1] != "-", "retired_at is stamped: {row:?}");
    let kept = [&held[2], &held[5], &held[6], &held[7], &held[8], &held[9]];
    assert_eq!(row[2..8].iter().collect::<Vec<_>>(), kept, "the retired result stays as history");
    assert_eq!((row[3].as_str(), row[7].as_str()), ("qa1-run.txt", "목록이 비어 있어요"));
    assert_eq!(row[8], "목록을 고쳤어요");
    assert_eq!(row[9], "1", "the first retire of QA1 is its attempt 1");
    assert_eq!(row.len(), 10, "{row:?}");
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `open`\n"), "{}", part3(&t));
    assert!(part3(&t).contains("- QA2: 사용 시나리오 S2\n  - 상태: `met`\n"), "{}", part3(&t));
}

#[test]
fn R08_qa_rerun_refusals_write_nothing() {
    let t = goal();
    record(&t, "QA1", "qa1-run.txt", &[]);
    t.write(".dstack/quick/fix/meta.tsv", "status\topen\n");
    let combined = "dstack: --qa does not combine with --r or --case — usage: dstack evidence retire ";
    let reason = "dstack: retiring a QA result needs a reason (--why) — usage: dstack evidence retire ";
    let cases: [(&[&str], &str); 9] = [
        (&["--qa", "QA1", "--why", "다시 해요", "--r", "R01"], combined),
        (&["--qa", "QA1", "--why", "다시 해요", "--case", "c1"], combined),
        (&["--r", "R01", "--case", "c1", "--why", "다시 해요", "--qa", "QA1"], combined),
        (&["--qa", "QA1", "--why", "다시 해요", "--quick", "fix"], "dstack: quick tasks have no Goal QA scenarios; evidence retire --qa retires them for a run\n"),
        (&["--qa", "QA9", "--why", "다시 해요"], "dstack: unknown QA scenario: QA9 (known: QA1 QA2)\n"),
        (&["--qa", "QA1"], reason),
        (&["--qa", "QA1", "--why", " "], reason),
        (&["--qa", "QA2", "--why", "다시 해요"], "dstack: QA2 is open — no recorded result; nothing to retire\n"),
        (&["--qa=QA2", "--why=다시 해요"], "dstack: QA2 is open — no recorded result; nothing to retire\n"),
    ];
    for (args, message) in cases {
        let before = (tree(&t.0.join(DIR)), tree(&t.0.join(".dstack/quick/fix")));
        let out = retire(&t, args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stderr(&out));
        assert!(stderr(&out).starts_with(message), "{args:?}: {}", stderr(&out));
        let after = (tree(&t.0.join(DIR)), tree(&t.0.join(".dstack/quick/fix")));
        assert_eq!(after, before, "{args:?} wrote something");
    }
    assert!(!history_file(&t).exists());
    assert_eq!(qa_row(&t, "QA1")[2], "met");
    assert_eq!(qa_row(&t, "QA2"), fresh(2));
}

#[test]
fn R08_qa_rerun_recording_again_works_and_the_history_survives() {
    let t = goal();
    record(&t, "QA1", "qa1-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    let first = qa_row(&t, "QA1");
    let out = retire(&t, &["--qa", "QA1", "--why", "목록을 고쳤어요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));

    record(&t, "QA1", "qa1-second.txt", &[]);
    let second = qa_row(&t, "QA1");
    assert_eq!((second[2].as_str(), second[5].as_str()), ("met", "qa1-second.txt"));
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `met`\n"), "{}", part3(&t));
    let out = retire(&t, &["--qa", "QA1", "--why", "산출물을 잘못 골랐어요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out).lines().next(), Some("retired QA1 (was met): 산출물을 잘못 골랐어요"));
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `open`\n"), "{}", part3(&t));

    record(&t, "QA1", "qa1-third.txt", &["--status", "skipped", "--note", "환경이 없어요"]);
    let rows = history(&t);
    assert_eq!(rows.len(), 2, "two retires leave two history rows: {rows:?}");
    let was = |row: &Vec<String>| (row[2].clone(), row[3].clone(), row[4].clone(), row[7].clone(), row[8].clone());
    let note = "목록이 비어 있어요".to_string();
    assert_eq!(was(&rows[0]), ("failed".into(), first[5].clone(), first[6].clone(), note, "목록을 고쳤어요".into()));
    let why = "산출물을 잘못 골랐어요".to_string();
    assert_eq!(was(&rows[1]), ("met".into(), second[5].clone(), second[6].clone(), "-".into(), why));
    let third = qa_row(&t, "QA1");
    assert_eq!((third[2].as_str(), third[5].as_str(), third[9].as_str()), ("skipped", "qa1-third.txt", "환경이 없어요"));
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `skipped`\n"), "{}", part3(&t));
}

#[test]
fn R08_qa_rerun_a_qa_option_inside_a_value_stays_r_retire() {
    let t = goal();
    record(&t, "QA1", "qa1-run.txt", &[]);
    let index = t.read(&format!("{DIR}/qa.tsv"));
    let out = retire(&t, &["--r", "R01", "--case", "c1", "--why", "--qa=QA1"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out).lines().next(), Some("retired R01 case c1 (was met): --qa=QA1"));
    let cases = t.read(&format!("{DIR}/cases.tsv"));
    let row = cases.lines().find(|line| line.starts_with("R01\tc1\t")).expect("the R01 row");
    assert_eq!(row.split('\t').nth(3), Some("retired"), "{row}");
    assert!(row.contains(": --qa=QA1 (was met"), "{row}");
    assert_eq!(t.read(&format!("{DIR}/qa.tsv")), index, "R retire wrote into the QA ledger");
    assert!(!history_file(&t).exists());

    // --qa after other options still chooses the QA retire.
    let out = retire(&t, &["--why", "다시 해요", "--qa", "QA1"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(qa_row(&t, "QA1"), fresh(1));
    assert_eq!(history(&t).len(), 1);
}

/// `args` started in the background with the environment `Scratch::run` gives, as another
/// worktree's writer would run beside this one.
fn spawn(t: &Scratch, args: &[&str]) -> Child {
    Command::new(env!("CARGO_BIN_EXE_dstack"))
        .current_dir(&t.0)
        .env("DSTACK_ROOT", &t.0)
        .env("DSTACK_HOME", PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../claude"))
        .env("DSTACK_DEPS", t.0.join("deps.tsv"))
        .env("CLAUDE_CODE_SESSION_ID", "mode-settings-test")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawned")
}

#[test]
fn R08_qa_rerun_retire_waits_for_the_run_lock() {
    let t = goal();
    record(&t, "QA1", "qa1-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    let lock = t.0.join(DIR).join("lock");
    let index = format!("{DIR}/qa.tsv");
    // Another worktree's writer holds the run lock; this worktree's own lock is free.
    std::fs::create_dir(&lock).expect("the run lock");
    let before = t.read(&index);
    let mut child = spawn(&t, &["evidence", "retire", "--qa", "QA1", "--why", "목록을 고쳤어요"]);
    std::thread::sleep(Duration::from_millis(500));
    let waiting = child.try_wait().expect("polled").is_none();
    assert!(waiting, "the retire finished while the run lock was held");
    assert_eq!(t.read(&index), before, "the retire wrote qa.tsv while the run lock was held");
    assert!(!history_file(&t).exists(), "the retire wrote its history while the run lock was held");
    std::fs::remove_dir(&lock).expect("the run lock released");
    let out = child.wait_with_output().expect("finished");
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!lock.exists(), "the retire left the run lock behind");
    assert_eq!(qa_row(&t, "QA1"), fresh(1));
    assert_eq!(history(&t).len(), 1);
}

#[test]
fn R08_qa_rerun_verify_at_goal_close_refuses_the_reopened_qa_until_it_is_recorded_again() {
    // No plan.json: the Goal is closing, so an open QA result refuses.
    let t = goal();
    let verify = |t: &Scratch| {
        let out = t.run(&["verify"]);
        (out.status.code().unwrap_or(-1), stdout(&out))
    };
    record(&t, "QA1", "qa1-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    record(&t, "QA2", "qa2-run.txt", &[]);
    let (code, out) = verify(&t);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("QA1 (S1) FAIL (failed): 목록이 비어 있어요; a failed result does not close the Goal\n"), "{out}");

    let retired = retire(&t, &["--qa", "QA1", "--why", "목록을 고쳤어요"]);
    assert_eq!(retired.status.code(), Some(0), "{}", stderr(&retired));
    let (code, out) = verify(&t);
    assert_eq!(code, 1, "{out}");
    let open = "QA1 (S1) FAIL (open): no recorded result — record it: dstack evidence add --qa QA1 --artifact <file> --produced-by <command>\n";
    assert!(out.contains(open), "{out}");

    record(&t, "QA1", "qa1-second.txt", &[]);
    let (code, out) = verify(&t);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("QA1 (S1) met\nQA2 (S2) met\nqa: "), "{out}");
    assert_eq!(history(&t).len(), 1);
}

#[test]
fn R08_qa_rerun_the_same_retire_finishes_a_partial_one_with_one_history_row() {
    let t = goal();
    record(&t, "QA1", "qa1-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    let (index, request) = (format!("{DIR}/qa.tsv"), format!("{DIR}/request.md"));
    let before = [t.read(&index), t.read(&request)];
    let held = qa_row(&t, "QA1");
    let args = ["--qa", "QA1", "--why", "목록을 고쳤어요"];
    let out = retire(&t, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    // What a failed qa.tsv rewrite leaves: the pending row and the history row written, qa.tsv
    // and part 3 as they were. The pending row is the history row the retire wrote.
    t.write(PENDING, &format!("{}\n", history(&t)[0].join("\t")));
    t.write(&index, &before[0]);
    t.write(&request, &before[1]);
    assert_eq!((qa_row(&t, "QA1")[2].as_str(), history(&t).len()), ("failed", 1));
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `failed`"), "{}", part3(&t));

    let out = retire(&t, &args);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out).lines().next(), Some("retired QA1 (was failed): 목록을 고쳤어요"));
    assert_eq!(qa_row(&t, "QA1"), fresh(1), "the same retire reopens the scenario");
    let rows = history(&t);
    assert_eq!(rows.len(), 1, "no second row for the same result: {rows:?}");
    let kept = [&held[2], &held[5], &held[6], &held[7], &held[8], &held[9]];
    assert_eq!(rows[0][2..8].iter().collect::<Vec<_>>(), kept);
    assert_eq!(rows[0][9], "1");
    assert!(!t.0.join(PENDING).exists(), "the finished retire is no longer pending");
    assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `open`\n"), "{}", part3(&t));

    // A new recording is another result: retiring it adds the second row.
    record(&t, "QA1", "qa1-second.txt", &[]);
    let out = retire(&t, &["--qa", "QA1", "--why", "산출물을 잘못 골랐어요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let rows = history(&t);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!((rows[1][2].as_str(), rows[1][3].as_str(), rows[1][9].as_str()), ("met", "qa1-second.txt", "2"));
}

#[test]
fn R08_qa_rerun_identical_recordings_within_one_second_keep_both_retires() {
    let t = goal();
    let failed = ["--status", "failed", "--note", "목록이 비어 있어요"];
    record(&t, "QA1", "qa1-run.txt", &failed);
    let first = qa_row(&t, "QA1");
    let out = retire(&t, &["--qa", "QA1", "--why", "목록을 고쳤어요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    record(&t, "QA1", "qa1-run.txt", &failed);
    // recorded_at has one-second precision: the second recording takes the first one's stamp, so
    // the two recordings are identical whatever the clock did between them.
    let index = format!("{DIR}/qa.tsv");
    let second = qa_row(&t, "QA1").join("\t");
    t.write(&index, &t.read(&index).replace(&second, &first.join("\t")));
    assert_eq!(qa_row(&t, "QA1"), first, "the same artifact, status, command, note and stamp");

    let out = retire(&t, &["--qa", "QA1", "--why", "같은 결과를 다시 거둬요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out).lines().next(), Some("retired QA1 (was failed): 같은 결과를 다시 거둬요"));
    assert_eq!(qa_row(&t, "QA1"), fresh(1));
    let rows = history(&t);
    assert_eq!(rows.len(), 2, "each retire keeps its own row: {rows:?}");
    assert_eq!(rows[0][2..8], rows[1][2..8], "the two retired results are identical");
    let reason = |row: &Vec<String>| (row[8].clone(), row[9].clone());
    assert_eq!(reason(&rows[0]), ("목록을 고쳤어요".into(), "1".into()));
    assert_eq!(reason(&rows[1]), ("같은 결과를 다시 거둬요".into(), "2".into()));
}

/// What a QA1 retire leaves when it stops after each of its first three steps.
const POINTS: [&str; 3] = ["the pending write", "the history append", "the qa.tsv rewrite"];

/// A Goal with QA2 met whose retire of a QA1 result with `status` stopped after POINTS[point]: a
/// whole retire, then every file a later step writes put back as it was. The pending row is the
/// history row the retire wrote. QA2 is recorded first, since any record finishes the retire.
fn interrupted(point: usize, status: &str) -> Scratch {
    let t = goal();
    record(&t, "QA2", "qa2-run.txt", &[]);
    record(&t, "QA1", "qa1-run.txt", &["--status", status, "--note", "목록이 비어 있어요"]);
    let (index, request) = (format!("{DIR}/qa.tsv"), format!("{DIR}/request.md"));
    let before = [t.read(&index), t.read(&request)];
    let out = retire(&t, &["--qa", "QA1", "--why", "목록을 고쳤어요"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    t.write(PENDING, &format!("{}\n", history(&t)[0].join("\t")));
    if point == 0 {
        std::fs::remove_file(history_file(&t)).expect("no history before the append");
    }
    if point < 2 {
        t.write(&index, &before[0]);
    }
    t.write(&request, &before[1]);
    let held = format!("- QA1: 사용 시나리오 S1\n  - 상태: `{status}`\n");
    assert!(part3(&t).contains(&held), "{}", part3(&t));
    t
}

/// The interrupted retire is finished: one history row, attempt 1 with its reason, nothing pending.
fn finished(t: &Scratch, after: &str) {
    let rows = history(t);
    assert_eq!(rows.len(), 1, "after {after}: {rows:?}");
    let row = (rows[0][2].as_str(), rows[0][8].as_str(), rows[0][9].as_str());
    assert_eq!(row, ("failed", "목록을 고쳤어요", "1"), "after {after}");
    assert!(!t.0.join(PENDING).exists(), "after {after}: the retire is still pending");
}

#[test]
fn R08_qa_rerun_each_interruption_point_is_finished_by_the_same_retire_or_a_record() {
    for (point, after) in POINTS.iter().enumerate() {
        let t = interrupted(point, "failed");
        let out = retire(&t, &["--qa", "QA1", "--why", "목록을 고쳤어요"]);
        assert_eq!(out.status.code(), Some(0), "after {after}: {}", stderr(&out));
        let first = stdout(&out).lines().next().map(String::from);
        assert_eq!(first.as_deref(), Some("retired QA1 (was failed): 목록을 고쳤어요"), "after {after}");
        assert_eq!(qa_row(&t, "QA1"), fresh(1), "after {after}");
        finished(&t, after);
        assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `open`\n"), "after {after}: {}", part3(&t));

        let t = interrupted(point, "failed");
        record(&t, "QA1", "qa1-second.txt", &[]);
        let row = qa_row(&t, "QA1");
        assert_eq!((row[2].as_str(), row[5].as_str()), ("met", "qa1-second.txt"), "after {after}");
        finished(&t, after);
        assert!(part3(&t).contains("- QA1: 사용 시나리오 S1\n  - 상태: `met`\n"), "after {after}: {}", part3(&t));
    }
}

#[test]
fn R08_qa_rerun_a_history_ending_without_a_newline_is_refused_and_left_unchanged() {
    // A row an append cut short: its cells stop before the newline that ends a row.
    let partial = format!("{HISTORY}\nQA1\t");
    let file = format!("{DIR}/qa-history.tsv");
    let refused = |t: &Scratch, out: &Output| {
        assert_eq!(out.status.code(), Some(2), "{}", stdout(out));
        assert!(stderr(out).contains("qa-history.tsv does not end with a newline"), "{}", stderr(out));
        assert_eq!(t.read(&file), partial, "the history is left as it was");
        assert_eq!(qa_row(t, "QA1")[2], "failed", "the result is still recorded");
    };
    // Finishing a pending retire, by the same retire or by a record, refuses before any write.
    let t = interrupted(0, "failed");
    t.write(&file, &partial);
    let pending = t.read(PENDING);
    t.write("qa1-second.txt", "$ dstack status\nQA1 확인했어요.\nexit: 0\n");
    let again = ["evidence", "retire", "--qa", "QA1", "--why", "목록을 고쳤어요"];
    let record_again = ["evidence", "add", "--qa", "QA1", "--artifact", "qa1-second.txt", "--produced-by", "dstack status"];
    for args in [&again[..], &record_again[..]] {
        refused(&t, &t.run(args));
        assert_eq!(t.read(PENDING), pending, "{args:?}: the retire is still pending");
    }

    // A retire with nothing pending refuses before its pending write.
    let t = goal();
    record(&t, "QA1", "qa1-run.txt", &["--status", "failed", "--note", "목록이 비어 있어요"]);
    t.write(&file, &partial);
    refused(&t, &t.run(&again));
    assert!(!t.0.join(PENDING).exists(), "the refused retire left a pending row");
}

/// A plan.json whose one Plan has `status`: while it is in progress the Goal is not closing yet.
fn plan(status: &str) -> String {
    format!(
        r#"{{ "v": 2,
  "milestones": [ {{"id":"M1","slug":"rerun","order":1}} ],
  "plans": [ {{"id":"P1","milestone":"M1","slug":"rerun","files":["a"],"deps":[],
              "status":"{status}","worktree":"","started_at":"","done_at":"",
              "tasks":[ {{"id":"T1","slug":"rerun","covers":["R01"],"files":["a"],
                         "deps":[],"commit":"","done_at":""}} ] }} ] }}
"#
    )
}

/// verify, verify --at-close and report: each one's exit code and stdout.
fn judged(t: &Scratch) -> [(i32, String); 3] {
    let calls: [&[&str]; 3] = [&["verify"], &["verify", "--run", RUN, "--at-close"], &["report"]];
    calls.map(|args| {
        let out = t.run(args);
        (out.status.code().unwrap_or(-1), stdout(&out))
    })
}

#[test]
fn R08_qa_rerun_a_pending_retire_refuses_verify_and_report_until_it_is_finished() {
    let unfinished = "a retire of QA1 is unfinished (qa-retire.pending) — run the same dstack evidence retire --qa QA1 --why … again, or record QA1, which finishes it first";
    // The same retire finishes it while a Plan is in progress: QA1 is open, which only the close
    // itself refuses. A record finishes it once every Plan is done.
    for (status, after) in [("in-progress", [0, 1, 0]), ("done", [0, 0, 0])] {
        // A retire of a met QA1 stopped before its qa.tsv rewrite: the ledger still reads met, and
        // nothing but the pending retire refuses.
        let t = interrupted(1, "met");
        t.write(&format!("{DIR}/plan.json"), &plan(status));
        let refused = judged(&t);
        for (code, out) in &refused {
            assert_eq!(*code, 1, "{status}: {out}");
            assert!(out.contains(unfinished), "{status}: {out}");
        }
        let verify = &refused[0].1;
        assert!(verify.contains(&format!("QA1 (S1) FAIL (met): {unfinished}\n")), "{verify}");

        match status {
            "in-progress" => {
                let out = retire(&t, &["--qa", "QA1", "--why", "목록을 고쳤어요"]);
                assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
            }
            _ => record(&t, "QA1", "qa1-second.txt", &[]),
        }
        assert!(!t.0.join(PENDING).exists(), "{status}: the retire is still pending");
        for ((code, out), want) in judged(&t).into_iter().zip(after) {
            assert_eq!(code, want, "{status}: {out}");
            assert!(!out.contains("is unfinished"), "{status}: {out}");
        }
    }

    // A pending file that holds no history row cannot say which QA it retires, and one naming a QA
    // qa.tsv does not hold has no line to refuse on, so both refuse as undecidable.
    let t = interrupted(1, "met");
    let unknown = [&["QA9".to_string()][..], &history(&t)[0][1..]].concat().join("\t");
    for (held, named) in [("읽을 수 없어요", "holds no qa-history.tsv row"), (unknown.as_str(), "names QA9")] {
        t.write(PENDING, &format!("{held}\n"));
        for args in [&["verify"][..], &["report"]] {
            let out = t.run(args);
            assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stdout(&out));
            assert!(stderr(&out).contains(&format!("qa-retire.pending {named}")), "{args:?}: {}", stderr(&out));
        }
    }
}
