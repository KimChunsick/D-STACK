// tests/r07_e2e_brief.rs
// R07: dstack e2e brief --milestone prints the part of the e2e-runner brief the ledgers hold —
// each Plan's E2E focus, the R rows its tasks cover and the open cases, verbatim — and writes
// nothing, refusals included.

// The pipeline names a test after the R row it proves, which is not snake case.
#![allow(non_snake_case)]

#[path = "support/mode_settings.rs"]
mod support;

use std::process::{Command, Output};

use support::{tree, Scratch};

const RUN: &str = "20261001T000000Z_brief";
const DIR: &str = ".dstack/runs/20261001T000000Z_brief";
const R01: &str = "- [ ] **R01** 첫 화면을 보여줘요. — accept: `dstack status` 첫 줄에 run id가 보여요.";
const R02: &str = "- [ ] **R02** 보고서를 만들어요. — accept: `dstack report` 표에 두 번째 행이 보여요.";
const R03: &str = "- [ ] **R03** 아무 Plan도 맡지 않아요. — accept: 아무도 확인하지 않아요.";
const FOCUS1: &str = "status 첫 줄을 그대로 봐요";
const FOCUS2: &str = "report 표의 두 번째 행을 봐요";
const USAGE: &str = "usage: dstack e2e brief --milestone M<n>";

fn request(e2e: &str) -> String {
    format!(
        "---\nwork_type: cli\nroute: new-goal\nexternal_research: none\nrisk_axes: none\n\
         design_review: auto\nreview: on\ncodex_effort: high\ne2e: {e2e}\n\
         unit_tests: on\nvisual: none\nkorean_polish: on\n---\n\n# 검증 지시문 시험\n\n\
         마일스톤 E2E 지시문을 출력해요.\n\n{R01}\n{R02}\n{R03}\n"
    )
}

fn git(t: &Scratch, args: &[&str]) {
    let out = Command::new("git").args(args).current_dir(&t.0).output().expect("run git");
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// An open run with an approved request (its e2e value given), the synced ledger, Milestone M1
/// with two focused Plans whose tasks cover R01 and R02, and an empty Milestone M2. R03 is
/// covered by no Plan.
fn scratch(e2e: &str) -> Scratch {
    let t = Scratch::new();
    git(&t, &["init", "-q"]);
    let who = ["-c", "commit.gpgsign=false", "-c", "user.email=t@t", "-c", "user.name=t"];
    git(&t, &[&who[..], &["commit", "-q", "--allow-empty", "-m", "init"]].concat());
    t.init();
    t.run_fixture(RUN, None, true);
    // evidence add dates an artifact against the run's start.
    t.write(
        &format!("{DIR}/meta.tsv"),
        "status\topen\nowner_session\tfixture\nstarted_at\t2026-01-01T00:00:00Z\n",
    );
    t.write(&format!("{DIR}/request.md"), &request(e2e));
    t.write(&format!("{DIR}/request.approved"), "sha256 -  approved_at -\n");
    t.ok(&["cases", "sync"]);
    t.ok(&["milestone", "add", "core", "--goal", "지시문을 확인해요"]);
    t.ok(&["milestone", "add", "later"]);
    for (slug, purpose, focus) in [("first", "화면을 정리해요", FOCUS1), ("second", "보고서를 정리해요", FOCUS2)] {
        let file = format!("{slug}/a.sh");
        t.ok(&["plan", "add", slug, "--milestone", "M1", "--files", &file, "--purpose", purpose, "--e2e-focus", focus]);
    }
    t.ok(&["task", "add", "show", "--plan", "P1", "--covers", "R01", "--files", "first/a.sh"]);
    t.ok(&["task", "add", "table", "--plan", "P2", "--covers", "R02", "--files", "second/a.sh"]);
    t
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn header(e2e: &str) -> String {
    format!("e2e brief: run {RUN} — milestone M1 core — goal: 지시문을 확인해요 — e2e: {e2e}\n")
}

#[test]
fn R07_e2e_brief_prints_focus_rows_and_open_cases_verbatim() {
    let t = scratch("cli");
    t.write("R02-c1.txt", "$ dstack report\nR02 행이 보여요\nexit: 0\n");
    t.ok(&["evidence", "add", "--r", "R02", "--case", "c1", "--kind", "cli", "--artifact", "R02-c1.txt", "--produced-by", "dstack report"]);

    let before = tree(&t.0.join(".dstack"));
    let out = t.ok(&["e2e", "brief", "--milestone", "M1"]);
    assert_eq!(tree(&t.0.join(".dstack")), before, "e2e brief wrote to the store");
    let expected = format!(
        "{}\n## Plans\n\
         - P1 first (ready) — purpose: 화면을 정리해요 — E2E focus: {FOCUS1}\n\
         - P2 second (ready) — purpose: 보고서를 정리해요 — E2E focus: {FOCUS2}\n\
         \n## R rows (verbatim from request.md)\n{R01}\n{R02}\n\
         \n## Cases\n\
         | R | case | acceptance criterion (verbatim from the request) | Plans |\n\
         |---|---|---|---|\n\
         | R01 | c1 | `dstack status` 첫 줄에 run id가 보여요. | P1 |\n",
        header("cli")
    );
    assert_eq!(out, expected);
    // The met cli case and the workers' test cases stay out; no line names the uncovered R03.
    assert!(!out.contains("| R02 | c1 |"), "{out}");
    assert!(!out.contains("c-test"), "{out}");
    assert!(!out.contains("R03"), "{out}");
}

#[test]
fn R07_e2e_brief_says_when_no_case_is_open() {
    let t = scratch("cli");
    for r in ["R01", "R02"] {
        let file = format!("{r}-c1.txt");
        t.write(&file, &format!("$ dstack status\n{r} 확인\nexit: 0\n"));
        t.ok(&["evidence", "add", "--r", r, "--case", "c1", "--kind", "cli", "--artifact", &file, "--produced-by", "dstack status"]);
    }
    let out = t.ok(&["e2e", "brief", "--milestone", "M1"]);
    assert!(out.ends_with("\n## Cases\n(no open cases)\n"), "{out}");
}

#[test]
fn R07_e2e_brief_e2e_none_prints_the_skip_line() {
    let t = scratch("none");
    let out = t.ok(&["e2e", "brief", "--milestone", "M1"]);
    assert_eq!(
        out,
        format!("{}e2e: none — the milestone has no E2E cases to run\n", header("none"))
    );
}

#[test]
fn R07_e2e_brief_refusals_exit_one_and_write_nothing() {
    let t = scratch("cli");
    t.run_fixture("20261001T000001Z_bare", None, false);
    let cases: [(&[&str], &str); 6] = [
        (&["e2e", "brief"], "dstack: usage: dstack e2e brief --milestone M<n>\n"),
        (&["e2e", "brief", "--milestone", "M9"], "milestone not found: M9 (known: M1 M2)"),
        (&["e2e", "brief", "--milestone", "M2"], "milestone M2 has no plans"),
        (&["e2e", "brief", "--milestone", "M1", "--goal", "G1"], "unknown option: --goal"),
        (&["e2e", "brief", "--milestone", "M1", "M2"], "unexpected argument: M2"),
        (&["e2e", "brief", "--milestone", "M1", "--run", "20261001T000001Z_bare"], "no plan.json in "),
    ];
    for (args, message) in cases {
        let before = tree(&t.0.join(".dstack"));
        let out = t.run(args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stderr(&out));
        assert_eq!(stdout(&out), "", "{args:?}");
        assert!(stderr(&out).contains(message), "{args:?}: {}", stderr(&out));
        if !message.starts_with("no plan.json") {
            assert!(stderr(&out).contains(USAGE), "{args:?}: {}", stderr(&out));
        }
        assert_eq!(tree(&t.0.join(".dstack")), before, "{args:?} wrote to the store");
    }
}
